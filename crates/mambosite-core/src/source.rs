use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::normalize_relative;
use crate::{Config, Diagnostic, Mount, normalize_route};

#[derive(Debug, Clone)]
pub(crate) struct DiscoveredSource {
    pub absolute_path: PathBuf,
    pub logical_path: String,
    /// Path used for route derivation, relative to the physical or mount root.
    pub route_source: String,
    pub mount_prefix: Option<String>,
    pub is_entry: bool,
}

#[derive(Debug, Default)]
pub(crate) struct DiscoveryOutcome {
    pub sources: Vec<DiscoveredSource>,
    pub diagnostics: Vec<Diagnostic>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn discover(config: &Config, mounts: &[Mount]) -> DiscoveryOutcome {
    let mut outcome = DiscoveryOutcome::default();
    let root = &config.content_root;
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            outcome.diagnostics.push(
                Diagnostic::error("MS2103", "the content root may not be a symlink")
                    .at_path("<content-root>"),
            );
            return outcome;
        }
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            outcome.diagnostics.push(
                Diagnostic::error("MS2101", "the configured content root is not a directory")
                    .at_path("<content-root>"),
            );
            return outcome;
        }
        Err(error) => {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS2101",
                    format!("could not open the configured content root: {error}"),
                )
                .at_path("<content-root>"),
            );
            return outcome;
        }
    }

    walk_directory(root, root, root, None, &config.entry, true, &mut outcome);

    let entry_path = root.join(&config.entry);
    if !entry_path.is_file() {
        outcome.diagnostics.push(
            Diagnostic::error(
                "MS2102",
                format!("configured entry `{}` does not exist", config.entry),
            )
            .at_path(config.entry.clone()),
        );
    } else if fs::symlink_metadata(&entry_path)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        && !outcome.sources.iter().any(|source| source.is_entry)
    {
        outcome.sources.push(DiscoveredSource {
            absolute_path: entry_path,
            logical_path: config.entry.clone(),
            route_source: config.entry.clone(),
            mount_prefix: None,
            is_entry: true,
        });
    }

    let Ok(canonical_root) = root.canonicalize() else {
        return finish(outcome);
    };
    let mut mounted_sources = BTreeSet::new();
    let mut mount_routes: Vec<(String, String)> = Vec::new();
    for mount in mounts {
        let route = match normalize_route(&mount.path, config.site.trailing_slash) {
            Ok(route) if route != "/" => route,
            Ok(_) => {
                outcome.diagnostics.push(
                    Diagnostic::error("MS4201", "a mount path cannot be the site root")
                        .at_path(config.entry.clone()),
                );
                continue;
            }
            Err(message) => {
                outcome.diagnostics.push(
                    Diagnostic::error(
                        "MS4201",
                        format!("invalid mount path `{}`: {message}", mount.path),
                    )
                    .at_path(config.entry.clone()),
                );
                continue;
            }
        };
        let Some(normalized_source) = normalize_relative(&mount.source) else {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS4202",
                    format!(
                        "mount source `{}` must be a content-root-relative index.md",
                        mount.source
                    ),
                )
                .at_path(config.entry.clone()),
            );
            continue;
        };
        if Path::new(&normalized_source)
            .file_name()
            .and_then(|value| value.to_str())
            != Some("index.md")
        {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS4202",
                    format!(
                        "mount source `{}` must be a content-root-relative index.md",
                        mount.source
                    ),
                )
                .at_path(config.entry.clone()),
            );
            continue;
        }
        if !mounted_sources.insert(normalized_source.clone()) {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS4203",
                    format!("mount source `{}` is declared more than once", mount.source),
                )
                .at_path(config.entry.clone()),
            );
            continue;
        }
        if let Some((other_route, other_source)) = mount_routes
            .iter()
            .find(|(other, _)| route_prefixes_overlap(&route, other))
        {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS4205",
                    format!("mount route `{route}` overlaps `{other_route}`"),
                )
                .at_path(config.entry.clone())
                .with_note(format!("the overlapping route mounts `{other_source}`")),
            );
            continue;
        }

        let source_path = root.join(&normalized_source);
        if contains_symlink(root, Path::new(&normalized_source)) {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS2103",
                    format!("mount source `{}` crosses a symlink", mount.source),
                )
                .at_path(mount.source.clone()),
            );
            continue;
        }
        let canonical_source = match source_path.canonicalize() {
            Ok(source) if source.starts_with(&canonical_root) => source,
            Ok(_) => {
                outcome.diagnostics.push(
                    Diagnostic::error(
                        "MS4204",
                        format!("mount source `{}` escapes the content root", mount.source),
                    )
                    .at_path(config.entry.clone()),
                );
                continue;
            }
            Err(error) => {
                outcome.diagnostics.push(
                    Diagnostic::error(
                        "MS4204",
                        format!("could not open mount source `{}`: {error}", mount.source),
                    )
                    .at_path(config.entry.clone()),
                );
                continue;
            }
        };
        if !canonical_source.is_file()
            || canonical_source.file_name().and_then(|name| name.to_str()) != Some("index.md")
        {
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS4202",
                    format!("mount source `{}` is not an index.md file", mount.source),
                )
                .at_path(config.entry.clone()),
            );
            continue;
        }

        let Some(mount_root) = source_path.parent() else {
            continue;
        };
        walk_directory(
            root,
            mount_root,
            mount_root,
            Some(&route),
            &config.entry,
            false,
            &mut outcome,
        );
        mount_routes.push((route, mount.source.clone()));
    }

    finish(outcome)
}

