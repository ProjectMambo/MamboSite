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
    pub renderer: Option<RendererConfig>,
    pub deploy: DeployConfig,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RendererKind {
    Next,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub kind: RendererKind,
    pub package_manager: PackageManager,
    pub build_script: String,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DeployConfig {
    pub remote: String,
    pub branch: String,
    pub workflow: String,
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
    renderer: Option<RawRendererConfig>,
    deploy: RawDeployConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawRendererConfig {
    enabled: bool,
    kind: RendererKind,
    package_manager: PackageManager,
    build_script: String,
    output_dir: String,
}

impl Default for RawRendererConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            kind: RendererKind::Next,
            package_manager: PackageManager::Npm,
            build_script: "mambosite:render".to_owned(),
            output_dir: "out".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawDeployConfig {
    remote: String,
    branch: String,
    workflow: String,
}

impl Default for RawDeployConfig {
    fn default() -> Self {
        Self {
            remote: "origin".to_owned(),
            branch: "main".to_owned(),
            workflow: ".github/workflows/pages.yml".to_owned(),
        }
    }
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
            renderer: None,
            deploy: RawDeployConfig::default(),
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
        let normalized_renderer_out = raw
            .renderer
            .as_ref()
            .filter(|renderer| renderer.enabled)
            .map(|renderer| {
                normalize_or_report(
                    &renderer.output_dir,
                    "renderer.output_dir",
                    &label,
                    &mut diagnostics,
                )
            });
        let normalized_workflow = normalize_or_report(
            &raw.deploy.workflow,
            "deploy.workflow",
            &label,
            &mut diagnostics,
        );
        let normalized_entry = normalize_or_report(&raw.entry, "entry", &label, &mut diagnostics);
        let content_root = resolve_relative_path(base, &normalized_content_root);
        let typescript_out = resolve_relative_path(base, &normalized_typescript_out);
        let assets_out = resolve_relative_path(base, &normalized_assets_out);
        let renderer_output = normalized_renderer_out
            .as_ref()
            .map(|output| resolve_relative_path(base, output));

        if matches!(raw.typescript_out.as_str(), "." | "./")
            || matches!(raw.assets_out.as_str(), "." | "./")
            || raw.renderer.as_ref().is_some_and(|renderer| {
                renderer.enabled && matches!(renderer.output_dir.as_str(), "." | "./")
            })
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1008",
                    "generated output directories cannot be the repository/configuration root",
                )
                .at_path(label.clone()),
            );
        }
        let assets_relative = Path::new(&normalized_assets_out);
        if !assets_relative.starts_with("public")
            || assets_relative == Path::new("public")
            || !normalized_assets_out.split('/').all(valid_url_path_segment)
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1014",
                    "`assets_out` must be a URL-safe managed subdirectory under `public/`",
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
        if !valid_base_path(&raw.site.base_path) {
            diagnostics.push(
                Diagnostic::error(
                    "MS1005",
                    "`site.base_path` must be empty or a canonical URL path with one leading slash",
                )
                .at_path(label.clone()),
            );
        }
        if raw
            .site
            .url
            .as_deref()
            .is_some_and(|url| !valid_site_url(url))
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1013",
                    "`site.url` must be an absolute `http://` or `https://` URL with a valid host",
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
        if let Some(renderer) = raw.renderer.as_ref().filter(|renderer| renderer.enabled) {
            if !valid_script_name(&renderer.build_script) {
                diagnostics.push(
                    Diagnostic::error(
                        "MS1010",
                        "`renderer.build_script` must be a non-lifecycle package script name containing only letters, numbers, `.`, `_`, `-`, or `:`",
                    )
                    .at_path(label.clone()),
                );
            }
        }
        if !valid_remote_name(&raw.deploy.remote) {
            diagnostics.push(
                Diagnostic::error("MS1011", "`deploy.remote` must be a simple Git remote name")
                    .at_path(label.clone()),
            );
        }
        if !valid_branch_name(&raw.deploy.branch) {
            diagnostics.push(
                Diagnostic::error("MS1011", "`deploy.branch` must be a simple Git branch name")
                    .at_path(label.clone()),
            );
        }
        let workflow_path = Path::new(&normalized_workflow);
        let workflow_extension = workflow_path.extension().and_then(|value| value.to_str());
        if workflow_path.parent() != Some(Path::new(".github/workflows"))
            || !matches!(workflow_extension, Some("yml" | "yaml"))
        {
            diagnostics.push(
                Diagnostic::error(
                    "MS1012",
                    "`deploy.workflow` must be a `.yml` or `.yaml` file under `.github/workflows/`",
                )
                .at_path(label.clone()),
            );
        }
        let mut managed_directories = vec![
            content_root.as_path(),
            typescript_out.as_path(),
            assets_out.as_path(),
        ];
        managed_directories.extend(renderer_output.as_deref());
        if managed_directories.iter().enumerate().any(|(index, path)| {
            managed_directories[index + 1..]
                .iter()
                .any(|other| path.starts_with(other) || other.starts_with(path))
        }) {
            diagnostics.push(
                Diagnostic::error(
                    "MS1008",
                    "content and generated output directories must not overlap",
                )
                .at_path(label),
            );
        }

        if diagnostics.iter().any(Diagnostic::is_error) {
            return Err(diagnostics);
        }

        let renderer = raw
            .renderer
            .filter(|renderer| renderer.enabled)
            .zip(renderer_output)
            .map(|(renderer, output_dir)| RendererConfig {
                kind: renderer.kind,
                package_manager: renderer.package_manager,
                build_script: renderer.build_script,
                output_dir,
            });

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
            renderer,
            deploy: DeployConfig {
                remote: raw.deploy.remote,
                branch: raw.deploy.branch,
                workflow: normalized_workflow,
            },
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
        let mut checked_paths = vec![
            (self.content_root.clone(), "content_root"),
            (self.typescript_out.clone(), "typescript_out"),
            (self.assets_out.clone(), "assets_out"),
            (
                self.project_root.join(&self.deploy.workflow),
                "deploy.workflow",
            ),
            (self.content_root.join(&self.entry), "entry"),
        ];
        if let Some(renderer) = &self.renderer {
            checked_paths.push((renderer.output_dir.clone(), "renderer.output_dir"));
        }
        for (path, field) in checked_paths {
            if !path.starts_with(&self.project_root)
                || path_crosses_symlink(&self.project_root, &path)
                || existing_path_escapes(&path, &canonical_root)
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

fn valid_script_name(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !matches!(
            value.to_ascii_lowercase().as_str(),
            "build" | "deploy" | "mambosite:build" | "mambosite:deploy"
        )
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._-:".contains(character))
}

fn valid_base_path(value: &str) -> bool {
    value.is_empty()
        || value.strip_prefix('/').is_some_and(|path| {
            !path.is_empty() && !path.ends_with('/') && path.split('/').all(valid_url_path_segment)
        })
}

fn valid_url_path_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !matches!(segment, "." | "..")
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
}

