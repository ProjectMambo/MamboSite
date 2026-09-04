use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use crate::markdown::plain_text;
use crate::source::{DiscoveredSource, discover};
use crate::{
    ComrakMarkdownParser, Config, Diagnostic, HeadingRecord, MarkdownNode, MarkdownParser,
    NodeKind, Page, SCHEMA_VERSION, Site, SiteMetadata, derive_route, normalize_route,
    parse_frontmatter, slugify_segment,
};

pub struct Compiler {
    config: Config,
    parser: Box<dyn MarkdownParser>,
}

#[derive(Debug)]
pub struct CompileOutcome {
    pub site: Option<Site>,
    pub assets: Vec<crate::CompiledAsset>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Compiler {
    pub fn new(config: Config) -> Self {
        Self::with_parser(config, ComrakMarkdownParser::default())
    }

    pub fn with_parser(config: Config, parser: impl MarkdownParser + 'static) -> Self {
        Self {
            config,
            parser: Box::new(parser),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn compile(&self) -> CompileOutcome {
        let mut diagnostics = self.config.filesystem_diagnostics();
        if diagnostics.iter().any(Diagnostic::is_error) {
            return CompileOutcome {
                site: None,
                assets: Vec::new(),
                diagnostics,
            };
        }
        let entry_source = read_utf8(
            &self.config.content_root.join(&self.config.entry),
            &self.config.entry,
            &mut diagnostics,
        );
        let entry_frontmatter = entry_source
            .as_deref()
            .map(|source| parse_frontmatter(source, &self.config.entry, &self.config.frontmatter));
        if let Some(parsed) = &entry_frontmatter {
            diagnostics.extend(parsed.diagnostics.clone());
        }

        let mounts = entry_frontmatter
            .as_ref()
            .map(|parsed| parsed.metadata.mounts.as_slice())
            .unwrap_or_default();
        let discovery = discover(&self.config, mounts);
        diagnostics.extend(discovery.diagnostics);

        let mut pages = Vec::new();
        let mut routes_seen: BTreeMap<String, (String, String)> = BTreeMap::new();
        let mut ids_seen: HashMap<String, String> = HashMap::new();
        let mut route_owners = Vec::new();
        for source in discovery.sources {
            let parsed = if source.is_entry {
                entry_frontmatter.clone()
            } else {
                read_utf8(
                    &source.absolute_path,
                    &source.logical_path,
                    &mut diagnostics,
                )
                .map(|source_text| {
                    parse_frontmatter(&source_text, &source.logical_path, &self.config.frontmatter)
                })
            };
            let Some(parsed) = parsed else {
                continue;
            };
            if !source.is_entry {
                diagnostics.extend(parsed.diagnostics.clone());
                if !parsed.metadata.mounts.is_empty() {
                    diagnostics.push(
                        Diagnostic::error(
                            "MS4206",
                            "only the configured site entry may declare mounts",
                        )
                        .at_path(source.logical_path.clone()),
                    );
                }
            }

            let mut body = self
                .parser
                .parse(&parsed.body, parsed.body_start_line.saturating_sub(1));
            offset_byte_spans(&mut body, parsed.body_start_byte);
            crate::dialect::lower_directives(
                &mut body,
                &parsed.body,
                parsed.body_start_line,
                parsed.body_start_byte,
                &source.logical_path,
                &mut diagnostics,
            );
            crate::dialect::lower_obsidian(
                &mut body,
                &parsed.body,
                parsed.body_start_line,
                parsed.body_start_byte,
                &source.logical_path,
                &mut diagnostics,
            );
            validate_markdown(
                &body,
                &source.logical_path,
                self.config.markdown.raw_html,
                &mut diagnostics,
            );
            let headings = collect_headings(&body);
            validate_headings(&headings, &source.logical_path, &mut diagnostics);
            let blocks = collect_blocks(&body, &source.logical_path, &mut diagnostics);
            let directive_outcome = crate::validate_directives(
                &body,
                crate::DirectiveValidationContext {
                    logical_path: &source.logical_path,
                    body: &parsed.body,
                    body_start_line: parsed.body_start_line,
                    body_start_byte: parsed.body_start_byte,
                    is_index: Path::new(&source.route_source)
                        .file_name()
                        .and_then(|name| name.to_str())
                        == Some("index.md"),
                },
            );
            diagnostics.extend(directive_outcome.diagnostics);

            let route = match route_for_source(
                &source,
                parsed.metadata.slug.as_deref(),
                self.config.site.trailing_slash,
            ) {
                Ok(route) => route,
                Err(message) => {
                    diagnostics.push(
                        Diagnostic::error(
                            "MS4101",
                            format!("could not derive page route: {message}"),
                        )
                        .at_path(source.logical_path.clone()),
                    );
                    continue;
                }
            };
            let title = parsed
                .metadata
                .title
                .clone()
                .or_else(|| {
                    headings
                        .iter()
                        .find(|heading| heading.level == 1)
                        .map(|heading| heading.text.clone())
                })
                .unwrap_or_else(|| fallback_title(&source.route_source));
            let description = parsed
                .metadata
                .description
                .clone()
                .or_else(|| first_description(&body));
            let id = page_id(&source);

            if let Some(previous_path) = ids_seen.insert(id.clone(), source.logical_path.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        "MS4103",
                        format!("stable page identifier `{id}` occurs more than once"),
                    )
                    .at_path(source.logical_path.clone())
                    .with_related(previous_path, crate::SourceSpan::point(1, 1)),
                );
            }
            if let Some((_, previous_path)) = routes_seen.get(&route) {
                diagnostics.push(
                    Diagnostic::error(
                        "MS4102",
                        format!("route `{route}` is produced by more than one page"),
                    )
                    .at_path(source.logical_path.clone())
                    .with_related(previous_path.clone(), crate::SourceSpan::point(1, 1)),
                );
            } else {
                routes_seen.insert(route.clone(), (id.clone(), source.logical_path.clone()));
            }
            route_owners.push(RouteOwner {
                route: route.clone(),
                source_path: source.logical_path.clone(),
                mount_prefix: source.mount_prefix.clone(),
            });

            pages.push(Page::from_parts(
                id,
                route,
                source.logical_path,
                title,
                description,
                parsed.metadata,
                headings,
                blocks,
                directive_outcome.directives,
                body,
            ));
        }
        validate_mount_namespaces(&route_owners, &mut diagnostics);

        pages.sort_by(|left, right| {
            left.route
                .cmp(&right.route)
                .then_with(|| left.source_path.cmp(&right.source_path))
        });
        populate_children(&mut pages, self.config.site.trailing_slash);
        crate::reference::resolve_links(
            &mut pages,
            self.config.markdown.strict_links,
            self.config.site.trailing_slash,
            self.config.markdown.max_embed_depth,
            &mut diagnostics,
        );
        let asset_outcome = crate::asset::compile(&self.config, &mut pages);
        diagnostics.extend(asset_outcome.diagnostics);
        let assets = asset_outcome.assets;

        let routes: BTreeMap<_, _> = pages
            .iter()
            .map(|page| (page.route.clone(), page.id.clone()))
            .collect();
        let entry_page = routes.get("/").cloned();
        if entry_page.is_none() {
            diagnostics.push(
                Diagnostic::error(
                    "MS4104",
                    "the configured site entry did not produce the `/` route",
                )
                .at_path(self.config.entry.clone()),
            );
        }

        diagnostics.sort_by(diagnostic_order);
        if diagnostics.iter().any(Diagnostic::is_error) {
            return CompileOutcome {
                site: None,
                assets,
                diagnostics,
            };
        }

        let Some(entry_page) = entry_page else {
            return CompileOutcome {
                site: None,
                assets,
                diagnostics,
            };
        };
        let root_title = pages
            .iter()
            .find(|page| page.id == entry_page)
            .map_or_else(|| "MamboSite".to_owned(), |page| page.title.clone());
        let site = Site {
            schema_version: SCHEMA_VERSION,
            site: SiteMetadata {
                title: self
                    .config
                    .site
                    .title
                    .clone()
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(root_title),
                url: self
                    .config
                    .site
                    .url
                    .clone()
                    .filter(|value| !value.trim().is_empty()),
                base_path: self.config.site.base_path.clone(),
                language: self.config.site.language.clone(),
                trailing_slash: self.config.site.trailing_slash,
            },
            entry_page,
            routes,
            pages,
        };
        CompileOutcome {
            site: Some(site),
            assets,
            diagnostics,
        }
    }
}

struct RouteOwner {
    route: String,
    source_path: String,
    mount_prefix: Option<String>,
}

fn validate_mount_namespaces(owners: &[RouteOwner], diagnostics: &mut Vec<Diagnostic>) {
    let mount_roots: BTreeMap<_, _> = owners
        .iter()
        .filter_map(|owner| {
            owner.mount_prefix.as_ref().map(|prefix| {
                let root_source = owners
                    .iter()
                    .find(|candidate| {
                        candidate.mount_prefix.as_ref() == Some(prefix)
                            && candidate.route == *prefix
                    })
                    .map_or(owner.source_path.as_str(), |candidate| {
                        candidate.source_path.as_str()
                    });
                (prefix.as_str(), root_source)
            })
        })
        .collect();
    for physical in owners
        .iter()
        .filter(|owner| owner.mount_prefix.is_none() && owner.route != "/")
    {
        if let Some((prefix, mount_source)) = mount_roots
            .iter()
            .find(|(prefix, _)| route_is_strict_descendant(&physical.route, prefix))
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS4207",
                    format!(
                        "physical route `{}` is inside mounted namespace `{prefix}`",
                        physical.route
                    ),
                )
                .at_path(physical.source_path.clone())
                .with_related(*mount_source, crate::SourceSpan::point(1, 1)),
            );
        }
    }
}

