use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path};

use unicode_normalization::UnicodeNormalization;

use crate::config::normalize_relative;
use crate::{Config, Diagnostic, DirectiveValue, MarkdownNode, NodeKind, Page, SourceSpan};

const ASSET_SOURCE_DIRECTORY: &str = "_assets";

/// One validated asset ready for publication below the managed asset tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledAsset {
    pub output_path: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Default)]
pub(crate) struct AssetOutcome {
    pub assets: Vec<CompiledAsset>,
    pub diagnostics: Vec<Diagnostic>,
}

struct Candidate {
    asset: CompiledAsset,
    source_path: String,
}

pub(crate) fn compile(config: &Config, pages: &mut [Page]) -> AssetOutcome {
    let mut diagnostics = Vec::new();
    let mut candidates = discover(&config.content_root, &mut diagnostics);
    candidates.sort_by(|left, right| left.asset.output_path.cmp(&right.asset.output_path));
    validate_collisions(&candidates, &mut diagnostics);

    let paths: BTreeSet<_> = candidates
        .iter()
        .map(|candidate| candidate.asset.output_path.clone())
        .collect();
    if let Some(public_root) = public_root(config) {
        rewrite_pages(pages, &paths, &public_root, &mut diagnostics);
    } else {
        diagnostics.push(
            Diagnostic::error(
                "MS1014",
                "`assets_out` must be a URL-safe managed subdirectory under `public/`",
            )
            .at_path(
                config
                    .config_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("mambo.toml"),
            ),
        );
    }

    AssetOutcome {
        assets: candidates
            .into_iter()
            .map(|candidate| candidate.asset)
            .collect(),
        diagnostics,
    }
}

fn discover(content_root: &Path, diagnostics: &mut Vec<Diagnostic>) -> Vec<Candidate> {
    let root = content_root.join(ASSET_SOURCE_DIRECTORY);
    match fs::symlink_metadata(&root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            diagnostics.push(asset_tree_error(
                ASSET_SOURCE_DIRECTORY,
                format!("could not inspect the asset directory: {error}"),
            ));
            return Vec::new();
        }
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            diagnostics.push(asset_tree_error(
                ASSET_SOURCE_DIRECTORY,
                "the asset source must be a regular directory",
            ));
            return Vec::new();
        }
        Ok(_) => {}
    }

    let mut candidates = Vec::new();
    walk(&root, &root, &mut candidates, diagnostics);
    candidates
}

fn walk(
    root: &Path,
    directory: &Path,
    candidates: &mut Vec<Candidate>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(asset_tree_error(
                source_label(root, directory),
                format!("could not read the asset directory: {error}"),
            ));
            return;
        }
    };
    let mut entries = entries
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(error) => {
                diagnostics.push(asset_tree_error(
                    source_label(root, directory),
                    format!("could not inspect an asset directory entry: {error}"),
                ));
                None
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let source_path = source_label(root, &path);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                diagnostics.push(asset_tree_error(
                    source_path,
                    format!("could not inspect the asset: {error}"),
                ));
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            diagnostics.push(asset_tree_error(
                source_path,
                "symbolic links are not allowed in the asset directory",
            ));
        } else if metadata.is_dir() {
            walk(root, &path, candidates, diagnostics);
        } else if metadata.is_file() {
            let output_path = match normalized_asset_path(root, &path) {
                Ok(output_path) => output_path,
                Err(message) => {
                    diagnostics.push(asset_tree_error(source_path, message));
                    continue;
                }
            };
            match fs::read(&path) {
                Ok(contents) => candidates.push(Candidate {
                    asset: CompiledAsset {
                        output_path,
                        contents,
                    },
                    source_path,
                }),
                Err(error) => diagnostics.push(asset_tree_error(
                    source_path,
                    format!("could not read the asset: {error}"),
                )),
            }
        } else {
            diagnostics.push(asset_tree_error(
                source_path,
                "only regular files and directories are allowed in the asset directory",
            ));
        }
    }
}

fn normalized_asset_path(root: &Path, path: &Path) -> Result<String, &'static str> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "asset path escapes the asset directory")?;
    let mut segments = Vec::new();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err("asset path contains an unsafe component");
        };
        let Some(segment) = component.to_str() else {
            return Err("asset paths must be valid UTF-8");
        };
        if segment.contains('\\') || segment.chars().any(char::is_control) {
            return Err("asset paths may not contain backslashes or control characters");
        }
        segments.push(segment.nfc().collect::<String>());
    }
    if segments.is_empty() {
        Err("asset path must name a file")
    } else {
        Ok(segments.join("/"))
    }
}

