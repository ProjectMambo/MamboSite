use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use crate::{
    Diagnostic, MarkdownNode, NodeKind, Page, ReferenceSyntax, ResolvedEmbed, ResolvedFragment,
    ResolvedLink, ResolvedLinkTarget, SourceSpan, normalize_route, slugify_segment,
};

#[derive(Debug, Clone)]
struct IndexedPage {
    id: String,
    route: String,
    source_path: String,
    source_without_extension: String,
    note_path: String,
    stem: String,
    aliases: Vec<String>,
    headings: Vec<crate::HeadingRecord>,
    blocks: Vec<crate::BlockRecord>,
}

#[derive(Debug)]
struct RawLink {
    syntax: ReferenceSyntax,
    destination: String,
    span: Option<SourceSpan>,
}

#[derive(Debug)]
struct RawEmbed {
    destination: String,
    option: Option<String>,
    span: Option<SourceSpan>,
}

#[derive(Debug)]
struct ResolutionFailure {
    code: &'static str,
    message: String,
    candidates: Vec<usize>,
    always_error: bool,
}

enum Resolution {
    Target(ResolvedLinkTarget),
    /// Local non-Markdown files are handled by the asset pass.
    DeferredAsset,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn resolve_links(
    pages: &mut [Page],
    strict: bool,
    trailing_slash: bool,
    max_embed_depth: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let index: Vec<_> = pages.iter().map(IndexedPage::from).collect();
    let mut backlinks: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for (source_index, page) in pages.iter_mut().enumerate() {
        let mut raw_links = Vec::new();
        let mut raw_embeds = Vec::new();
        collect_references(&page.body, &mut raw_links, &mut raw_embeds);
        let mut outgoing = Vec::new();

        for raw in raw_links {
            match resolve_one(&index, source_index, &raw, trailing_slash) {
                Ok(Resolution::DeferredAsset) => {}
                Ok(Resolution::Target(target)) => {
                    if let ResolvedLinkTarget::Page { page_id, .. } = &target {
                        backlinks
                            .entry(page_id.clone())
                            .or_default()
                            .insert(page.id.clone());
                    }
                    outgoing.push(ResolvedLink {
                        syntax: raw.syntax,
                        authored_destination: raw.destination,
                        target,
                        span: raw.span,
                    });
                }
                Err(failure) => {
                    let mut diagnostic = if strict || failure.always_error {
                        Diagnostic::error(failure.code, failure.message)
                    } else {
                        Diagnostic::warning(failure.code, failure.message)
                    };
                    diagnostic = if let Some(span) = raw.span {
                        diagnostic.at(page.source_path.clone(), span)
                    } else {
                        diagnostic.at_path(page.source_path.clone())
                    };
                    for candidate in failure.candidates {
                        diagnostic = diagnostic.with_related(
                            index[candidate].source_path.clone(),
                            SourceSpan::point(1, 1),
                        );
                    }
                    diagnostics.push(diagnostic);
                    if !strict && !failure.always_error {
                        outgoing.push(ResolvedLink {
                            syntax: raw.syntax,
                            authored_destination: raw.destination,
                            target: ResolvedLinkTarget::Unresolved,
                            span: raw.span,
                        });
                    }
                }
            }
        }
        page.outgoing_links = outgoing;

        let mut embeds = Vec::new();
        for (ordinal, raw) in raw_embeds.into_iter().enumerate() {
            let instance_id = embed_instance_id(&page.id, &raw, ordinal);
            let link = RawLink {
                syntax: ReferenceSyntax::Wiki,
                destination: raw.destination.clone(),
                span: raw.span,
            };
            match resolve_one(&index, source_index, &link, trailing_slash) {
                Ok(Resolution::DeferredAsset) => {}
                Ok(Resolution::Target(target)) => {
                    if let ResolvedLinkTarget::Page { page_id, .. } = &target {
                        backlinks
                            .entry(page_id.clone())
                            .or_default()
                            .insert(page.id.clone());
                    }
                    embeds.push(ResolvedEmbed {
                        authored_destination: raw.destination,
                        option: raw.option,
                        instance_id,
                        target,
                        span: raw.span,
                    });
                }
                Err(failure) => {
                    let retain_unresolved = !strict && !failure.always_error;
                    diagnostics.push(failure_diagnostic(
                        failure,
                        strict,
                        &page.source_path,
                        raw.span,
                        &index,
                    ));
                    if retain_unresolved {
                        embeds.push(ResolvedEmbed {
                            authored_destination: raw.destination,
                            option: raw.option,
                            instance_id,
                            target: ResolvedLinkTarget::Unresolved,
                            span: raw.span,
                        });
                    }
                }
            }
        }
        page.embeds = embeds;
    }

    validate_embed_graph(pages, &index, max_embed_depth, diagnostics);
    for page in pages {
        page.backlinks = backlinks
            .remove(&page.id)
            .map_or_else(Vec::new, |sources| sources.into_iter().collect());
    }
}

impl From<&Page> for IndexedPage {
    fn from(page: &Page) -> Self {
        let source_without_extension = strip_markdown_extension(&page.source_path);
        let note_path = source_without_extension
            .strip_suffix("/index")
            .unwrap_or(&source_without_extension)
            .to_owned();
        let stem = note_path
            .rsplit('/')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or("index")
            .to_owned();
        Self {
            id: page.id.clone(),
            route: page.route.clone(),
            source_path: page.source_path.clone(),
            source_without_extension,
            note_path,
            stem,
            aliases: page.aliases.clone(),
            headings: page.headings.clone(),
            blocks: page.blocks.clone(),
        }
    }
}

fn collect_references(node: &MarkdownNode, links: &mut Vec<RawLink>, embeds: &mut Vec<RawEmbed>) {
    match &node.kind {
        NodeKind::Link { destination, .. } => links.push(RawLink {
            syntax: ReferenceSyntax::Markdown,
            destination: destination.clone(),
            span: node.span,
        }),
        NodeKind::WikiLink { destination } => links.push(RawLink {
            syntax: ReferenceSyntax::Wiki,
            destination: destination.clone(),
            span: node.span,
        }),
        NodeKind::ObsidianEmbed {
            destination,
            option,
        } => embeds.push(RawEmbed {
            destination: destination.clone(),
            option: option.clone(),
            span: node.span,
        }),
        _ => {}
    }
    for child in &node.children {
        collect_references(child, links, embeds);
    }
}

fn failure_diagnostic(
    failure: ResolutionFailure,
    strict: bool,
    source_path: &str,
    span: Option<SourceSpan>,
    index: &[IndexedPage],
) -> Diagnostic {
    let mut diagnostic = if strict || failure.always_error {
        Diagnostic::error(failure.code, failure.message)
    } else {
        Diagnostic::warning(failure.code, failure.message)
    };
    diagnostic = if let Some(span) = span {
        diagnostic.at(source_path, span)
    } else {
        diagnostic.at_path(source_path)
    };
    for candidate in failure.candidates {
        diagnostic = diagnostic.with_related(
            index[candidate].source_path.clone(),
            SourceSpan::point(1, 1),
        );
    }
    diagnostic
}

fn embed_instance_id(source_page: &str, embed: &RawEmbed, ordinal: usize) -> String {
    let identity = format!(
        "{source_page}\0{}\0{ordinal}\0{}",
        embed
            .span
            .and_then(|span| span.start_byte)
            .map_or_else(|| "unknown".to_owned(), |byte| byte.to_string()),
        embed.destination
    );
    let hash = identity
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("e_{hash:016x}")
}

fn validate_embed_graph(
    pages: &[Page],
    index: &[IndexedPage],
    max_depth: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let positions: BTreeMap<_, _> = index
        .iter()
        .enumerate()
        .map(|(position, page)| (page.id.as_str(), position))
        .collect();
    let edges: Vec<Vec<(usize, Option<SourceSpan>)>> = pages
        .iter()
        .map(|page| {
            page.embeds
                .iter()
                .filter_map(|embed| {
                    let ResolvedLinkTarget::Page { page_id, .. } = &embed.target else {
                        return None;
                    };
                    positions
                        .get(page_id.as_str())
                        .map(|target| (*target, embed.span))
                })
                .collect()
        })
        .collect();
    let mut colors = vec![0_u8; pages.len()];
    let mut stack = Vec::new();
    let mut reported_cycles = BTreeSet::new();
    let mut has_cycles = false;
    for page in 0..pages.len() {
        if colors[page] == 0 {
            visit_embed_cycles(
                page,
                pages,
                &edges,
                &mut colors,
                &mut stack,
                &mut reported_cycles,
                &mut has_cycles,
                diagnostics,
            );
        }
    }

    // A cycle already makes expansion unbounded and has its own precise
    // diagnostic. Longest-path memoization is defined only for the remaining
    // DAG, where every node and edge can be evaluated once.
    if !has_cycles {
        validate_embed_depth(pages, &edges, max_depth, diagnostics);
    }
}

#[allow(clippy::too_many_arguments)]
fn visit_embed_cycles(
    page: usize,
    pages: &[Page],
    edges: &[Vec<(usize, Option<SourceSpan>)>],
    colors: &mut [u8],
    stack: &mut Vec<usize>,
    reported_cycles: &mut BTreeSet<String>,
    has_cycles: &mut bool,
    diagnostics: &mut Vec<Diagnostic>,
) {
    colors[page] = 1;
    stack.push(page);
    for &(target, span) in &edges[page] {
        match colors[target] {
            0 => visit_embed_cycles(
                target,
                pages,
                edges,
                colors,
                stack,
                reported_cycles,
                has_cycles,
                diagnostics,
            ),
            1 => {
                *has_cycles = true;
                let cycle_start = stack
                    .iter()
                    .position(|candidate| *candidate == target)
                    .unwrap_or(0);
                let mut cycle = stack[cycle_start..].to_vec();
                cycle.push(target);
                let signature = cycle
                    .iter()
                    .map(|position| pages[*position].id.as_str())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                if reported_cycles.insert(signature.clone()) {
                    let route_chain = cycle
                        .iter()
                        .map(|position| pages[*position].route.as_str())
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    let diagnostic =
                        Diagnostic::error("MS5201", "embed cycle detected").with_note(route_chain);
                    diagnostics.push(if let Some(span) = span {
                        diagnostic.at(pages[page].source_path.clone(), span)
                    } else {
                        diagnostic.at_path(pages[page].source_path.clone())
                    });
                }
            }
            _ => {}
        }
    }
    stack.pop();
    colors[page] = 2;
}

fn validate_embed_depth(
    pages: &[Page],
    edges: &[Vec<(usize, Option<SourceSpan>)>],
    max_depth: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut memo = vec![None; pages.len()];
    for page in 0..pages.len() {
        longest_embed_path(page, edges, &mut memo);
    }

    let over_depth: Vec<_> = memo
        .iter()
        .map(|depth| depth.is_some_and(|depth| depth > max_depth))
        .collect();
    let mut nested_over_depth = vec![false; pages.len()];
    for (source, targets) in edges.iter().enumerate() {
        if !over_depth[source] {
            continue;
        }
        for &(target, _) in targets {
            if over_depth[target] {
                nested_over_depth[target] = true;
            }
        }
    }

    // Report one deterministic longest witness from each over-depth root. This
    // avoids emitting the same long suffix once for every ancestor.
    for start in 0..pages.len() {
        if !over_depth[start] || nested_over_depth[start] {
            continue;
        }
        let (chain, spans) = embed_depth_witness(start, edges, &memo);
        let Some(&source) = chain.get(max_depth) else {
            continue;
        };
        let span = spans.get(max_depth).copied().flatten();
        let route_chain = chain
            .iter()
            .take(max_depth.saturating_add(2))
            .map(|position| pages[*position].route.as_str())
            .collect::<Vec<_>>()
            .join(" -> ");
        let diagnostic = Diagnostic::error(
            "MS5202",
            format!("embed depth exceeds configured maximum of {max_depth}"),
        )
        .with_note(route_chain);
        diagnostics.push(if let Some(span) = span {
            diagnostic.at(pages[source].source_path.clone(), span)
        } else {
            diagnostic.at_path(pages[source].source_path.clone())
        });
    }
}

fn longest_embed_path(
    page: usize,
    edges: &[Vec<(usize, Option<SourceSpan>)>],
    memo: &mut [Option<usize>],
) -> usize {
    if let Some(depth) = memo[page] {
        return depth;
    }
    let depth = edges[page]
        .iter()
        .map(|(target, _)| longest_embed_path(*target, edges, memo).saturating_add(1))
        .max()
        .unwrap_or(0);
    memo[page] = Some(depth);
    depth
}

fn embed_depth_witness(
    start: usize,
    edges: &[Vec<(usize, Option<SourceSpan>)>],
    memo: &[Option<usize>],
) -> (Vec<usize>, Vec<Option<SourceSpan>>) {
    let mut chain = vec![start];
    let mut spans = Vec::new();
    let mut current = start;
    while let Some(current_depth) = memo[current].filter(|depth| *depth > 0) {
        let expected = current_depth - 1;
        let Some(&(target, span)) = edges[current]
            .iter()
            .find(|(target, _)| memo[*target] == Some(expected))
        else {
            break;
        };
        chain.push(target);
        spans.push(span);
        current = target;
    }
    (chain, spans)
}

fn resolve_one(
    index: &[IndexedPage],
    source_index: usize,
    raw: &RawLink,
    trailing_slash: bool,
) -> Result<Resolution, ResolutionFailure> {
    reject_control_characters(&raw.destination)?;
    if raw.syntax == ReferenceSyntax::Markdown {
        if let Some(scheme) = url_scheme(&raw.destination) {
            return if ["http", "https", "mailto", "tel"]
                .iter()
                .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
            {
                Ok(Resolution::Target(ResolvedLinkTarget::External {
                    href: raw.destination.clone(),
                }))
            } else {
                Err(ResolutionFailure {
                    code: "MS5104",
                    message: format!("URL scheme `{scheme}` is not allowed"),
                    candidates: Vec::new(),
                    always_error: true,
                })
            };
        }
        if raw.destination.starts_with("//") {
            return Ok(Resolution::Target(ResolvedLinkTarget::External {
                href: raw.destination.clone(),
            }));
        }
    }

    let (authored_path, authored_fragment) = split_fragment(&raw.destination);
    let path = percent_decode(authored_path).map_err(|message| ResolutionFailure {
        code: "MS5104",
        message,
        candidates: Vec::new(),
        always_error: true,
    })?;
    let fragment = authored_fragment
        .map(percent_decode)
        .transpose()
        .map_err(|message| ResolutionFailure {
            code: "MS5104",
            message,
            candidates: Vec::new(),
            always_error: true,
        })?;
    reject_control_characters(&path)?;
    if path.contains('?') {
        return Err(ResolutionFailure {
            code: "MS5104",
            message: "internal links may not contain a query string".to_owned(),
            candidates: Vec::new(),
            always_error: true,
        });
    }

    let target_index = if raw.syntax == ReferenceSyntax::Markdown && path.starts_with('/') {
        resolve_route(index, &path, trailing_slash)?
    } else if path.is_empty() {
        source_index
    } else {
        if is_non_markdown_file(&path) {
            return Ok(Resolution::DeferredAsset);
        }
        resolve_note(index, source_index, &path)?
    };
    let resolved_fragment = resolve_fragment(&index[target_index], fragment.as_deref())?;
    Ok(Resolution::Target(ResolvedLinkTarget::Page {
        page_id: index[target_index].id.clone(),
        route: index[target_index].route.clone(),
        fragment: resolved_fragment,
    }))
}

fn resolve_route(
    index: &[IndexedPage],
    path: &str,
    trailing_slash: bool,
) -> Result<usize, ResolutionFailure> {
    let route = normalize_route(path, trailing_slash).map_err(|message| ResolutionFailure {
        code: "MS5104",
        message: format!("invalid root-relative route `{path}`: {message}"),
        candidates: Vec::new(),
        always_error: true,
    })?;
    choose(
        index,
        index
            .iter()
            .enumerate()
            .filter_map(|(position, page)| (page.route == route).then_some(position))
            .collect(),
        &format!("route `{route}`"),
    )
}

fn resolve_note(
    index: &[IndexedPage],
    source_index: usize,
    authored_path: &str,
) -> Result<usize, ResolutionFailure> {
    let authored_path = authored_path.trim();
    let source_directory = index[source_index]
        .source_path
        .rsplit_once('/')
        .map_or("", |(directory, _)| directory);
    let relative_target =
        normalize_logical(source_directory, authored_path).ok_or_else(|| ResolutionFailure {
            code: "MS5104",
            message: format!("note target `{authored_path}` escapes the content root"),
            candidates: Vec::new(),
            always_error: true,
        })?;

    let candidates = path_candidates(index, &relative_target);
    if !candidates.is_empty() {
        return choose(index, candidates, &format!("note `{authored_path}`"));
    }

    let root_target = normalize_logical("", authored_path);
    if let Some(root_target) = &root_target {
        let candidates = path_candidates(index, root_target);
        if !candidates.is_empty() {
            return choose(index, candidates, &format!("note `{authored_path}`"));
        }
    }

    let normalized_without_extension =
        strip_markdown_extension(root_target.as_deref().unwrap_or(relative_target.as_str()));
    let candidates = path_candidates(index, &normalized_without_extension);
    if !candidates.is_empty() {
        return choose(index, candidates, &format!("note `{authored_path}`"));
    }

    let stem = normalized_without_extension
        .trim_end_matches("/index")
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let candidates: Vec<_> = index
        .iter()
        .enumerate()
        .filter_map(|(position, page)| (page.stem == stem).then_some(position))
        .collect();
    if !candidates.is_empty() {
        return choose(index, candidates, &format!("note basename `{stem}`"));
    }

    let candidates: Vec<_> = index
        .iter()
        .enumerate()
        .filter_map(|(position, page)| {
            page.aliases
                .iter()
                .any(|alias| {
                    alias.trim() == authored_path
                        || root_target
                            .as_deref()
                            .is_some_and(|target| alias.trim() == target)
                })
                .then_some(position)
        })
        .collect();
    choose(index, candidates, &format!("note `{authored_path}`"))
}

fn path_candidates(index: &[IndexedPage], target: &str) -> Vec<usize> {
    let target = strip_markdown_extension(target);
    let target_note = target.strip_suffix("/index").unwrap_or(&target);
    index
        .iter()
        .enumerate()
        .filter_map(|(position, page)| {
            (page.source_without_extension == target || page.note_path == target_note)
                .then_some(position)
        })
        .collect()
}

fn choose(
    index: &[IndexedPage],
    candidates: Vec<usize>,
    label: &str,
) -> Result<usize, ResolutionFailure> {
    match candidates.as_slice() {
        [candidate] => Ok(*candidate),
        [] => Err(ResolutionFailure {
            code: "MS5101",
            message: format!("could not resolve {label}"),
            candidates,
            always_error: false,
        }),
        _ => Err(ResolutionFailure {
            code: "MS5102",
            message: format!("{label} is ambiguous"),
            candidates: candidates
                .into_iter()
                .filter(|position| *position < index.len())
                .collect(),
            always_error: false,
        }),
    }
}

fn resolve_fragment(
    page: &IndexedPage,
    fragment: Option<&str>,
) -> Result<Option<ResolvedFragment>, ResolutionFailure> {
    let Some(fragment) = fragment.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if let Some(block_id) = fragment.strip_prefix('^') {
        return page
            .blocks
            .iter()
            .find(|block| block.id == block_id)
            .map(|block| {
                Some(ResolvedFragment::Block {
                    id: block.id.clone(),
                })
            })
            .ok_or_else(|| ResolutionFailure {
                code: "MS5105",
                message: format!(
                    "page `{}` has no block identifier `^{block_id}`",
                    page.source_path
                ),
                candidates: Vec::new(),
                always_error: false,
            });
    }

    let normalized = slugify_segment(fragment);
    let heading = page
        .headings
        .iter()
        .find(|heading| heading.id == fragment)
        .or_else(|| {
            page.headings
                .iter()
                .find(|heading| heading.id == normalized)
        });
    heading
        .map(|heading| {
            Some(ResolvedFragment::Heading {
                id: heading.id.clone(),
            })
        })
        .ok_or_else(|| ResolutionFailure {
            code: "MS5103",
            message: format!(
                "page `{}` has no heading matching `{fragment}`",
                page.source_path
            ),
            candidates: Vec::new(),
            always_error: false,
        })
}

fn normalize_logical(base: &str, target: &str) -> Option<String> {
    if target.contains('\\') {
        return None;
    }
    let rooted = target.starts_with('/');
    let mut segments: Vec<String> = if rooted || base.is_empty() {
        Vec::new()
    } else {
        base.split('/').map(str::to_owned).collect()
    };
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            value => segments.push(value.nfc().collect()),
        }
    }
    Some(segments.join("/"))
}

