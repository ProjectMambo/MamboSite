use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::{Diagnostic, SourceSpan};

#[derive(Debug, Clone)]
pub struct Config {
    pub schema: u32,
    pub config_path: PathBuf,
    pub project_root: PathBuf,
    pub content_root: PathBuf,
    pub entry: String,
    pub typescript_out: PathBuf,
    pub assets_out: PathBuf,
    pub site: SiteConfig,
    pub markdown: MarkdownConfig,
    pub frontmatter: FrontmatterConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SiteConfig {
    pub title: Option<String>,
    pub url: Option<String>,
    pub base_path: String,
    pub trailing_slash: bool,
    pub language: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: None,
            url: None,
            base_path: String::new(),
            trailing_slash: true,
            language: "en".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MarkdownConfig {
    /// Kept for the renderer policy. The parser always preserves raw HTML nodes.
    pub raw_html: bool,
    pub strict_links: bool,
    pub max_embed_depth: usize,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            raw_html: false,
            strict_links: true,
            max_embed_depth: 16,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FrontmatterConfig {
    /// Unknown top-level values become errors when enabled. They are always
    /// retained in `extra` so a diagnostic never destroys source data.
    pub strict: bool,
    pub ignored_fields: Vec<String>,
    /// Migration-only fields promoted into `data` for renderer compatibility.
    pub legacy_data_fields: Vec<String>,
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        Self {
            strict: false,
            ignored_fields: vec![
                "created".to_owned(),
                "project".to_owned(),
                "categories".to_owned(),
            ],
            legacy_data_fields: vec![
                "period".to_owned(),
                "wikiUrl".to_owned(),
                "githubUrl".to_owned(),
            ],
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawConfig {
    schema: u32,
    content_root: String,
    entry: String,
    typescript_out: String,
    assets_out: String,
    site: SiteConfig,
    markdown: MarkdownConfig,
    frontmatter: FrontmatterConfig,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            schema: 1,
            content_root: "docs".to_owned(),
            entry: "index.md".to_owned(),
            typescript_out: "src/generated/mambo".to_owned(),
            assets_out: "public/mambo".to_owned(),
            site: SiteConfig::default(),
            markdown: MarkdownConfig::default(),
            frontmatter: FrontmatterConfig::default(),
        }
    }
}

impl Config {
    /// Load and validate a site configuration.
    ///
    /// # Errors
    ///
    /// Returns structured diagnostics when the file cannot be read, TOML is
    /// invalid, or a configuration invariant fails.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Vec<Diagnostic>> {
        let path = path.as_ref();
        let label = config_label(path);
        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(vec![
                Diagnostic::error("MS1009", "the configuration file may not be a symlink")
                    .at_path(label),
            ]);
        }
        let canonical_path = path.canonicalize().map_err(|error| {
            vec![
                Diagnostic::error("MS1001", format!("could not locate configuration: {error}"))
                    .at_path(label.clone()),
            ]
        })?;
        let source = fs::read_to_string(&canonical_path).map_err(|error| {
            vec![
                Diagnostic::error("MS1001", format!("could not read configuration: {error}"))
                    .at_path(label.clone()),
            ]
        })?;
        Self::from_toml(&source, canonical_path)
    }

    /// Parse configuration text as if it were stored at `path`.
    ///
    /// # Errors
    ///
    /// Returns structured diagnostics for invalid TOML and configuration
    /// invariants.
    #[allow(clippy::too_many_lines)]
    pub fn from_toml(source: &str, path: impl AsRef<Path>) -> Result<Self, Vec<Diagnostic>> {
        let path = path.as_ref();
        let label = config_label(path);
        let raw: RawConfig = toml::from_str(source).map_err(|error| {
            vec![
                Diagnostic::error("MS1002", format!("invalid configuration: {error}"))
                    .at_path(label.clone()),
            ]
        })?;

        let mut diagnostics = Vec::new();
        if raw.schema != 1 {
            diagnostics.push(
                Diagnostic::error(
                    "MS1003",
                    format!("unsupported configuration schema {}", raw.schema),
                )
                .at(label.clone(), SourceSpan::point(1, 1))
                .with_help("set `schema = 1`"),
            );
        }

        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let normalized_content_root =
            normalize_or_report(&raw.content_root, "content_root", &label, &mut diagnostics);
        let normalized_typescript_out = normalize_or_report(
            &raw.typescript_out,
            "typescript_out",
            &label,
            &mut diagnostics,
        );
        let normalized_assets_out =
            normalize_or_report(&raw.assets_out, "assets_out", &label, &mut diagnostics);
        let normalized_entry = normalize_or_report(&raw.entry, "entry", &label, &mut diagnostics);
        let content_root = resolve_relative_path(base, &normalized_content_root);
        let typescript_out = resolve_relative_path(base, &normalized_typescript_out);
        let assets_out = resolve_relative_path(base, &normalized_assets_out);

        if matches!(raw.typescript_out.as_str(), "." | "./")
            || matches!(raw.assets_out.as_str(), "." | "./")
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1008",
                    "generated output directories cannot be the repository/configuration root",
                )
                .at_path(label.clone()),
            );
        }

        if Path::new(&normalized_entry)
            .file_name()
            .and_then(|value| value.to_str())
            != Some("index.md")
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1004",
                    "`entry` must be a relative path to an index.md file",
                )
                .at_path(label.clone()),
            );
        }
        if !raw.site.base_path.is_empty()
            && (!raw.site.base_path.starts_with('/')
                || raw.site.base_path.ends_with('/')
                || raw.site.base_path.contains(['?', '#']))
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1005",
                    "`site.base_path` must be empty or begin with one slash and have no trailing slash",
                )
                .at_path(label.clone()),
            );
        }
        if raw.site.language.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error("MS1006", "`site.language` cannot be empty")
                    .at_path(label.clone()),
            );
        }
        if raw.markdown.max_embed_depth == 0 {
            diagnostics.push(
                Diagnostic::error(
                    "MS1007",
                    "`markdown.max_embed_depth` must be greater than zero",
                )
                .at_path(label.clone()),
            );
        }
        if typescript_out.starts_with(&content_root)
            || assets_out.starts_with(&content_root)
            || typescript_out == assets_out
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1008",
                    "generated output directories must not be inside the content root",
                )
                .at_path(label),
            );
        }

        if diagnostics.iter().any(Diagnostic::is_error) {
            return Err(diagnostics);
        }

        Ok(Self {
            schema: raw.schema,
            config_path: path.to_path_buf(),
            project_root: base.to_path_buf(),
            content_root,
            entry: normalized_entry,
            typescript_out,
            assets_out,
            site: raw.site,
            markdown: raw.markdown,
            frontmatter: raw.frontmatter,
        })
    }

    pub(crate) fn filesystem_diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let Ok(canonical_root) = self.project_root.canonicalize() else {
            diagnostics.push(
                Diagnostic::error("MS1009", "could not resolve the configuration directory")
                    .at_path(config_label(&self.config_path)),
            );
            return diagnostics;
        };
        for (path, field) in [
            (&self.content_root, "content_root"),
            (&self.typescript_out, "typescript_out"),
            (&self.assets_out, "assets_out"),
            (&self.content_root.join(&self.entry), "entry"),
        ] {
            if !path.starts_with(&self.project_root)
                || path_crosses_symlink(&self.project_root, path)
                || existing_path_escapes(path, &canonical_root)
            {
                diagnostics.push(
                    Diagnostic::error(
                        "MS1009",
                        format!(
                            "`{field}` crosses a symlink or escapes the configuration directory"
                        ),
                    )
                    .at_path(config_label(&self.config_path)),
                );
            }
        }
        diagnostics
    }
}