fn route_is_strict_descendant(route: &str, prefix: &str) -> bool {
    let route = route.trim_end_matches('/');
    let prefix = prefix.trim_end_matches('/');
    route
        .strip_prefix(prefix)
        .is_some_and(|suffix| suffix.starts_with('/') && suffix.len() > 1)
}

fn read_utf8(path: &Path, logical_path: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            diagnostics.push(
                Diagnostic::error("MS2106", format!("could not read source file: {error}"))
                    .at_path(logical_path),
            );
            return None;
        }
    };
    if let Ok(source) = String::from_utf8(bytes) {
        Some(source)
    } else {
        diagnostics.push(
            Diagnostic::error("MS2107", "Markdown source must be valid UTF-8")
                .at_path(logical_path),
        );
        None
    }
}

fn offset_byte_spans(node: &mut MarkdownNode, offset: usize) {
    if let Some(span) = &mut node.span {
        span.start_byte = span.start_byte.map(|byte| byte + offset);
        span.end_byte = span.end_byte.map(|byte| byte + offset);
    }
    for child in &mut node.children {
        offset_byte_spans(child, offset);
    }
}

fn route_for_source(
    source: &DiscoveredSource,
    slug: Option<&str>,
    trailing_slash: bool,
) -> Result<String, String> {
    if source.is_entry {
        return Ok("/".to_owned());
    }
    let derived = derive_route(&source.route_source, slug, trailing_slash)?;
    let Some(prefix) = &source.mount_prefix else {
        return Ok(derived);
    };
    if derived == "/" {
        return Ok(prefix.clone());
    }
    let joined = format!(
        "{}/{}",
        prefix.trim_end_matches('/'),
        derived.trim_matches('/')
    );
    normalize_route(&joined, trailing_slash)
}

