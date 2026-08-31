use std::fs;
use std::path::{Path, PathBuf};

use tempfile::Builder;

use crate::{Error, GeneratedTree, OUTPUT_MARKER, OUTPUT_MARKER_CONTENT};

/// Validate that an output and its recovery backup are absent, empty, or
/// owned by `MamboSite` without changing the filesystem.
///
/// # Errors
///
/// Returns an error for invalid targets, symbolic links, unmanaged content, or
/// filesystem inspection failures.
pub fn validate_output(output_dir: &Path) -> Result<(), Error> {
    let parent = output_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| Error::InvalidOutputDirectory(output_dir.display().to_string()))?;
    let output_name = output_dir
        .file_name()
        .ok_or_else(|| Error::InvalidOutputDirectory(output_dir.display().to_string()))?;
    let backup = backup_path(parent, output_name);
    if path_present(output_dir)? {
        validate_managed_or_empty(output_dir)?;
    }
    if path_present(&backup)? {
        validate_managed_or_empty(&backup)?;
    }
    Ok(())
}

/// Write a complete generated tree and replace the previous tree only after all
/// new files have been written successfully.
///
/// # Errors
///
/// Returns an error if the target is not a directory-like path or any staging,
/// writing, replacement, or cleanup operation fails.
pub fn write(generated: &GeneratedTree, output_dir: &Path) -> Result<(), Error> {
    let parent = output_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| Error::InvalidOutputDirectory(output_dir.display().to_string()))?;
    let output_name = output_dir
        .file_name()
        .ok_or_else(|| Error::InvalidOutputDirectory(output_dir.display().to_string()))?;

    validate_output(output_dir)?;
    fs::create_dir_all(parent).map_err(|error| Error::io("create output parent", parent, error))?;
    let backup = backup_path(parent, output_name);
    let had_previous = path_present(output_dir)?;

    let staging = Builder::new()
        .prefix(".mambosite-stage-")
        .tempdir_in(parent)
        .map_err(|error| Error::io("create staging directory", parent, error))?;

    for file in generated.files() {
        let path = staging.path().join(&file.path);
        if let Some(file_parent) = path.parent() {
            fs::create_dir_all(file_parent)
                .map_err(|error| Error::io("create generated directory", file_parent, error))?;
        }
        fs::write(&path, &file.contents)
            .map_err(|error| Error::io("write generated file", &path, error))?;
    }

    let staged_path = staging.keep();
    if had_previous {
        if backup.exists() {
            remove_path(&backup)?;
        }
        fs::rename(output_dir, &backup)
            .map_err(|error| Error::io("move previous generated directory", output_dir, error))?;
    }

    publish_staged(&staged_path, output_dir, &backup, had_previous)
}

fn publish_staged(
    staged_path: &Path,
    output_dir: &Path,
    backup: &Path,
    had_previous: bool,
) -> Result<(), Error> {
    if let Err(publish) = fs::rename(staged_path, output_dir) {
        let _ = remove_path(staged_path);
        if !had_previous {
            return Err(Error::PublishFailed {
                output: output_dir.display().to_string(),
                source: publish,
            });
        }

        return match fs::rename(backup, output_dir) {
            Ok(()) => Err(Error::PublishFailedPreviousRestored {
                output: output_dir.display().to_string(),
                backup: backup.display().to_string(),
                publish,
            }),
            Err(rollback) => Err(Error::PublishAndRollbackFailed {
                output: output_dir.display().to_string(),
                backup: backup.display().to_string(),
                publish,
                rollback,
            }),
        };
    }

    if had_previous {
        remove_path(backup).map_err(|cleanup| Error::PublishedButBackupCleanupFailed {
            output: output_dir.display().to_string(),
            backup: backup.display().to_string(),
            cleanup: Box::new(cleanup),
        })?;
    }
    Ok(())
}

fn validate_managed_or_empty(path: &Path) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Error::io("inspect existing output", path, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(Error::UnmanagedOutputDirectory(path.display().to_string()));
    }
    let mut entries =
        fs::read_dir(path).map_err(|error| Error::io("inspect existing output", path, error))?;
    if entries.next().is_none() {
        return Ok(());
    }
    let marker = path.join(OUTPUT_MARKER);
    let marker_is_regular = fs::symlink_metadata(&marker)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink());
    if marker_is_regular
        && fs::read_to_string(&marker).is_ok_and(|contents| contents == OUTPUT_MARKER_CONTENT)
    {
        return Ok(());
    }
    Err(Error::UnmanagedOutputDirectory(path.display().to_string()))
}

fn backup_path(parent: &Path, output_name: &std::ffi::OsStr) -> PathBuf {
    let mut name = std::ffi::OsString::from(".mambosite-previous-");
    name.push(output_name);
    parent.join(name)
}

fn path_present(path: &Path) -> Result<bool, Error> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(Error::io("inspect generated path", path, error)),
    }
}