fn validate_collisions(candidates: &[Candidate], diagnostics: &mut Vec<Diagnostic>) {
    for (index, candidate) in candidates.iter().enumerate() {
        let folded = candidate.asset.output_path.to_ascii_lowercase();
        for previous in &candidates[..index] {
            let previous_folded = previous.asset.output_path.to_ascii_lowercase();
            if folded == previous_folded
                || folded.starts_with(&format!("{previous_folded}/"))
                || previous_folded.starts_with(&format!("{folded}/"))
            {
                diagnostics.push(
                    Diagnostic::error(
                        "MS5303",
                        format!(
                            "asset output path `{}` collides with another asset",
                            candidate.asset.output_path
                        ),
                    )
                    .at_path(candidate.source_path.clone())
                    .with_related(previous.source_path.clone(), SourceSpan::point(1, 1)),
                );
            }
        }
    }
}

fn public_root(config: &Config) -> Option<String> {
    let relative = config
        .assets_out
        .strip_prefix(config.project_root.join("public"))
        .ok()?;
    let parts = relative
        .iter()
        .map(|part| part.to_str())
        .collect::<Option<Vec<_>>>()?;
    (!parts.is_empty()).then(|| format!("/{}/assets", parts.join("/")))
}

fn rewrite_pages(
    pages: &mut [Page],
    paths: &BTreeSet<String>,
    public_root: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for page in pages {
        let source_path = page.source_path.clone();
        if let Some(cover) = &mut page.cover {
            rewrite_value(cover, paths, public_root, &source_path, None, diagnostics);
        }
        rewrite_node(
            &mut page.body,
            paths,
            public_root,
            &source_path,
            diagnostics,
        );
        rewrite_validated_directives(&mut page.directives, paths, public_root);
    }
}

fn rewrite_node(
    node: &mut MarkdownNode,
    paths: &BTreeSet<String>,
    public_root: &str,
    source_path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let span = node.span;
    match &mut node.kind {
        NodeKind::Link { destination, .. }
        | NodeKind::WikiLink { destination }
        | NodeKind::ObsidianEmbed { destination, .. } => rewrite_value(
            destination,
            paths,
            public_root,
            source_path,
            span,
            diagnostics,
        ),
        NodeKind::Image { source, .. } => {
            rewrite_value(source, paths, public_root, source_path, span, diagnostics);
        }
        NodeKind::Directive { invocation, .. } => {
            if let Some(DirectiveValue::String(value)) = directive_asset_property(&invocation.name)
                .and_then(|property_name| {
                    invocation
                        .properties
                        .iter_mut()
                        .find(|property| property.name == property_name)
                        .map(|property| &mut property.value)
                })
            {
                rewrite_value(value, paths, public_root, source_path, span, diagnostics);
            }
        }
        _ => {}
    }
    rewrite_children(node, paths, public_root, source_path, diagnostics);
}

fn rewrite_children(
    node: &mut MarkdownNode,
    paths: &BTreeSet<String>,
    public_root: &str,
    source_path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for child in &mut node.children {
        rewrite_node(child, paths, public_root, source_path, diagnostics);
    }
}

fn rewrite_validated_directives(
    directives: &mut [crate::ValidatedDirective],
    paths: &BTreeSet<String>,
    public_root: &str,
) {
    for directive in directives {
        let Some(property_name) = directive_asset_property(&directive.name) else {
            continue;
        };
        let Some(DirectiveValue::String(value)) = directive.properties.get_mut(property_name)
        else {
            continue;
        };
        if let Ok(Some(rewritten)) = asset_href(value, paths, public_root) {
            *value = rewritten;
        }
    }
}

fn directive_asset_property(name: &str) -> Option<&'static str> {
    match name {
        "hero" => Some("image"),
        "button" => Some("href"),
        _ => None,
    }
}

fn rewrite_value(
    value: &mut String,
    paths: &BTreeSet<String>,
    public_root: &str,
    source_path: &str,
    span: Option<SourceSpan>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match asset_href(value, paths, public_root) {
        Ok(Some(rewritten)) => *value = rewritten,
        Ok(None) => {}
        Err(message) => {
            let diagnostic = Diagnostic::error("MS5301", message);
            diagnostics.push(if let Some(span) = span {
                diagnostic.at(source_path, span)
            } else {
                diagnostic.at_path(source_path)
            });
        }
    }
}

