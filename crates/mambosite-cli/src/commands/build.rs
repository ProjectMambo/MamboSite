use std::fmt::Write as _;
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mambosite_codegen_ts::{GeneratedFile, GeneratedTree};
use mambosite_core::{CompileOutcome, Config, Diagnostic, PackageManager, RendererKind};
use mambosite_theme::{
    CompiledTheme, Theme, ThemeDiagnostic, ThemeError, compile_theme_file_with_accent_seed,
};

use crate::commands::CommandError;
use crate::process::{ChildStdout, ProcessSpec, run_inherited};

const RENDERER_ACTIVE_ENV: &str = "MAMBOSITE_INTERNAL_RENDERER_ACTIVE";
const MAX_JAVASCRIPT_DATE_SECONDS: u64 = 8_640_000_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildMode {
    Check,
    ContentOnly,
    Full,
}

#[derive(Debug)]
pub struct BuildReport {
    pub page_count: usize,
    pub generated_dir: Option<PathBuf>,
    pub assets_dir: Option<PathBuf>,
    pub artifact_dir: Option<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn run(
    config: Config,
    mode: BuildMode,
    child_stdout: ChildStdout,
) -> Result<BuildReport, CommandError> {
    reject_renderer_recursion(mode, std::env::var_os(RENDERER_ACTIVE_ENV).as_deref())?;

    let generated_dir = config.typescript_out.clone();
    let assets_dir = config.assets_out.clone();
    let artifact_dir = config
        .renderer
        .as_ref()
        .map(|renderer| renderer.output_dir.clone());
    let renderer = renderer_spec(&config);
    let project_root = config.project_root.clone();
    let CompileOutcome {
        site,
        assets: compiled_assets,
        diagnostics,
    } = mambosite_core::Compiler::new(config).compile();

    if diagnostics.iter().any(Diagnostic::is_error) {
        return Err(CommandError::Diagnostics(diagnostics));
    }
    let Some(mut site) = site else {
        return Err(CommandError::Message(
            "compilation produced neither a site nor an error diagnostic".to_owned(),
        ));
    };
    let page_count = site.pages.len();
    let (accent_seed, generated_at) = if mode == BuildMode::Check {
        (0, None)
    } else if let Some(epoch) = source_date_epoch(std::env::var_os("SOURCE_DATE_EPOCH").as_deref())?
    {
        (epoch, Some(epoch))
    } else {
        (random_accent_seed(), Some(current_epoch_seconds()?))
    };
    site.generated_at = generated_at;
    let mut theme = compile_project_theme(&project_root, accent_seed)?;

    if mode == BuildMode::Check {
        return Ok(BuildReport {
            page_count,
            generated_dir: None,
            assets_dir: None,
            artifact_dir: None,
            diagnostics,
        });
    }

    let theme_stylesheet_href = theme_stylesheet_href(&project_root, &assets_dir)?;
    write!(
        theme.typescript,
        "\nexport const themeStylesheetHref = {} as const;\n",
        serde_json::to_string(&theme_stylesheet_href).expect("a path string is serializable")
    )
    .expect("writing to a string cannot fail");
    let mut typescript = mambosite_codegen_ts::generate(&site)
        .map_err(|error| CommandError::Message(format!("TypeScript generation failed: {error}")))?;
    typescript
        .insert(GeneratedFile {
            path: "theme.ts".to_owned(),
            contents: theme.typescript.into_bytes(),
        })
        .map_err(|error| CommandError::Message(format!("theme generation failed: {error}")))?;
    let mut assets = GeneratedTree::new([GeneratedFile {
        path: "theme.css".to_owned(),
        contents: theme.css.into_bytes(),
    }])
    .map_err(|error| CommandError::Message(format!("theme generation failed: {error}")))?;
    for asset in compiled_assets {
        assets
            .insert(GeneratedFile {
                path: format!("assets/{}", asset.output_path),
                contents: asset.contents,
            })
            .map_err(|error| CommandError::Message(format!("asset generation failed: {error}")))?;
    }

    // Validate every destination before either managed tree is mutated.
    mambosite_codegen_ts::validate_output(&generated_dir)
        .map_err(|error| CommandError::Message(format!("TypeScript output is unsafe: {error}")))?;
    mambosite_codegen_ts::validate_output(&assets_dir)
        .map_err(|error| CommandError::Message(format!("asset output is unsafe: {error}")))?;
    mambosite_codegen_ts::write(&typescript, &generated_dir)
        .map_err(|error| CommandError::Message(format!("TypeScript generation failed: {error}")))?;
    mambosite_codegen_ts::write(&assets, &assets_dir)
        .map_err(|error| CommandError::Message(format!("asset generation failed: {error}")))?;

    if mode == BuildMode::ContentOnly {
        return Ok(BuildReport {
            page_count,
            generated_dir: Some(generated_dir),
            assets_dir: Some(assets_dir),
            artifact_dir: None,
            diagnostics,
        });
    }

    if let Some(renderer) = &renderer {
        run_inherited(renderer, &project_root, child_stdout).map_err(CommandError::message)?;
    }
    if let Some(artifact_dir) = &artifact_dir {
        validate_artifact(artifact_dir)?;
    }

    Ok(BuildReport {
        page_count,
        generated_dir: Some(generated_dir),
        assets_dir: Some(assets_dir),
        artifact_dir,
        diagnostics,
    })
}

fn theme_stylesheet_href(project_root: &Path, assets_dir: &Path) -> Result<String, CommandError> {
    let relative = assets_dir
        .strip_prefix(project_root.join("public"))
        .map_err(|_| CommandError::Message("`assets_out` must be under `public/`".to_owned()))?;
    let relative = relative
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    Ok(format!("/{relative}/theme.css"))
}

fn compile_project_theme(
    project_root: &Path,
    accent_seed: u64,
) -> Result<CompiledTheme, CommandError> {
    let path = project_root.join("mambo.theme.toml");
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(CommandError::Message(format!(
                    "theme configuration `{}` must be a regular file",
                    path.display()
                )));
            }
            compile_theme_file_with_accent_seed(&path, accent_seed).map_err(theme_error)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Theme::default()
            .compile_with_accent_seed(accent_seed)
            .map_err(|diagnostics| theme_diagnostics("default theme", &diagnostics)),
        Err(error) => Err(CommandError::Message(format!(
            "could not inspect theme configuration `{}`: {error}",
            path.display()
        ))),
    }
}