fn strip_markdown_extension(path: &str) -> String {
    path.strip_suffix(".md").unwrap_or(path).to_owned()
}

fn is_non_markdown_file(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension != "md")
}

fn split_fragment(destination: &str) -> (&str, Option<&str>) {
    destination
        .split_once('#')
        .map_or((destination, None), |(path, fragment)| {
            (path, Some(fragment))
        })
}

fn url_scheme(destination: &str) -> Option<&str> {
    let colon = destination.find(':')?;
    let candidate = &destination[..colon];
    let mut characters = candidate.chars();
    if !characters
        .next()
        .is_some_and(|value| value.is_ascii_alphabetic())
        || !characters
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '+' | '-' | '.'))
    {
        return None;
    }
    Some(candidate)
}

fn reject_control_characters(value: &str) -> Result<(), ResolutionFailure> {
    if value.chars().any(char::is_control) {
        Err(ResolutionFailure {
            code: "MS5104",
            message: "link destinations may not contain control characters".to_owned(),
            candidates: Vec::new(),
            always_error: true,
        })
    } else {
        Ok(())
    }
}

fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some(pair) = bytes.get(index + 1..index + 3) else {
                return Err(format!(
                    "invalid percent escape in link destination `{value}`"
                ));
            };
            let high = hex_value(pair[0])
                .ok_or_else(|| format!("invalid percent escape in link destination `{value}`"))?;
            let low = hex_value(pair[1])
                .ok_or_else(|| format!("invalid percent escape in link destination `{value}`"))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded)
        .map_err(|_| format!("link destination `{value}` is not valid percent-encoded UTF-8"))
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;

    use tempfile::tempdir;

    use crate::{Compiler, Config, ResolvedFragment, ResolvedLinkTarget};

    #[test]
    fn resolves_markdown_wikilinks_fragments_aliases_and_backlinks() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(docs.join("guide")).unwrap();
        fs::write(
            docs.join("index.md"),
            concat!(
                "# Home\n\n",
                "[[Guide|Read it]] [Intro](guide/Guide.md#Install) ",
                "[external](https://example.com)\n",
            ),
        )
        .unwrap();
        fs::write(
            docs.join("guide/Guide.md"),
            "---\naliases: [Guide]\n---\n# Guide\n\n## Install\n\n[Home](../index.md)\n",
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert_eq!(result.diagnostics, []);
        let site = result.site.unwrap();
        let home = site.pages.iter().find(|page| page.route == "/").unwrap();
        assert_eq!(home.outgoing_links.len(), 3);
        assert!(matches!(
            &home.outgoing_links[1].target,
            ResolvedLinkTarget::Page {
                fragment: Some(ResolvedFragment::Heading { id }),
                ..
            } if id == "install"
        ));
        assert!(matches!(
            home.outgoing_links[2].target,
            ResolvedLinkTarget::External { .. }
        ));
        let guide = site
            .pages
            .iter()
            .find(|page| page.route == "/guide/guide/")
            .unwrap();
        assert_eq!(guide.backlinks.as_slice(), std::slice::from_ref(&home.id));
        assert_eq!(home.backlinks.as_slice(), std::slice::from_ref(&guide.id));
    }

    #[test]
    fn resolves_allowed_external_schemes_case_insensitively() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            concat!(
                "# Home\n\n",
                "[HTTP](HTTP://example.com) ",
                "[HTTPS](HtTpS://example.com/path) ",
                "[mail](MAILTO:hello@example.com) ",
                "[phone](TeL:+6512345678)\n",
            ),
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert_eq!(result.diagnostics, []);
        let page = &result.site.unwrap().pages[0];
        let hrefs: Vec<_> = page
            .outgoing_links
            .iter()
            .map(|link| match &link.target {
                ResolvedLinkTarget::External { href } => href.as_str(),
                target => panic!("expected external target, got {target:?}"),
            })
            .collect();
        assert_eq!(
            hrefs,
            [
                "HTTP://example.com",
                "HtTpS://example.com/path",
                "MAILTO:hello@example.com",
                "TeL:+6512345678",
            ]
        );
    }

    #[test]
    fn reports_ambiguous_missing_and_unsafe_links() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(docs.join("one")).unwrap();
        fs::create_dir_all(docs.join("two")).unwrap();
        fs::write(
            docs.join("index.md"),
            "# Home\n\n[[Same]] [[Missing]] [bad](javascript:alert(1))\n",
        )
        .unwrap();
        fs::write(docs.join("one/Same.md"), "# One\n").unwrap();
        fs::write(docs.join("two/Same.md"), "# Two\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        let codes: BTreeSet<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert!(codes.contains("MS5101"));
        assert!(codes.contains("MS5102"));
        assert!(codes.contains("MS5104"));
        assert!(result.site.is_none());
    }

    #[test]
    fn non_strict_links_are_retained_as_unresolved_warnings() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home\n\n[[Missing]]\n").unwrap();
        let config = Config::from_toml(
            concat!(
                "schema=1\ncontent_root=\"docs\"\n",
                "[markdown]\nstrict_links=false\n",
            ),
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS5101")
        );
        let page = &result.site.unwrap().pages[0];
        assert!(matches!(
            page.outgoing_links[0].target,
            ResolvedLinkTarget::Unresolved
        ));
    }

    #[test]
    fn resolves_embeds_with_stable_unique_instances_and_detects_cycles() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            "# Home\n\n![[Other]] and ![[Other]]\n",
        )
        .unwrap();
        fs::write(docs.join("Other.md"), "# Other\n\n![[index]]\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS5201")
        );
        assert!(result.site.is_none());

        fs::write(docs.join("Other.md"), "# Other\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();
        let site = Compiler::new(config).compile().site.unwrap();
        let home = site.pages.iter().find(|page| page.route == "/").unwrap();
        assert_eq!(home.embeds.len(), 2);
        assert_ne!(home.embeds[0].instance_id, home.embeds[1].instance_id);
    }

    #[test]
    fn enforces_configured_embed_depth() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home\n\n![[Second]]\n").unwrap();
        fs::write(docs.join("Second.md"), "# Second\n\n![[Third]]\n").unwrap();
        fs::write(docs.join("Third.md"), "# Third\n").unwrap();
        let config = Config::from_toml(
            concat!(
                "schema=1\ncontent_root=\"docs\"\n",
                "[markdown]\nmax_embed_depth=1\n",
            ),
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS5202")
        );
        assert!(result.site.is_none());
    }

    #[test]
    fn enforces_embed_depth_through_a_previously_visited_shared_suffix() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("index.md"), "# Home\n").unwrap();
        fs::write(docs.join("a-suffix.md"), "# Suffix\n\n![[b-tail]]\n").unwrap();
        fs::write(docs.join("b-tail.md"), "# Tail\n").unwrap();
        fs::write(docs.join("z-start.md"), "# Start\n\n![[a-suffix]]\n").unwrap();
        let config = Config::from_toml(
            concat!(
                "schema=1\ncontent_root=\"docs\"\n",
                "[markdown]\nmax_embed_depth=1\n",
            ),
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        let depth = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "MS5202")
            .expect("shared suffix must retain its full memoized depth");
        assert_eq!(depth.notes, ["/z-start/ -> /a-suffix/ -> /b-tail/"]);
        assert!(result.site.is_none());
    }
}