fn remove_path(path: &Path) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Error::io("inspect generated path", path, error))?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| Error::io("remove generated directory", path, error))
    } else {
        fs::remove_file(path).map_err(|error| Error::io("remove generated file", path, error))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::generate;

    #[test]
    fn replaces_tree_and_removes_stale_files() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("generated");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join(OUTPUT_MARKER), OUTPUT_MARKER_CONTENT).unwrap();
        fs::write(output.join("stale.ts"), "old").unwrap();

        let generated = generate(&json!({
            "schemaVersion": 1,
            "pages": [{ "id": "p_root", "route": "/", "title": "Home" }]
        }))
        .unwrap();
        write(&generated, &output).unwrap();

        assert!(!output.join("stale.ts").exists());
        assert!(output.join("manifest.ts").is_file());
        assert!(output.join("pages/p_root.ts").is_file());
        assert!(output.join("pages/index.ts").is_file());
    }

    #[test]
    fn refuses_to_replace_an_unmanaged_directory() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("generated");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("keep.txt"), "user data").unwrap();
        let generated = generate(&json!({
            "schemaVersion": 1,
            "pages": [{ "id": "p_root", "route": "/", "title": "Home" }]
        }))
        .unwrap();

        let error = write(&generated, &output).unwrap_err();
        assert!(matches!(error, Error::UnmanagedOutputDirectory(_)));
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).unwrap(),
            "user data"
        );
    }

    #[test]
    fn refuses_an_unrecognized_marker_payload() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("generated");
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join(OUTPUT_MARKER), "not our marker").unwrap();
        let generated = generate(&json!({
            "schemaVersion": 1,
            "pages": [{ "id": "p_root", "route": "/", "title": "Home" }]
        }))
        .unwrap();

        assert!(matches!(
            write(&generated, &output).unwrap_err(),
            Error::UnmanagedOutputDirectory(_)
        ));
    }

    #[test]
    fn reports_when_publish_fails_but_previous_output_is_restored() {
        let temporary = tempfile::tempdir().unwrap();
        let staged = temporary.path().join("missing-stage");
        let output = temporary.path().join("generated");
        let backup = temporary.path().join("previous");
        fs::create_dir(&backup).unwrap();
        fs::write(backup.join("old.ts"), "old").unwrap();

        let error = publish_staged(&staged, &output, &backup, true).unwrap_err();

        assert!(matches!(error, Error::PublishFailedPreviousRestored { .. }));
        assert_eq!(fs::read_to_string(output.join("old.ts")).unwrap(), "old");
        assert!(!backup.exists());
    }

    #[test]
    fn retains_both_errors_when_publish_and_rollback_fail() {
        let temporary = tempfile::tempdir().unwrap();
        let staged = temporary.path().join("missing-stage");
        let output = temporary.path().join("generated");
        let backup = temporary.path().join("missing-previous");

        let error = publish_staged(&staged, &output, &backup, true).unwrap_err();

        match error {
            Error::PublishAndRollbackFailed {
                publish, rollback, ..
            } => {
                assert_eq!(publish.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(rollback.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn reports_cleanup_failure_after_successful_publication() {
        let temporary = tempfile::tempdir().unwrap();
        let staged = temporary.path().join("stage");
        let output = temporary.path().join("generated");
        let missing_backup = temporary.path().join("missing-previous");
        fs::create_dir(&staged).unwrap();
        fs::write(staged.join("new.ts"), "new").unwrap();

        let error = publish_staged(&staged, &output, &missing_backup, true).unwrap_err();

        assert!(matches!(
            error,
            Error::PublishedButBackupCleanupFailed { .. }
        ));
        assert_eq!(fs::read_to_string(output.join("new.ts")).unwrap(), "new");
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_symlinked_marker() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("generated");
        fs::create_dir_all(&output).unwrap();
        let external = temporary.path().join("marker.txt");
        fs::write(&external, OUTPUT_MARKER_CONTENT).unwrap();
        symlink(&external, output.join(OUTPUT_MARKER)).unwrap();
        let generated = generate(&json!({
            "schemaVersion": 1,
            "pages": [{ "id": "p_root", "route": "/", "title": "Home" }]
        }))
        .unwrap();

        assert!(matches!(
            write(&generated, &output).unwrap_err(),
            Error::UnmanagedOutputDirectory(_)
        ));
    }

    #[test]
    fn writes_a_managed_non_typescript_tree() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("assets");
        let generated = GeneratedTree::new([crate::GeneratedFile {
            path: "theme.css".to_owned(),
            contents: ":root { --mambo-test: 1; }\n".to_owned(),
        }])
        .unwrap();

        validate_output(&output).unwrap();
        write(&generated, &output).unwrap();

        assert_eq!(
            fs::read_to_string(output.join("theme.css")).unwrap(),
            ":root { --mambo-test: 1; }\n"
        );
        assert_eq!(
            fs::read_to_string(output.join(OUTPUT_MARKER)).unwrap(),
            OUTPUT_MARKER_CONTENT
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_broken_symlink_output() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("generated");
        symlink(temporary.path().join("missing"), &output).unwrap();

        assert!(matches!(
            validate_output(&output).unwrap_err(),
            Error::UnmanagedOutputDirectory(_)
        ));
    }
}