fn valid_site_url(value: &str) -> bool {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|character| {
            character.is_whitespace() || character.is_control() || character == '\\'
        })
    {
        return false;
    }
    let Some((scheme, remainder)) = value.split_once("://") else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return false;
    }
    let authority = remainder.split(['/', '?', '#']).next().unwrap_or_default();
    let host_and_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host_and_port)| host_and_port);
    valid_host_and_port(host_and_port)
}

fn valid_host_and_port(value: &str) -> bool {
    if let Some(bracketed) = value.strip_prefix('[') {
        let Some((host, suffix)) = bracketed.split_once(']') else {
            return false;
        };
        return host.parse::<std::net::Ipv6Addr>().is_ok() && valid_port_suffix(suffix);
    }

    let (host, port) = value
        .rsplit_once(':')
        .map_or((value, None), |(host, port)| (host, Some(port)));
    if host.contains(':') || port.is_some_and(|port| port.parse::<u16>().is_err()) {
        return false;
    }
    let host = host.strip_suffix('.').unwrap_or(host);
    !host.is_empty()
        && host.split('.').all(|label| {
            !label.is_empty()
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|character| character.is_alphanumeric() || character == '-')
        })
}

fn valid_port_suffix(value: &str) -> bool {
    value.is_empty()
        || value
            .strip_prefix(':')
            .is_some_and(|port| port.parse::<u16>().is_ok())
}

fn valid_remote_name(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value.chars().next(), Some('-' | '.'))
        && !value.ends_with('.')
        && !value.contains("..")
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