fn asset_href(
    authored: &str,
    paths: &BTreeSet<String>,
    public_root: &str,
) -> Result<Option<String>, String> {
    if authored != "assets" && !authored.starts_with("assets/") {
        return Ok(None);
    }
    if authored.contains(['?', '#']) {
        return Err(format!(
            "asset reference `{authored}` may not contain a query string or fragment"
        ));
    }
    let decoded = crate::reference::percent_decode(authored)
        .map_err(|_| format!("asset reference `{authored}` has invalid percent encoding"))?;
    let relative = decoded
        .strip_prefix("assets/")
        .ok_or_else(|| format!("asset reference `{authored}` must name a file below `assets/`"))?;
    if relative.chars().any(char::is_control) || relative.contains('\\') {
        return Err(format!(
            "asset reference `{authored}` contains an unsafe character"
        ));
    }
    let normalized = normalize_relative(relative)
        .map(|path| path.nfc().collect::<String>())
        .ok_or_else(|| format!("asset reference `{authored}` escapes `assets/`"))?;
    if !paths.contains(&normalized) {
        return Err(format!(
            "asset reference `{authored}` does not match a file in `{ASSET_SOURCE_DIRECTORY}/`"
        ));
    }
    Ok(Some(format!(
        "{public_root}/{}",
        percent_encode_path(&normalized)
    )))
}

fn percent_encode_path(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            output.push(char::from(byte));
        } else {
            let _ = write!(output, "%{byte:02X}");
        }
    }
    output
}

fn source_label(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let relative = relative.to_string_lossy().replace('\\', "/");
    if relative.is_empty() {
        ASSET_SOURCE_DIRECTORY.to_owned()
    } else {
        format!("{ASSET_SOURCE_DIRECTORY}/{relative}")
    }
}

fn asset_tree_error(path: impl Into<String>, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error("MS5302", message).at_path(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{Candidate, CompiledAsset, validate_collisions};
    use crate::{Compiler, Config};

    #[test]
    fn compiles_binary_assets_and_rewrites_explicit_references() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(docs.join("_assets/media")).unwrap();
        fs::write(docs.join("_assets/media/Hero image.md"), [0, 159, 146, 150]).unwrap();
        fs::write(
            docs.join("index.md"),
            concat!(
                "---\ncover: assets/media/Hero image.md\n---\n",
                "# Home\n\n",
                "::hero{image=\"assets/media/Hero image.md\"}\n\n",
                "::button{label=\"Download\" href=\"assets/media/Hero%20image.md\"}\n\n",
                "![Preview](assets/media/Hero%20image.md)\n\n",
                "[Download](assets/media/Hero%20image.md)\n\n",
                "![[assets/media/Hero image.md]]\n",
            ),
        )
        .unwrap();
        let config = Config::from_toml(
            concat!(
                "schema=1\ncontent_root=\"docs\"\n",
                "assets_out=\"public/site-assets\"\n",
                "[site]\nbase_path=\"/portfolio\"\n",
            ),
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();

        assert_eq!(result.diagnostics, []);
        assert_eq!(result.assets.len(), 1);
        assert_eq!(result.assets[0].output_path, "media/Hero image.md");
        assert_eq!(result.assets[0].contents, [0, 159, 146, 150]);
        let serialized = serde_json::to_string(&result.site.unwrap().pages[0]).unwrap();
        assert_eq!(
            serialized
                .matches("/site-assets/assets/media/Hero%20image.md")
                .count(),
            8
        );
        assert!(!serialized.contains("/portfolio/site-assets"));
    }

    #[test]
    fn reports_missing_and_escaping_asset_references() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(docs.join("_assets")).unwrap();
        fs::write(
            docs.join("index.md"),
            "# Home\n\n![Missing](assets/missing.png) ![Unsafe](assets/../secret.png)\n",
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"\n",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();

        assert!(result.site.is_none());
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "MS5301")
                .count(),
            2
        );
    }

    #[test]
    fn rejects_normalized_asset_path_collisions() {
        let candidates = [
            Candidate {
                asset: CompiledAsset {
                    output_path: "caf\u{e9}.png".to_owned(),
                    contents: Vec::new(),
                },
                source_path: "_assets/caf\u{e9}.png".to_owned(),
            },
            Candidate {
                asset: CompiledAsset {
                    output_path: "caf\u{e9}.png".to_owned(),
                    contents: Vec::new(),
                },
                source_path: "_assets/cafe\u{301}.png".to_owned(),
            },
        ];
        let mut diagnostics = Vec::new();

        validate_collisions(&candidates, &mut diagnostics);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "MS5303");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_asset_directory() {
        use std::os::unix::fs::symlink;

        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        let external = temporary.path().join("external");
        fs::create_dir_all(&docs).unwrap();
        fs::create_dir_all(&external).unwrap();
        fs::write(docs.join("index.md"), "# Home\n").unwrap();
        symlink(&external, docs.join("_assets")).unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"\n",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();

        assert!(result.site.is_none());
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS5302")
        );
    }
}
