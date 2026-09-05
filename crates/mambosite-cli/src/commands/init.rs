use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::{Builder, TempDir};

use crate::commands::CommandError;

const MANIFEST_PATH: &str = ".mambosite/scaffold.json";
const TEMPLATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_PLACEHOLDER: &str = "__MAMBOSITE_VERSION__";

#[derive(Clone, Copy)]
struct EmbeddedTemplateFile {
    path: &'static str,
    contents: &'static str,
}

macro_rules! template {
    ($path:literal) => {
        EmbeddedTemplateFile {
            path: $path,
            contents: include_str!(concat!("../../../../templates/default/", $path)),
        }
    };
}

const EMBEDDED_TEMPLATE_FILES: &[EmbeddedTemplateFile] = &[
    template!(".gitignore"),
    template!("README.md"),
    template!("mambo.toml"),
    template!("package.json"),
    template!("tsconfig.json"),
    template!("next.config.ts"),
    template!("docs/getting-started.md"),
    template!("docs/index.md"),
    template!("src/app/globals.css"),
    template!("src/app/layout.tsx"),
    template!("src/app/page.tsx"),
    template!("src/app/[...slug]/page.tsx"),
    template!("src/app/not-found.tsx"),
    template!("src/mambo/runtime.ts"),
    template!(".github/workflows/pages.yml"),
];

const RETIRED_TEMPLATE_PATHS: &[&str] = &["src/app/[[...slug]]/page.tsx", "src/mambo/site.ts"];

struct TemplateFile {
    path: &'static str,
    contents: String,
}

fn template_files() -> Result<Vec<TemplateFile>, CommandError> {
    let mut files: Vec<_> = EMBEDDED_TEMPLATE_FILES
        .iter()
        .map(|file| TemplateFile {
            path: file.path,
            contents: file.contents.replace(VERSION_PLACEHOLDER, TEMPLATE_VERSION),
        })
        .collect();
    files.push(TemplateFile {
        path: "mambo.theme.toml",
        contents: mambosite_theme::default_theme_toml().map_err(CommandError::message)?,
    });
    Ok(files)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ScaffoldManifest {
    schema: u32,
    template_version: String,
    files: Vec<String>,
}

#[derive(Debug)]
pub struct InitReport {
    pub target: PathBuf,
    pub file_count: usize,
}

#[allow(clippy::too_many_lines)]
pub fn run(path: &Path, force: bool) -> Result<InitReport, CommandError> {
    let target = absolute_path(path)?;
    prepare_target(&target)?;
    let template_files = template_files()?;

    let entries = visible_entries(&target)?;
    let existing_manifest = load_manifest(&target)?;
    if !force && !entries.is_empty() {
        return Err(CommandError::Message(format!(
            "refusing to initialize non-empty directory `{}`; use `--force` only to refresh an existing MamboSite scaffold",
            target.display()
        )));
    }
    if force && !entries.is_empty() && existing_manifest.is_none() {
        return Err(CommandError::Message(format!(
            "refusing to force initialization in `{}` because it has no MamboSite scaffold manifest",
            target.display()
        )));
    }

    let owned = existing_manifest
        .as_ref()
        .map_or_else(HashSet::new, |manifest| {
            manifest.files.iter().cloned().collect()
        });
    validate_owned_paths(&owned, &template_files)?;
    preflight_template(&target, &owned, force, &template_files)?;

    let stage_parent = target.parent().unwrap_or_else(|| Path::new("."));
    let staging = Builder::new()
        .prefix(".mambosite-init-")
        .tempdir_in(stage_parent)
        .map_err(CommandError::message)?;
    for file in &template_files {
        let destination = staging.path().join(file.path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(CommandError::message)?;
        }
        fs::write(&destination, &file.contents).map_err(CommandError::message)?;
    }

    let manifest = ScaffoldManifest {
        schema: 1,
        template_version: TEMPLATE_VERSION.to_owned(),
        files: template_files
            .iter()
            .map(|file| file.path.to_owned())
            .collect(),
    };
    let staged_manifest = staging.path().join(MANIFEST_PATH);
    fs::create_dir_all(staged_manifest.parent().expect("manifest has a parent"))
        .map_err(CommandError::message)?;
    fs::write(
        &staged_manifest,
        serde_json::to_vec_pretty(&manifest).map_err(CommandError::message)?,
    )
    .map_err(CommandError::message)?;

    publish_template(&target, staging.path(), &template_files, &owned)?;

    Ok(InitReport {
        target,
        file_count: template_files.len(),
    })
}

fn absolute_path(path: &Path) -> Result<PathBuf, CommandError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(CommandError::message)
    }
}