fn valid_branch_name(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value.chars().next(), Some('-' | '.' | '/'))
        && !value.ends_with('.')
        && !value.ends_with('/')
        && !value.split('/').any(|segment| {
            Path::new(segment)
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("lock"))
        })
        && !value.contains("..")
        && !value.contains("//")
        && !value.contains("@{")
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
        })
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
    use super::{Config, PackageManager, RendererKind};

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
    fn requires_a_managed_asset_subdirectory_under_public() {
        for assets in [
            "assets",
            ".mambosite/assets",
            "public",
            "public/site?draft",
            "public/site#fragment",
            "public/site%2Fescape",
            "public/site\ncontrol",
        ] {
            let source = format!("schema=1\nassets_out={assets:?}\n");
            let diagnostics =
                Config::from_toml(&source, "/repo/mambo.toml").expect_err("invalid assets path");
            assert!(diagnostics.iter().any(|item| item.code == "MS1014"));
        }

        let config = Config::from_toml(
            "schema=1\nassets_out=\"public/site-assets\"\n",
            "/repo/mambo.toml",
        )
        .expect("public asset subdirectory");
        assert_eq!(
            config.assets_out.to_string_lossy(),
            "/repo/public/site-assets"
        );
    }

    #[test]
    fn requires_a_canonical_site_base_path() {
        for base_path in ["", "/MamboFolio", "/docs/v1_0", "/a~b.c"] {
            let source = format!("schema=1\n[site]\nbase_path={base_path:?}\n");
            let config = Config::from_toml(&source, "/repo/mambo.toml")
                .unwrap_or_else(|_| panic!("valid base path: {base_path:?}"));
            assert_eq!(config.site.base_path, base_path);
        }

        for base_path in [
            "/",
            "repo",
            "//repo",
            "/repo/",
            "/repo//docs",
            "/./repo",
            "/repo/../docs",
            "/repo\\docs",
            "/repo?draft",
            "/repo#fragment",
            "/%2e",
            "/repo%2Fdocs",
            "/repo%5Cdocs",
            "/repo\ncontrol",
            "/café",
        ] {
            let source = format!("schema=1\n[site]\nbase_path={base_path:?}\n");
            let diagnostics = Config::from_toml(&source, "/repo/mambo.toml")
                .expect_err("non-canonical base path");
            assert!(
                diagnostics.iter().any(|item| item.code == "MS1005"),
                "expected MS1005 for {base_path:?}"
            );
        }
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

    #[test]
    fn loads_typed_renderer_and_deploy_settings() {
        let config = Config::from_toml(
            concat!(
                "schema = 1\n",
                "[renderer]\n",
                "kind = \"next\"\n",
                "package_manager = \"pnpm\"\n",
                "build_script = \"site:render\"\n",
                "output_dir = \"dist/site\"\n",
                "[deploy]\n",
                "remote = \"upstream\"\n",
                "branch = \"pages/main\"\n",
                "workflow = \".github/workflows/deploy.yaml\"\n",
            ),
            "/repo/mambo.toml",
        )
        .expect("valid lifecycle configuration");

        let renderer = config.renderer.expect("renderer is enabled");
        assert_eq!(renderer.kind, RendererKind::Next);
        assert_eq!(renderer.package_manager, PackageManager::Pnpm);
        assert_eq!(renderer.build_script, "site:render");
        assert_eq!(renderer.output_dir.to_string_lossy(), "/repo/dist/site");
        assert_eq!(config.deploy.remote, "upstream");
        assert_eq!(config.deploy.branch, "pages/main");
        assert_eq!(config.deploy.workflow, ".github/workflows/deploy.yaml");
    }

    #[test]
    fn rejects_unsafe_lifecycle_settings() {
        for source in [
            "schema=1\n[renderer]\nbuild_script=\"build && publish\"\n",
            "schema=1\n[renderer]\nbuild_script=\"build\"\n",
            "schema=1\n[renderer]\nbuild_script=\"mambosite:deploy\"\n",
            "schema=1\n[renderer]\noutput_dir=\"../outside\"\n",
            "schema=1\n[deploy]\nremote=\"--upload-pack\"\n",
            "schema=1\n[deploy]\nbranch=\"main..backup\"\n",
            "schema=1\n[deploy]\nworkflow=\"scripts/deploy.yml\"\n",
        ] {
            assert!(
                Config::from_toml(source, "/repo/mambo.toml").is_err(),
                "configuration should be rejected: {source}"
            );
        }
    }

    #[test]
    fn renderer_is_optional_and_can_be_disabled() {
        let omitted = Config::from_toml("schema=1\n", "/repo/mambo.toml").unwrap();
        assert!(omitted.renderer.is_none());

        let disabled =
            Config::from_toml("schema=1\n[renderer]\nenabled=false\n", "/repo/mambo.toml").unwrap();
        assert!(disabled.renderer.is_none());
    }

    #[test]
    fn rejects_ancestor_and_descendant_overlaps_between_managed_directories() {
        let cases = [
            (
                "content -> typescript",
                "tree",
                "tree/child",
                "assets",
                "render",
            ),
            (
                "typescript -> content",
                "tree/child",
                "tree",
                "assets",
                "render",
            ),
            (
                "content -> assets",
                "tree",
                "typescript",
                "tree/child",
                "render",
            ),
            (
                "assets -> content",
                "tree/child",
                "typescript",
                "tree",
                "render",
            ),
            (
                "content -> renderer",
                "tree",
                "typescript",
                "assets",
                "tree/child",
            ),
            (
                "renderer -> content",
                "tree/child",
                "typescript",
                "assets",
                "tree",
            ),
            (
                "typescript -> assets",
                "content",
                "tree",
                "tree/child",
                "render",
            ),
            (
                "assets -> typescript",
                "content",
                "tree/child",
                "tree",
                "render",
            ),
            (
                "typescript -> renderer",
                "content",
                "tree",
                "assets",
                "tree/child",
            ),
            (
                "renderer -> typescript",
                "content",
                "tree/child",
                "assets",
                "tree",
            ),
            (
                "assets -> renderer",
                "content",
                "typescript",
                "tree",
                "tree/child",
            ),
            (
                "renderer -> assets",
                "content",
                "typescript",
                "tree/child",
                "tree",
            ),
        ];

        for (case, content, typescript, assets, renderer) in cases {
            let source = format!(
                "schema=1\ncontent_root={content:?}\ntypescript_out={typescript:?}\nassets_out={assets:?}\n[renderer]\noutput_dir={renderer:?}\n"
            );
            let diagnostics = Config::from_toml(&source, "/repo/mambo.toml")
                .expect_err("overlapping directories must be rejected");
            assert!(
                diagnostics.iter().any(|item| item.code == "MS1008"),
                "expected MS1008 for {case}"
            );
        }
    }

    #[test]
    fn ignores_disabled_renderer_output_when_checking_overlaps() {
        let config = Config::from_toml(
            "schema=1\n[renderer]\nenabled=false\noutput_dir=\"docs\"\n",
            "/repo/mambo.toml",
        )
        .expect("a disabled renderer cannot overwrite anything");
        assert!(config.renderer.is_none());
    }

    #[test]
    fn validates_site_url_as_an_absolute_http_url() {
        for url in [
            "https://example.com",
            "https://example.com/docs?language=en#start",
            "http://localhost:3000",
            "https://[2001:db8::1]:8443/docs",
        ] {
            let source = format!("schema=1\n[site]\nurl={url:?}\n");
            let config = Config::from_toml(&source, "/repo/mambo.toml").expect("valid site URL");
            assert_eq!(config.site.url.as_deref(), Some(url));
        }

        for url in [
            "example.com",
            "ftp://example.com",
            "https:///docs",
            "https://example.com:not-a-port",
            "https://bad host.example",
            "../site",
        ] {
            let source = format!("schema=1\n[site]\nurl={url:?}\n");
            let diagnostics =
                Config::from_toml(&source, "/repo/mambo.toml").expect_err("invalid site URL");
            assert!(
                diagnostics.iter().any(|item| item.code == "MS1013"),
                "expected MS1013 for {url}"
            );
        }
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
                "typescript_out=\"linked/generated\"\nassets_out=\"public/assets\"\n",
            ),
            project.join("mambo.toml"),
        )
        .unwrap();

        let diagnostics = config.filesystem_diagnostics();
        assert!(diagnostics.iter().any(|item| item.code == "MS1009"));
    }
}