fn finish(mut outcome: DiscoveryOutcome) -> DiscoveryOutcome {
    outcome.sources.sort_by(|left, right| {
        left.mount_prefix
            .cmp(&right.mount_prefix)
            .then_with(|| left.logical_path.cmp(&right.logical_path))
    });
    outcome
}

fn walk_directory(
    content_root: &Path,
    directory: &Path,
    route_root: &Path,
    mount_prefix: Option<&str>,
    entry: &str,
    ordinary: bool,
    outcome: &mut DiscoveryOutcome,
) {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            let label = logical_path(content_root, directory)
                .unwrap_or_else(|| "<content-root>".to_owned());
            outcome.diagnostics.push(
                Diagnostic::error(
                    "MS2104",
                    format!("could not read source directory: {error}"),
                )
                .at_path(label),
            );
            return;
        }
    };
    let mut entries: Vec<_> = entries.collect();
    entries.sort_by(|left, right| {
        let left = left.as_ref().ok().map(fs::DirEntry::file_name);
        let right = right.as_ref().ok().map(fs::DirEntry::file_name);
        left.cmp(&right)
    });

    for entry_result in entries {
        let entry_item = match entry_result {
            Ok(entry) => entry,
            Err(error) => {
                outcome.diagnostics.push(Diagnostic::error(
                    "MS2104",
                    format!("could not inspect a source entry: {error}"),
                ));
                continue;
            }
        };
        let name = entry_item.file_name();
        let Some(name) = name.to_str() else {
            outcome.diagnostics.push(
                Diagnostic::error("MS2105", "source paths must be valid UTF-8")
                    .at_path("<content-root>"),
            );
            continue;
        };
        let path = entry_item.path();
        let file_type = match entry_item.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                outcome.diagnostics.push(
                    Diagnostic::error(
                        "MS2104",
                        format!("could not inspect source `{name}`: {error}"),
                    )
                    .at_path(logical_path(content_root, &path).unwrap_or_else(|| name.to_owned())),
                );
                continue;
            }
        };
        if file_type.is_symlink() {
            outcome.diagnostics.push(
                Diagnostic::error("MS2103", "symlinks are not allowed in the content tree")
                    .at_path(logical_path(content_root, &path).unwrap_or_else(|| name.to_owned())),
            );
            continue;
        }
        if file_type.is_dir() {
            if is_reserved_segment(name) {
                continue;
            }
            walk_directory(
                content_root,
                &path,
                route_root,
                mount_prefix,
                entry,
                ordinary,
                outcome,
            );
            continue;
        }
        if !file_type.is_file() || !is_publishable_markdown(name) {
            continue;
        }

        let Some(logical) = logical_path(content_root, &path) else {
            continue;
        };
        let route_source = if mount_prefix.is_some() {
            logical_path(route_root, &path).unwrap_or_else(|| name.to_owned())
        } else {
            logical.clone()
        };
        outcome.sources.push(DiscoveredSource {
            absolute_path: path,
            logical_path: logical.clone(),
            route_source,
            mount_prefix: mount_prefix.map(str::to_owned),
            is_entry: ordinary && logical == entry,
        });
    }
}

