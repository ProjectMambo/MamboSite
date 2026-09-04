#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn renderer_stdout_does_not_corrupt_json_diagnostics() {
    let temporary = tempfile::tempdir().unwrap();
    let project = temporary.path().join("site");
    let commands = temporary.path().join("commands");
    fs::create_dir_all(project.join("docs")).unwrap();
    fs::create_dir(&commands).unwrap();
    fs::write(project.join("docs/index.md"), "# Site\n").unwrap();
    fs::write(
        project.join("mambo.toml"),
        concat!(
            "schema=1\n",
            "typescript_out=\"generated\"\n",
            "assets_out=\"public/mambo\"\n",
            "[renderer]\n",
            "build_script=\"site:render\"\n",
            "output_dir=\"out\"\n",
        ),
    )
    .unwrap();
    let npm = commands.join("npm");
    fs::write(&npm, "#!/bin/sh\necho renderer-stdout\nmkdir -p out\n").unwrap();
    let mut permissions = fs::metadata(&npm).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&npm, permissions).unwrap();

    let mut paths = vec![commands];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let path = std::env::join_paths(paths).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_mbsite"))
        .args(["--diagnostics", "json", "build"])
        .current_dir(&project)
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let diagnostics: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(diagnostics, serde_json::json!([]));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("renderer-stdout"));
    assert!(stderr.contains("built 1 page(s)"));
}