fn prepare_target(target: &Path) -> Result<(), CommandError> {
    if let Ok(metadata) = fs::symlink_metadata(target) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(CommandError::Message(format!(
                "initialization target `{}` must be a regular directory",
                target.display()
            )));
        }
    } else {
        fs::create_dir_all(target).map_err(CommandError::message)?;
    }
    Ok(())
}

fn visible_entries(target: &Path) -> Result<Vec<PathBuf>, CommandError> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(target).map_err(CommandError::message)? {
        let entry = entry.map_err(CommandError::message)?;
        if entry.file_name() != ".git" {
            entries.push(entry.path());
        }
    }
    Ok(entries)
}

fn load_manifest(target: &Path) -> Result<Option<ScaffoldManifest>, CommandError> {
    let path = target.join(MANIFEST_PATH);
    reject_symlink_ancestors(target, &path)?;
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CommandError::message(error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CommandError::Message(format!(
            "scaffold manifest `{}` must be a regular file",
            path.display()
        )));
    }
    let manifest: ScaffoldManifest = serde_json::from_slice(
        &fs::read(&path).map_err(CommandError::message)?,
    )
    .map_err(|error| {
        CommandError::Message(format!(
            "invalid scaffold manifest `{}`: {error}",
            path.display()
        ))
    })?;
    if manifest.schema != 1 {
        return Err(CommandError::Message(format!(
            "unsupported scaffold manifest schema {}",
            manifest.schema
        )));
    }
    Ok(Some(manifest))
}

fn validate_owned_paths(
    paths: &HashSet<String>,
    template_files: &[TemplateFile],
) -> Result<(), CommandError> {
    let known: HashSet<&str> = template_files
        .iter()
        .map(|file| file.path)
        .chain(RETIRED_TEMPLATE_PATHS.iter().copied())
        .collect();
    for path in paths {
        if !safe_relative_path(path)
            || path == ".git"
            || path.starts_with(".git/")
            || !known.contains(path.as_str())
        {
            return Err(CommandError::Message(format!(
                "unknown or unsafe path in scaffold manifest: `{path}`"
            )));
        }
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && !Path::new(value).is_absolute()
}

fn preflight_template(
    target: &Path,
    owned: &HashSet<String>,
    force: bool,
    template_files: &[TemplateFile],
) -> Result<(), CommandError> {
    reject_symlink_ancestors(target, &target.join(MANIFEST_PATH))?;
    for file in template_files {
        let destination = target.join(file.path);
        reject_symlink_ancestors(target, &destination)?;
        if let Ok(metadata) = fs::symlink_metadata(&destination) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(CommandError::Message(format!(
                    "refusing to replace non-file scaffold path `{}`",
                    destination.display()
                )));
            }
            if !force || !owned.contains(file.path) {
                return Err(CommandError::Message(format!(
                    "refusing to replace unknown file `{}`",
                    destination.display()
                )));
            }
        }
    }
    Ok(())
}