fn is_reserved_segment(name: &str) -> bool {
    name.starts_with('.') || name.starts_with('_') || name == "archive"
}

fn is_publishable_markdown(name: &str) -> bool {
    Path::new(name).extension().and_then(|value| value.to_str()) == Some("md")
        && !name.starts_with('.')
        && !name.starts_with('_')
        && name != "README.md"
}

fn logical_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let mut segments = Vec::new();
    for component in relative.components() {
        segments.push(component.as_os_str().to_str()?);
    }
    Some(segments.join("/"))
}

fn contains_symlink(root: &Path, relative: &Path) -> bool {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        if fs::symlink_metadata(&current).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return true;
        }
    }
    false
}

fn route_prefixes_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_end_matches('/');
    let right = right.trim_end_matches('/');
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::discover;
    use crate::Config;

    #[test]
    fn excludes_reserved_files_and_discovers_explicit_mounts() {
        let temp = tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(docs.join("_mounts/project")).unwrap();
        fs::write(
            docs.join("index.md"),
            "---\nmounts:\n  - path: /project\n    source: _mounts/project/index.md\n---\n",
        )
        .unwrap();
        fs::write(docs.join("page.md"), "page").unwrap();
        fs::write(docs.join("README.md"), "readme").unwrap();
        fs::write(docs.join("_mounts/project/index.md"), "project").unwrap();
        fs::write(docs.join("_mounts/project/Guide.md"), "guide").unwrap();

        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temp.path().join("mambo.toml"),
        )
        .unwrap();
        let entry = fs::read_to_string(docs.join("index.md")).unwrap();
        let parsed = crate::parse_frontmatter(&entry, "index.md", &config.frontmatter);
        let discovered = discover(&config, &parsed.metadata.mounts);
        let paths: Vec<_> = discovered
            .sources
            .iter()
            .map(|source| (source.logical_path.as_str(), source.mount_prefix.as_deref()))
            .collect();
        assert!(paths.contains(&("index.md", None)));
        assert!(paths.contains(&("page.md", None)));
        assert!(paths.contains(&("_mounts/project/index.md", Some("/project/"))));
        assert!(paths.contains(&("_mounts/project/Guide.md", Some("/project/"))));
        assert!(!paths.iter().any(|(path, _)| *path == "README.md"));
        assert!(discovered.diagnostics.is_empty());
    }

    #[test]
    fn deduplicates_normalized_mount_sources() {
        let temp = tempdir().unwrap();
        let docs = temp.path().join("docs");
        fs::create_dir_all(docs.join("_mounts/project")).unwrap();
        fs::write(docs.join("index.md"), "# Home\n").unwrap();
        fs::write(docs.join("_mounts/project/index.md"), "# Project\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temp.path().join("mambo.toml"),
        )
        .unwrap();
        let mounts = [
            crate::Mount {
                path: "/one".into(),
                source: "_mounts/project/index.md".into(),
            },
            crate::Mount {
                path: "/two".into(),
                source: "_mounts/project/./index.md".into(),
            },
        ];

        let discovered = discover(&config, &mounts);
        assert!(
            discovered
                .diagnostics
                .iter()
                .any(|item| item.code == "MS4203")
        );
    }
}