fn config_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mambo.toml")
        .to_owned()
}

fn normalize_or_report(
    value: &str,
    field: &str,
    label: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    if let Some(normalized) = normalize_relative(value) {
        normalized
    } else {
        diagnostics.push(
            Diagnostic::error(
                "MS1004",
                format!("`{field}` must be a non-escaping relative path"),
            )
            .at_path(label),
        );
        value.replace('\\', "/")
    }
}

fn resolve_relative_path(base: &Path, value: &str) -> PathBuf {
    base.join(value)
}

pub(crate) fn normalize_relative(value: &str) -> Option<String> {
    if value.is_empty() || value.contains('\\') {
        return None;
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return None;
    }
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => segments.push(segment.to_str()?.to_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    (!segments.is_empty()).then(|| segments.join("/"))
}

fn path_crosses_symlink(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return true;
    };
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => return true,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(_) => return true,
        }
    }
    false
}

fn existing_path_escapes(path: &Path, canonical_root: &Path) -> bool {
    let mut candidate = path;
    loop {
        if candidate.exists() {
            return candidate
                .canonicalize()
                .map_or(true, |resolved| !resolved.starts_with(canonical_root));
        }
        let Some(parent) = candidate.parent() else {
            return true;
        };
        candidate = parent;
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn resolves_paths_relative_to_the_config() {
        let config =
            Config::from_toml("schema = 1\ncontent_root = \"content\"", "/repo/mambo.toml")
                .expect("valid config");
        assert_eq!(config.content_root.to_string_lossy(), "/repo/content");
        assert_eq!(config.entry, "index.md");
    }

    #[test]
    fn rejects_escaping_paths() {
        let diagnostics = Config::from_toml(
            "schema = 1\ncontent_root = \"../private\"",
            "/repo/mambo.toml",
        )
        .expect_err("invalid config");
        assert!(diagnostics.iter().any(|item| item.code == "MS1004"));
    }

    #[test]
    fn normalizes_entry_and_requires_exact_index_name() {
        let config = Config::from_toml("schema = 1\nentry = \"./index.md\"", "/repo/mambo.toml")
            .expect("normalized config");
        assert_eq!(config.entry, "index.md");

        let diagnostics =
            Config::from_toml("schema = 1\nentry = \"notindex.md\"", "/repo/mambo.toml")
                .expect_err("invalid entry");
        assert!(diagnostics.iter().any(|item| item.code == "MS1004"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_content_and_output_components() {
        use std::fs;
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let project = temporary.path().join("project");
        let external = temporary.path().join("external");
        fs::create_dir_all(external.join("docs")).unwrap();
        fs::create_dir_all(&project).unwrap();
        fs::write(external.join("docs/index.md"), "# Outside\n").unwrap();
        symlink(&external, project.join("linked")).unwrap();
        let config = Config::from_toml(
            concat!(
                "schema=1\ncontent_root=\"linked/docs\"\n",
                "typescript_out=\"linked/generated\"\nassets_out=\"assets\"\n",
            ),
            project.join("mambo.toml"),
        )
        .unwrap();

        let diagnostics = config.filesystem_diagnostics();
        assert!(diagnostics.iter().any(|item| item.code == "MS1009"));
    }
}