fn reject_symlink_ancestors(root: &Path, destination: &Path) -> Result<(), CommandError> {
    let relative = destination
        .strip_prefix(root)
        .map_err(CommandError::message)?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(CommandError::Message(format!(
                    "scaffold path crosses symbolic link `{}`",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(CommandError::message(error)),
        }
    }
    Ok(())
}

fn publish_file(root: &Path, staged: &Path, destination: &Path) -> Result<(), CommandError> {
    reject_symlink_ancestors(root, destination)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(CommandError::message)?;
    }
    match fs::symlink_metadata(destination) {
        Ok(_) => {
            return Err(CommandError::Message(format!(
                "refusing to overwrite scaffold path `{}`",
                destination.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(CommandError::message(error)),
    }
    fs::rename(staged, destination).map_err(CommandError::message)
}

fn publish_template(
    root: &Path,
    staging: &Path,
    template_files: &[TemplateFile],
    owned: &HashSet<String>,
) -> Result<(), CommandError> {
    let current: HashSet<&str> = template_files.iter().map(|file| file.path).collect();
    let mut touched: Vec<&str> = template_files.iter().map(|file| file.path).collect();
    touched.extend(
        owned
            .iter()
            .map(String::as_str)
            .filter(|path| !current.contains(path)),
    );
    touched.push(MANIFEST_PATH);
    touched.sort_unstable();
    touched.dedup();

    for relative in &touched {
        inspect_publish_path(root, &root.join(relative))?;
    }

    let backup = Builder::new()
        .prefix(".mambosite-backup-")
        .tempdir_in(root.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(CommandError::message)?;
    let mut backed_up = Vec::new();
    for relative in &touched {
        match backup_file(root, backup.path(), relative) {
            Ok(true) => backed_up.push((*relative).to_owned()),
            Ok(false) => {}
            Err(error) => {
                let rollback = rollback_publish(root, backup.path(), &[], &backed_up);
                return Err(transaction_error(&error, rollback, backup));
            }
        }
    }

    let mut published = Vec::new();
    for relative in template_files
        .iter()
        .map(|file| file.path)
        .chain(std::iter::once(MANIFEST_PATH))
    {
        if let Err(error) = publish_file(root, &staging.join(relative), &root.join(relative)) {
            let rollback = rollback_publish(root, backup.path(), &published, &backed_up);
            return Err(transaction_error(&error, rollback, backup));
        }
        published.push(relative.to_owned());
    }
    Ok(())
}

fn inspect_publish_path(root: &Path, path: &Path) -> Result<(), CommandError> {
    reject_symlink_ancestors(root, path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(CommandError::Message(format!(
            "refusing to replace non-file scaffold path `{}`",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CommandError::message(error)),
    }
}

fn backup_file(root: &Path, backup: &Path, relative: &str) -> Result<bool, CommandError> {
    let source = root.join(relative);
    reject_symlink_ancestors(root, &source)?;
    match fs::symlink_metadata(&source) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            let destination = backup.join(relative);
            fs::create_dir_all(destination.parent().expect("backup file has a parent"))
                .map_err(CommandError::message)?;
            fs::rename(source, destination).map_err(CommandError::message)?;
            Ok(true)
        }
        Ok(_) => Err(CommandError::Message(format!(
            "refusing to replace non-file scaffold path `{}`",
            source.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(CommandError::message(error)),
    }
}

fn rollback_publish(
    root: &Path,
    backup: &Path,
    published: &[String],
    backed_up: &[String],
) -> Result<(), CommandError> {
    let mut failures = Vec::new();
    for relative in published.iter().rev() {
        let destination = root.join(relative);
        if let Err(error) = reject_symlink_ancestors(root, &destination)
            .and_then(|()| remove_published_file(&destination))
        {
            failures.push(error.to_string());
        }
    }
    for relative in backed_up.iter().rev() {
        let destination = root.join(relative);
        let result = reject_symlink_ancestors(root, &destination).and_then(|()| {
            match fs::symlink_metadata(&destination) {
                Ok(_) => {
                    return Err(CommandError::Message(format!(
                        "cannot restore scaffold file because `{}` now exists",
                        destination.display()
                    )));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(CommandError::message(error)),
            }
            fs::create_dir_all(destination.parent().expect("scaffold file has a parent"))
                .map_err(CommandError::message)?;
            fs::rename(backup.join(relative), destination).map_err(CommandError::message)
        });
        if let Err(error) = result {
            failures.push(error.to_string());
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(CommandError::Message(failures.join("; ")))
    }
}

fn remove_published_file(path: &Path) -> Result<(), CommandError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            fs::remove_file(path).map_err(CommandError::message)
        }
        Ok(_) => Err(CommandError::Message(format!(
            "refusing to remove unexpected scaffold path `{}`",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CommandError::message(error)),
    }
}

fn transaction_error(
    publish: &CommandError,
    rollback: Result<(), CommandError>,
    backup: TempDir,
) -> CommandError {
    match rollback {
        Ok(()) => CommandError::Message(format!(
            "scaffold update failed; previous files were restored: {publish}"
        )),
        Err(rollback) => {
            let recovery = backup.keep();
            CommandError::Message(format!(
                "scaffold update failed: {publish}; rollback also failed: {rollback}; recovery files were kept at `{}`",
                recovery.display()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_an_empty_or_git_only_repository() {
        let temporary = tempfile::tempdir().unwrap();
        let empty = temporary.path().join("empty");
        let report = run(&empty, false).unwrap();
        assert_eq!(report.file_count, template_files().unwrap().len());
        assert!(empty.join("mambo.toml").is_file());
        assert!(empty.join("mambo.theme.toml").is_file());
        assert!(empty.join("docs/getting-started.md").is_file());
        assert!(empty.join("src/app/page.tsx").is_file());
        assert!(empty.join("src/app/[...slug]/page.tsx").is_file());
        assert!(empty.join("src/app/not-found.tsx").is_file());
        assert!(empty.join("src/mambo/runtime.ts").is_file());
        assert!(!empty.join("src/app/[[...slug]]/page.tsx").exists());
        assert!(!empty.join("src/mambo/site.ts").exists());
        assert_eq!(
            fs::read_to_string(empty.join("mambo.theme.toml")).unwrap(),
            mambosite_theme::default_theme_toml().unwrap()
        );
        assert!(empty.join(MANIFEST_PATH).is_file());
        assert!(!empty.join("node_modules").exists());

        let git_only = temporary.path().join("git-only");
        fs::create_dir_all(git_only.join(".git")).unwrap();
        run(&git_only, false).unwrap();
        assert!(git_only.join(".git").is_dir());
        assert!(git_only.join("docs/index.md").is_file());
    }

    #[test]
    fn refuses_unknown_nonempty_directories_even_with_force() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("existing");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("notes.txt"), "keep").unwrap();

        assert!(run(&target, false).is_err());
        assert!(run(&target, true).is_err());
        assert_eq!(
            fs::read_to_string(target.join("notes.txt")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn force_refreshes_owned_files_and_preserves_unknown_files() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        run(&target, false).unwrap();
        fs::write(target.join("README.md"), "customized scaffold").unwrap();
        fs::write(target.join("personal.txt"), "preserve me").unwrap();

        run(&target, true).unwrap();

        assert_eq!(
            fs::read_to_string(target.join("README.md")).unwrap(),
            include_str!("../../../../templates/default/README.md")
                .replace(VERSION_PLACEHOLDER, TEMPLATE_VERSION)
        );
        assert_eq!(
            fs::read_to_string(target.join("personal.txt")).unwrap(),
            "preserve me"
        );
    }

    #[test]
    fn force_rejects_manifest_paths_not_owned_by_the_template() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        run(&target, false).unwrap();
        fs::write(target.join("personal.txt"), "preserve me").unwrap();

        let manifest_path = target.join(MANIFEST_PATH);
        let mut manifest: ScaffoldManifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.files.push("personal.txt".to_owned());
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        assert!(run(&target, true).is_err());
        assert_eq!(
            fs::read_to_string(target.join("personal.txt")).unwrap(),
            "preserve me"
        );
    }

    #[test]
    fn restores_all_owned_files_after_a_partial_publish_failure() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        let staging = temporary.path().join("staging");
        fs::create_dir_all(target.join(".mambosite")).unwrap();
        fs::create_dir_all(&staging).unwrap();
        fs::write(target.join("a.txt"), "old a").unwrap();
        fs::write(target.join("b.txt"), "old b").unwrap();
        fs::write(target.join(MANIFEST_PATH), "old manifest").unwrap();
        fs::write(staging.join("a.txt"), "new a").unwrap();
        fs::create_dir_all(staging.join(".mambosite")).unwrap();
        fs::write(staging.join(MANIFEST_PATH), "new manifest").unwrap();

        let files = [
            TemplateFile {
                path: "a.txt",
                contents: String::new(),
            },
            TemplateFile {
                path: "b.txt",
                contents: String::new(),
            },
        ];
        let owned = HashSet::from(["a.txt".to_owned(), "b.txt".to_owned()]);

        let error = publish_template(&target, &staging, &files, &owned).unwrap_err();

        assert!(error.to_string().contains("previous files were restored"));
        assert_eq!(fs::read_to_string(target.join("a.txt")).unwrap(), "old a");
        assert_eq!(fs::read_to_string(target.join("b.txt")).unwrap(), "old b");
        assert_eq!(
            fs::read_to_string(target.join(MANIFEST_PATH)).unwrap(),
            "old manifest"
        );
    }

    #[test]
    fn scaffold_uses_the_public_runtime_and_lifecycle_contracts() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        run(&target, false).unwrap();

        let runtime = fs::read_to_string(target.join("src/mambo/runtime.ts")).unwrap();
        assert!(runtime.contains("createNextRuntime"));
        assert!(runtime.contains("defaultRegistry"));
        assert!(runtime.contains("import manifest from"));
        assert!(runtime.contains("../generated/mambo/manifest"));
        assert!(runtime.contains("../generated/mambo/pages"));
        assert!(runtime.contains("../generated/mambo/theme"));
        assert!(runtime.contains("themeStylesheetHref"));

        let layout = fs::read_to_string(target.join("src/app/layout.tsx")).unwrap();
        for public_api in [
            "MamboSiteFrame",
            "prefixBasePath",
            "siteMetadata",
            "themeBootstrapScript",
        ] {
            assert!(layout.contains(public_api));
        }
        assert!(layout.contains("themeStylesheetHref"));

        let next_config = fs::read_to_string(target.join("next.config.ts")).unwrap();
        assert!(next_config.contains("./src/generated/mambo/manifest"));
        assert!(next_config.contains("basePath: manifest.site.basePath"));
        assert!(next_config.contains("trailingSlash: manifest.site.trailingSlash"));
        assert!(!next_config.contains("MAMBOSITE_BASE_PATH"));

        let route = fs::read_to_string(target.join("src/app/[...slug]/page.tsx")).unwrap();
        assert!(route.contains("export const dynamicParams = false"));
        assert!(route.contains("export function generateStaticParams()"));
        assert!(route.contains("pageFromSegments"));
        assert!(route.contains("<MamboPage"));

        let not_found = fs::read_to_string(target.join("src/app/not-found.tsx")).unwrap();
        assert!(not_found.contains("MamboNotFound"));
        let styles = fs::read_to_string(target.join("src/app/globals.css")).unwrap();
        assert!(styles.contains("@mambosite/theme-default/styles.css"));

        let package: serde_json::Value =
            serde_json::from_slice(&fs::read(target.join("package.json")).unwrap()).unwrap();
        let scripts = package["scripts"].as_object().unwrap();
        assert_eq!(scripts["predev"], "mbsite build --content-only");
        assert_eq!(scripts["dev"], "next dev");
        assert_eq!(scripts["build"], "mbsite build");
        assert_eq!(scripts["mambosite:render"], "next build");
        assert_eq!(scripts["deploy"], "mbsite deploy");
        assert!(!scripts.contains_key("prebuild"));
        for dependency in [
            "@mambosite/next",
            "@mambosite/react",
            "@mambosite/runtime",
            "@mambosite/theme-default",
        ] {
            assert_eq!(package["dependencies"][dependency], TEMPLATE_VERSION);
        }

        let workflow = fs::read_to_string(target.join(".github/workflows/pages.yml")).unwrap();
        assert!(workflow.contains("Require npm lockfile"));
        assert!(workflow.contains("[ ! -f package-lock.json ]"));
        assert!(workflow.contains("--package mambosite-cli -- build"));
        assert_eq!(
            workflow.matches("--package mambosite-cli -- build").count(),
            1
        );
        assert!(workflow.contains(&format!("ref: v{TEMPLATE_VERSION}")));
        assert!(!workflow.contains("next build"));

        let readme = fs::read_to_string(target.join("README.md")).unwrap();
        assert!(readme.contains(TEMPLATE_VERSION));
        for generated in [package.to_string(), workflow, readme] {
            assert!(!generated.contains(VERSION_PLACEHOLDER));
        }
    }

    #[test]
    fn force_removes_files_retired_from_an_owned_scaffold() {
        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        run(&target, false).unwrap();

        let manifest_path = target.join(MANIFEST_PATH);
        let mut manifest: ScaffoldManifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        for path in RETIRED_TEMPLATE_PATHS {
            let destination = target.join(path);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::write(&destination, "retired scaffold file").unwrap();
            manifest.files.push((*path).to_owned());
        }
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        run(&target, true).unwrap();

        for path in RETIRED_TEMPLATE_PATHS {
            assert!(!target.join(path).exists());
        }
        assert!(target.join("src/app/page.tsx").is_file());
        assert!(target.join("src/app/[...slug]/page.tsx").is_file());
        assert!(target.join("src/mambo/runtime.ts").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn force_rejects_a_symlinked_manifest_parent_without_touching_the_external_manifest() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        run(&target, false).unwrap();

        let manifest_path = target.join(MANIFEST_PATH);
        let mut manifest: ScaffoldManifest =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest.template_version = "external-do-not-touch".to_owned();
        let protected_manifest = serde_json::to_vec_pretty(&manifest).unwrap();
        let external = temporary.path().join("external");
        fs::create_dir(&external).unwrap();
        fs::write(external.join("scaffold.json"), &protected_manifest).unwrap();

        fs::remove_dir_all(target.join(".mambosite")).unwrap();
        symlink(&external, target.join(".mambosite")).unwrap();

        let error = run(&target, true).unwrap_err();
        assert!(error.to_string().contains("crosses symbolic link"));
        assert_eq!(
            fs::read(external.join("scaffold.json")).unwrap(),
            protected_manifest
        );
    }

    #[cfg(unix)]
    #[test]
    fn publish_rechecks_manifest_ancestors_after_preflight() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let target = temporary.path().join("site");
        fs::create_dir(&target).unwrap();
        preflight_template(&target, &HashSet::new(), false, &template_files().unwrap()).unwrap();

        let external = temporary.path().join("external");
        fs::create_dir(&external).unwrap();
        let external_manifest = external.join("scaffold.json");
        fs::write(&external_manifest, "protected").unwrap();
        symlink(&external, target.join(".mambosite")).unwrap();

        let staged = temporary.path().join("staged.json");
        fs::write(&staged, "replacement").unwrap();
        let error = publish_file(&target, &staged, &target.join(MANIFEST_PATH)).unwrap_err();

        assert!(error.to_string().contains("crosses symbolic link"));
        assert_eq!(fs::read_to_string(external_manifest).unwrap(), "protected");
        assert_eq!(fs::read_to_string(staged).unwrap(), "replacement");
    }
}