fn source_date_epoch(value: Option<&std::ffi::OsStr>) -> Result<Option<u64>, CommandError> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .to_str()
        .and_then(|value| value.parse().ok())
        .filter(|epoch| *epoch <= MAX_JAVASCRIPT_DATE_SECONDS)
        .map(Some)
        .ok_or_else(|| {
            CommandError::Message(format!(
                "`SOURCE_DATE_EPOCH` must be an integer from 0 to {MAX_JAVASCRIPT_DATE_SECONDS}"
            ))
        })
}

fn current_epoch_seconds() -> Result<u64, CommandError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| {
            CommandError::Message(format!("system clock precedes Unix epoch: {error}"))
        })
}

fn random_accent_seed() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
}

fn theme_error(error: ThemeError) -> CommandError {
    if error.diagnostics().is_empty() {
        CommandError::message(error)
    } else {
        theme_diagnostics(&error.to_string(), error.diagnostics())
    }
}

fn theme_diagnostics(label: &str, diagnostics: &[ThemeDiagnostic]) -> CommandError {
    let details = diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{} at `{}`: {}",
                diagnostic.code, diagnostic.field, diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ");
    CommandError::Message(format!("{label}:\n  {details}"))
}

fn renderer_spec(config: &Config) -> Option<ProcessSpec> {
    let renderer = config.renderer.as_ref()?;
    let program = match renderer.package_manager {
        PackageManager::Npm => "npm",
        PackageManager::Pnpm => "pnpm",
        PackageManager::Yarn => "yarn",
        PackageManager::Bun => "bun",
    };
    let command = match renderer.kind {
        RendererKind::Next => "run",
    };
    Some(
        ProcessSpec::new(program, [command.to_owned(), renderer.build_script.clone()])
            .with_env(RENDERER_ACTIVE_ENV, "1"),
    )
}