fn collect_headings(root: &MarkdownNode) -> Vec<HeadingRecord> {
    fn visit(
        node: &MarkdownNode,
        headings: &mut Vec<HeadingRecord>,
        used: &mut HashMap<String, usize>,
    ) {
        if let NodeKind::Heading { level, .. } = node.kind {
            let text = plain_text(node);
            let base = {
                let slug = slugify_segment(&text);
                if slug.is_empty() {
                    "section".to_owned()
                } else {
                    slug
                }
            };
            let count = used.entry(base.clone()).or_default();
            *count += 1;
            let id = if *count == 1 {
                base
            } else {
                format!("{base}-{}", *count)
            };
            headings.push(HeadingRecord {
                id,
                level,
                text,
                span: node.span,
            });
        }
        for child in &node.children {
            visit(child, headings, used);
        }
    }

    let mut headings = Vec::new();
    visit(root, &mut headings, &mut HashMap::new());
    headings
}

fn collect_blocks(
    root: &MarkdownNode,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<crate::BlockRecord> {
    fn visit(
        node: &MarkdownNode,
        path: &str,
        blocks: &mut Vec<crate::BlockRecord>,
        seen: &mut BTreeMap<String, Option<crate::SourceSpan>>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(id) = &node.block_id {
            if let Some(previous) = seen.get(id) {
                let mut diagnostic = Diagnostic::error(
                    "MS3308",
                    format!("block identifier `^{id}` occurs more than once"),
                );
                diagnostic = if let Some(span) = node.span {
                    diagnostic.at(path, span)
                } else {
                    diagnostic.at_path(path)
                };
                if let Some(span) = previous {
                    diagnostic = diagnostic.with_related(path, *span);
                }
                diagnostics.push(diagnostic);
            } else {
                seen.insert(id.clone(), node.span);
                blocks.push(crate::BlockRecord {
                    id: id.clone(),
                    span: node.span,
                });
            }
        }
        for child in &node.children {
            visit(child, path, blocks, seen, diagnostics);
        }
    }

    let mut blocks = Vec::new();
    visit(root, path, &mut blocks, &mut BTreeMap::new(), diagnostics);
    blocks
}

fn validate_headings(headings: &[HeadingRecord], path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let h1_count = headings.iter().filter(|heading| heading.level == 1).count();
    if h1_count > 1 {
        let heading = headings
            .iter()
            .find(|heading| heading.level == 1)
            .expect("h1 exists");
        let mut diagnostic = Diagnostic::warning(
            "MS3303",
            format!("page contains {h1_count} level-one headings"),
        );
        diagnostic = if let Some(span) = heading.span {
            diagnostic.at(path, span)
        } else {
            diagnostic.at_path(path)
        };
        diagnostics.push(diagnostic);
    }
    for pair in headings.windows(2) {
        if pair[1].level > pair[0].level + 1 {
            let mut diagnostic = Diagnostic::warning(
                "MS3304",
                format!(
                    "heading level jumps from {} to {}",
                    pair[0].level, pair[1].level
                ),
            );
            diagnostic = if let Some(span) = pair[1].span {
                diagnostic.at(path, span)
            } else {
                diagnostic.at_path(path)
            };
            diagnostics.push(diagnostic);
        }
    }
}

fn validate_markdown(
    node: &MarkdownNode,
    path: &str,
    raw_html_enabled: bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match &node.kind {
        NodeKind::CodeBlock {
            fenced: true,
            closed: false,
            ..
        } => push_node_diagnostic(
            Diagnostic::error("MS3301", "fenced code block is not closed"),
            node,
            path,
            diagnostics,
        ),
        NodeKind::HtmlBlock { .. } | NodeKind::HtmlInline { .. } if !raw_html_enabled => {
            push_node_diagnostic(
                Diagnostic::warning(
                    "MS3302",
                    "raw HTML is preserved in the syntax tree but disabled by renderer policy",
                ),
                node,
                path,
                diagnostics,
            );
        }
        _ => {}
    }
    for child in &node.children {
        validate_markdown(child, path, raw_html_enabled, diagnostics);
    }
}

fn push_node_diagnostic(
    diagnostic: Diagnostic,
    node: &MarkdownNode,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.push(if let Some(span) = node.span {
        diagnostic.at(path, span)
    } else {
        diagnostic.at_path(path)
    });
}

fn first_description(root: &MarkdownNode) -> Option<String> {
    root.children.iter().find_map(|node| {
        if !matches!(node.kind, NodeKind::Paragraph) || contains_description_exclusion(node) {
            return None;
        }
        let text = plain_text(node);
        (!text.is_empty() && !text.starts_with("::")).then_some(text)
    })
}

fn contains_description_exclusion(node: &MarkdownNode) -> bool {
    matches!(
        node.kind,
        NodeKind::Directive { .. } | NodeKind::ObsidianEmbed { .. }
    ) || node.children.iter().any(contains_description_exclusion)
}

fn fallback_title(route_source: &str) -> String {
    let path = Path::new(route_source);
    let is_index = path.file_name().and_then(|name| name.to_str()) == Some("index.md");
    let value = if is_index {
        path.parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Home")
    } else {
        path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled")
    };
    value.replace(['-', '_'], " ")
}

fn page_id(source: &DiscoveredSource) -> String {
    let identity = format!(
        "{}\0{}",
        source.mount_prefix.as_deref().unwrap_or("physical"),
        source.logical_path
    );
    let hash = identity
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("p_{hash:016x}")
}

fn populate_children(pages: &mut [Page], trailing_slash: bool) {
    let route_to_index: BTreeMap<_, _> = pages
        .iter()
        .enumerate()
        .map(|(index, page)| (page.route.clone(), index))
        .collect();
    let mut additions = Vec::new();
    for (child_index, page) in pages.iter().enumerate() {
        if let Some(parent_route) =
            nearest_parent_route(&page.route, &route_to_index, trailing_slash)
            && let Some(parent_index) = route_to_index.get(&parent_route)
        {
            additions.push((*parent_index, child_index));
        }
    }
    for (parent, child) in additions {
        let id = pages[child].id.clone();
        pages[parent].children.push(id);
    }

    let sort_keys: HashMap<_, _> = pages
        .iter()
        .map(|page| {
            (
                page.id.clone(),
                (
                    page.order,
                    page.title.to_lowercase(),
                    page.source_path.clone(),
                ),
            )
        })
        .collect();
    for page in pages {
        page.children.sort_by(|left, right| {
            let left = &sort_keys[left];
            let right = &sort_keys[right];
            order_value(left.0, right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
    }
}

fn nearest_parent_route(
    route: &str,
    routes: &BTreeMap<String, usize>,
    trailing_slash: bool,
) -> Option<String> {
    if route == "/" {
        return None;
    }
    let mut path = route.trim_matches('/');
    loop {
        let candidate = if let Some(index) = path.rfind('/') {
            path = &path[..index];
            let suffix = if trailing_slash { "/" } else { "" };
            format!("/{path}{suffix}")
        } else {
            "/".to_owned()
        };
        if routes.contains_key(&candidate) {
            return Some(candidate);
        }
        if candidate == "/" {
            return None;
        }
    }
}

fn order_value(left: Option<i64>, right: Option<i64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn diagnostic_order(left: &Diagnostic, right: &Diagnostic) -> Ordering {
    let left_path = left.primary.as_ref().map(|location| location.path.as_str());
    let right_path = right
        .primary
        .as_ref()
        .map(|location| location.path.as_str());
    left_path
        .cmp(&right_path)
        .then_with(|| {
            left.primary
                .as_ref()
                .map(|location| location.span.start.line)
                .cmp(
                    &right
                        .primary
                        .as_ref()
                        .map(|location| location.span.start.line),
                )
        })
        .then_with(|| left.code.cmp(&right.code))
        .then_with(|| left.message.cmp(&right.message))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::Compiler;
    use crate::Config;

    #[test]
    fn compiles_physical_and_mounted_pages_into_a_site() {
        const HOME_SOURCE: &str = concat!(
            "---\ntitle: Example\nmounts:\n",
            "  - path: /tool\n    source: _mounts/tool/index.md\n---\n",
            "# Home\n\nWelcome to the site.\n",
        );

        let temp = tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(docs.join("blog")).unwrap();
        fs::create_dir_all(docs.join("_mounts/tool/deep")).unwrap();
        fs::write(docs.join("index.md"), HOME_SOURCE).unwrap();
        fs::write(
            docs.join("blog/First Post.md"),
            "---\ntags: web/test\nperiod: 2026\n---\n# First Post\n",
        )
        .unwrap();
        fs::write(docs.join("_mounts/tool/index.md"), "# Tool\n").unwrap();
        fs::write(docs.join("_mounts/tool/Guide.md"), "# Guide\n").unwrap();
        fs::write(docs.join("_mounts/tool/deep/Topic.md"), "# Nested topic\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"\n[site]\nlanguage=\"en-SG\"",
            temp.path().join("mambo.toml"),
        )
        .unwrap();

        let compiled = Compiler::new(config).compile();
        assert_eq!(compiled.diagnostics, []);
        let site = compiled.site.expect("site");
        assert_eq!(site.pages.len(), 5);
        assert!(site.routes.contains_key("/"));
        assert!(site.routes.contains_key("/blog/first-post/"));
        assert!(site.routes.contains_key("/tool/"));
        assert!(site.routes.contains_key("/tool/guide/"));
        assert!(site.routes.contains_key("/tool/deep/topic/"));
        let home = site.pages.iter().find(|page| page.route == "/").unwrap();
        assert_eq!(
            home.body.children[0].span.unwrap().start_byte,
            HOME_SOURCE.find("# Home")
        );
        let post = site
            .pages
            .iter()
            .find(|page| page.route == "/blog/first-post/")
            .unwrap();
        assert_eq!(post.data["period"], 2026);
    }

    #[test]
    fn route_collisions_are_build_errors() {
        let temp = tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(docs.join("same")).unwrap();
        fs::write(docs.join("index.md"), "# Home").unwrap();
        fs::write(docs.join("same.md"), "leaf").unwrap();
        fs::write(docs.join("same/index.md"), "folder").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temp.path().join("mambo.toml"),
        )
        .unwrap();
        let compiled = Compiler::new(config).compile();
        assert!(compiled.site.is_none());
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|item| item.code == "MS4102")
        );
    }

    #[test]
    fn derives_descriptions_only_from_top_level_paragraphs() {
        let temp = tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            "# Home\n\n> nested quote must not become the summary\n\nTop-level summary.\n",
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temp.path().join("mambo.toml"),
        )
        .unwrap();

        let site = Compiler::new(config).compile().site.unwrap();
        assert_eq!(
            site.pages[0].description.as_deref(),
            Some("Top-level summary.")
        );
    }

    #[test]
    fn rejects_physical_pages_inside_a_mount_namespace() {
        let temp = tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(docs.join("_mounts/project")).unwrap();
        fs::create_dir_all(docs.join("project")).unwrap();
        fs::write(
            docs.join("index.md"),
            "---\nmounts:\n  - path: /project\n    source: _mounts/project/index.md\n---\n# Home\n",
        )
        .unwrap();
        fs::write(docs.join("_mounts/project/index.md"), "# Project\n").unwrap();
        fs::write(docs.join("project/local.md"), "# Local\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temp.path().join("mambo.toml"),
        )
        .unwrap();

        let compiled = Compiler::new(config).compile();
        assert!(compiled.site.is_none());
        assert!(
            compiled
                .diagnostics
                .iter()
                .any(|item| item.code == "MS4207")
        );
    }
}