fn reject_renderer_recursion(
    mode: BuildMode,
    renderer_active: Option<&std::ffi::OsStr>,
) -> Result<(), CommandError> {
    if mode == BuildMode::Full && renderer_active.is_some() {
        Err(CommandError::Message(
            "refusing to start a nested full MamboSite build while the renderer is active; `renderer.build_script` must run the renderer directly, not `mbsite build` or `mbsite deploy`"
                .to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn validate_artifact(path: &std::path::Path) -> Result<(), CommandError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        CommandError::Message(format!(
            "renderer completed but static output `{}` is unavailable: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CommandError::Message(format!(
            "renderer output `{}` must be a regular directory",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site_config(root: &Path) -> Config {
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(root.join("docs/index.md"), "# Test site\n").unwrap();
        fs::write(
            root.join("mambo.toml"),
            concat!(
                "schema=1\n",
                "content_root=\"docs\"\n",
                "typescript_out=\"generated\"\n",
                "assets_out=\"public/site-assets\"\n",
            ),
        )
        .unwrap();
        Config::load(root.join("mambo.toml")).unwrap()
    }

    fn managed_tree(path: &Path, file: &str) {
        let tree = GeneratedTree::new([GeneratedFile {
            path: file.to_owned(),
            contents: b"previous\n".to_vec(),
        }])
        .unwrap();
        mambosite_codegen_ts::write(&tree, path).unwrap();
    }

    #[test]
    fn validates_source_date_epoch_for_javascript_dates() {
        assert_eq!(
            source_date_epoch(Some(std::ffi::OsStr::new("1234"))).unwrap(),
            Some(1234)
        );
        assert_eq!(
            source_date_epoch(Some(std::ffi::OsStr::new("8640000000000"))).unwrap(),
            Some(MAX_JAVASCRIPT_DATE_SECONDS)
        );
        assert!(source_date_epoch(Some(std::ffi::OsStr::new("8640000000001"))).is_err());
        assert!(source_date_epoch(Some(std::ffi::OsStr::new("tomorrow"))).is_err());
    }

    #[test]
    fn creates_direct_package_manager_commands() {
        for (manager, program) in [
            (PackageManager::Npm, "npm"),
            (PackageManager::Pnpm, "pnpm"),
            (PackageManager::Yarn, "yarn"),
            (PackageManager::Bun, "bun"),
        ] {
            let mut config = Config::from_toml(
                "schema=1\n[renderer]\nbuild_script=\"site:render\"\n",
                "/repo/mambo.toml",
            )
            .unwrap();
            config
                .renderer
                .as_mut()
                .expect("test renderer")
                .package_manager = manager;
            let spec = renderer_spec(&config).expect("test renderer command");
            assert_eq!(spec.program, program);
            assert_eq!(spec.args, ["run", "site:render"]);
            assert!(
                spec.env
                    .iter()
                    .any(|(name, value)| { name == RENDERER_ACTIVE_ENV && value == "1" })
            );
        }
    }

    #[test]
    fn rejects_nested_full_builds_while_allowing_non_renderer_modes() {
        let active = Some(std::ffi::OsStr::new("1"));
        assert!(reject_renderer_recursion(BuildMode::Full, active).is_err());
        assert!(reject_renderer_recursion(BuildMode::Check, active).is_ok());
        assert!(reject_renderer_recursion(BuildMode::ContentOnly, active).is_ok());
        assert!(reject_renderer_recursion(BuildMode::Full, None).is_ok());
    }

    #[test]
    fn omitted_renderer_produces_no_process() {
        let config = Config::from_toml("schema=1\n", "/repo/mambo.toml").unwrap();
        assert!(renderer_spec(&config).is_none());
    }

    #[test]
    fn rejects_missing_or_symlinked_artifacts() {
        let temporary = tempfile::tempdir().unwrap();
        let missing = temporary.path().join("missing");
        assert!(validate_artifact(&missing).is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let real = temporary.path().join("real");
            fs::create_dir(&real).unwrap();
            let linked = temporary.path().join("linked");
            symlink(real, &linked).unwrap();
            assert!(validate_artifact(&linked).is_err());
        }
    }

    #[test]
    fn content_build_publishes_site_and_theme_trees() {
        let temporary = tempfile::tempdir().unwrap();
        let config = site_config(temporary.path());
        fs::write(
            temporary.path().join("mambo.theme.toml"),
            "id=\"custom\"\n[colors.light]\nbackground=\"#abcdef\"\n",
        )
        .unwrap();

        let report = run(config, BuildMode::ContentOnly, ChildStdout::Stdout).unwrap();

        assert_eq!(report.page_count, 1);
        assert_eq!(
            report.assets_dir.as_deref(),
            Some(temporary.path().join("public/site-assets").as_path())
        );
        assert!(
            fs::read_to_string(temporary.path().join("generated/theme.ts"))
                .unwrap()
                .contains("\"id\": \"custom\"")
        );
        assert!(
            fs::read_to_string(temporary.path().join("generated/theme.ts"))
                .unwrap()
                .contains(
                    "export const themeStylesheetHref = \"/site-assets/theme.css\" as const;"
                )
        );
        assert!(
            fs::read_to_string(temporary.path().join("public/site-assets/theme.css"))
                .unwrap()
                .contains("--mambo-color-background: #abcdef;")
        );
        assert!(temporary.path().join("generated/manifest.ts").is_file());
        assert!(
            fs::read_to_string(temporary.path().join("generated/manifest.ts"))
                .unwrap()
                .contains("\"generatedAt\":")
        );
        assert!(
            temporary
                .path()
                .join("generated/.mambosite-generated")
                .is_file()
        );
        assert!(
            temporary
                .path()
                .join("public/site-assets/.mambosite-generated")
                .is_file()
        );
    }

    #[test]
    fn content_build_publishes_binary_assets_and_removes_stale_ones() {
        let temporary = tempfile::tempdir().unwrap();
        let config = site_config(temporary.path());
        fs::create_dir_all(temporary.path().join("docs/_assets/media")).unwrap();
        fs::write(
            temporary.path().join("docs/_assets/media/sample.bin"),
            [0, 255, 128, 1],
        )
        .unwrap();
        fs::write(
            temporary.path().join("docs/index.md"),
            "---\ncover: assets/media/sample.bin\n---\n# Test site\n",
        )
        .unwrap();

        run(config.clone(), BuildMode::ContentOnly, ChildStdout::Stdout).unwrap();

        assert_eq!(
            fs::read(
                temporary
                    .path()
                    .join("public/site-assets/assets/media/sample.bin")
            )
            .unwrap(),
            [0, 255, 128, 1]
        );
        assert!(
            fs::read_to_string(temporary.path().join("generated/manifest.ts"))
                .unwrap()
                .contains("/site-assets/assets/media/sample.bin")
        );

        fs::remove_file(temporary.path().join("docs/_assets/media/sample.bin")).unwrap();
        fs::write(temporary.path().join("docs/index.md"), "# Test site\n").unwrap();
        run(config, BuildMode::ContentOnly, ChildStdout::Stdout).unwrap();

        assert!(
            !temporary
                .path()
                .join("public/site-assets/assets/media/sample.bin")
                .exists()
        );
        assert!(
            temporary
                .path()
                .join("public/site-assets/theme.css")
                .is_file()
        );
    }

    #[test]
    fn check_validates_the_default_and_project_themes_without_writes() {
        let temporary = tempfile::tempdir().unwrap();
        let config = site_config(temporary.path());
        let default_theme = compile_project_theme(temporary.path(), 0).unwrap();
        assert_eq!(default_theme.theme, Theme::default());
        assert_eq!(
            run(config.clone(), BuildMode::Check, ChildStdout::Stdout)
                .unwrap()
                .page_count,
            1
        );
        assert!(!temporary.path().join("generated").exists());
        assert!(!temporary.path().join("public/site-assets").exists());

        fs::write(temporary.path().join("mambo.theme.toml"), "schema=99\n").unwrap();
        assert!(run(config, BuildMode::Check, ChildStdout::Stdout).is_err());
        assert!(!temporary.path().join("generated").exists());
        assert!(!temporary.path().join("public/site-assets").exists());
    }

    #[test]
    fn invalid_theme_or_unmanaged_assets_leave_typescript_untouched() {
        let temporary = tempfile::tempdir().unwrap();
        let config = site_config(temporary.path());
        managed_tree(&temporary.path().join("generated"), "previous.ts");
        fs::write(temporary.path().join("mambo.theme.toml"), "schema=99\n").unwrap();

        assert!(run(config.clone(), BuildMode::ContentOnly, ChildStdout::Stdout).is_err());
        assert_eq!(
            fs::read_to_string(temporary.path().join("generated/previous.ts")).unwrap(),
            "previous\n"
        );

        fs::remove_file(temporary.path().join("mambo.theme.toml")).unwrap();
        fs::create_dir_all(temporary.path().join("public/site-assets")).unwrap();
        fs::write(
            temporary.path().join("public/site-assets/user.txt"),
            "keep\n",
        )
        .unwrap();
        assert!(run(config, BuildMode::ContentOnly, ChildStdout::Stdout).is_err());
        assert_eq!(
            fs::read_to_string(temporary.path().join("generated/previous.ts")).unwrap(),
            "previous\n"
        );
        assert_eq!(
            fs::read_to_string(temporary.path().join("public/site-assets/user.txt")).unwrap(),
            "keep\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_symlinked_theme_file() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let external = temporary.path().join("external.toml");
        fs::write(&external, "").unwrap();
        let project = temporary.path().join("project");
        fs::create_dir(&project).unwrap();
        symlink(external, project.join("mambo.theme.toml")).unwrap();

        assert!(compile_project_theme(&project, 0).is_err());
    }
}
