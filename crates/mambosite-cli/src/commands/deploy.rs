use std::fs;
use std::path::Path;

use mambosite_core::Config;

use crate::commands::CommandError;
use crate::commands::build::{self, BuildMode, BuildReport};
use crate::process::{ChildStdout, ProcessSpec, capture, run_inherited};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DeployAction {
    Push { commits: usize },
    Dispatch,
}

#[derive(Debug)]
pub struct DeployReport {
    pub build: BuildReport,
    pub action: DeployAction,
    pub dry_run: bool,
}

#[allow(clippy::too_many_lines)]
pub fn run(
    config: &Config,
    dry_run: bool,
    child_stdout: ChildStdout,
) -> Result<DeployReport, CommandError> {
    let root = config.project_root.clone();
    verify_repository_root(&root)?;
    require_clean(&root)?;
    verify_workflow(&root, &config.deploy.workflow, &config.deploy.branch)?;
    let repository = github_repository(&git(
        &root,
        ["remote", "get-url", config.deploy.remote.as_str()],
    )?)
    .ok_or_else(|| {
        CommandError::Message(format!(
            "Git remote `{}` is not a GitHub repository",
            config.deploy.remote
        ))
    })?;
    let branch = git(&root, ["branch", "--show-current"])?;
    if branch.trim() != config.deploy.branch {
        return Err(CommandError::Message(format!(
            "deployment branch is `{}`, but the current branch is `{}`",
            config.deploy.branch,
            branch.trim()
        )));
    }

    let build = build::run(config.clone(), BuildMode::Full, child_stdout)?;
    if build.artifact_dir.is_none() {
        return Err(CommandError::Message(
            "deployment requires an enabled renderer and a static output directory".to_owned(),
        ));
    }
    require_clean(&root)?;

    let remote_ref = format!(
        "refs/remotes/{}/{}",
        config.deploy.remote, config.deploy.branch
    );
    let action = if dry_run {
        require_remote_tracking_ref(
            &root,
            &remote_ref,
            &config.deploy.remote,
            &config.deploy.branch,
        )?;
        action_against_remote(&root, &remote_ref)?
    } else if remote_branch_exists(&root, &config.deploy.remote, &config.deploy.branch)? {
        fetch_remote_branch(
            &root,
            &config.deploy.remote,
            &config.deploy.branch,
            &remote_ref,
            child_stdout,
        )?;
        action_against_remote(&root, &remote_ref)?
    } else {
        initial_push_action(&root)?
    };

    match &action {
        DeployAction::Push { .. } if !dry_run => run_inherited(
            &ProcessSpec::new(
                "git",
                [
                    "push".to_owned(),
                    "--porcelain".to_owned(),
                    config.deploy.remote.clone(),
                    format!("HEAD:refs/heads/{}", config.deploy.branch),
                ],
            ),
            &root,
            child_stdout,
        )
        .map_err(CommandError::message)?,
        DeployAction::Dispatch if !dry_run => {
            run_inherited(
                &ProcessSpec::new("gh", ["auth", "status", "--hostname", "github.com"]),
                &root,
                child_stdout,
            )
            .map_err(CommandError::message)?;
            let workflow_name = Path::new(&config.deploy.workflow)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    CommandError::Message("invalid deployment workflow name".to_owned())
                })?;
            run_inherited(
                &ProcessSpec::new(
                    "gh",
                    [
                        "workflow".to_owned(),
                        "run".to_owned(),
                        workflow_name.to_owned(),
                        "--ref".to_owned(),
                        config.deploy.branch.clone(),
                        "--repo".to_owned(),
                        repository,
                    ],
                ),
                &root,
                child_stdout,
            )
            .map_err(CommandError::message)?;
        }
        DeployAction::Push { .. } | DeployAction::Dispatch => {}
    }

    Ok(DeployReport {
        build,
        action,
        dry_run,
    })
}

fn git<const N: usize>(root: &Path, args: [&str; N]) -> Result<String, CommandError> {
    capture(&ProcessSpec::new("git", args), root).map_err(CommandError::message)
}

fn require_remote_tracking_ref(
    root: &Path,
    remote_ref: &str,
    remote: &str,
    branch: &str,
) -> Result<(), CommandError> {
    git(root, ["rev-parse", "--verify", remote_ref])
        .map(|_| ())
        .map_err(|_| {
            CommandError::Message(format!(
                "remote branch `{remote}/{branch}` is unavailable locally; fetch it before using `--dry-run`"
            ))
        })
}

fn remote_branch_exists(root: &Path, remote: &str, branch: &str) -> Result<bool, CommandError> {
    let reference = format!("refs/heads/{branch}");
    let output = git(root, ["ls-remote", "--heads", remote, &reference])?;
    Ok(!output.trim().is_empty())
}

fn fetch_remote_branch(
    root: &Path,
    remote: &str,
    branch: &str,
    remote_ref: &str,
    child_stdout: ChildStdout,
) -> Result<(), CommandError> {
    let refspec = format!("+refs/heads/{branch}:{remote_ref}");
    run_inherited(
        &ProcessSpec::new("git", ["fetch", "--no-tags", remote, &refspec]),
        root,
        child_stdout,
    )
    .map_err(CommandError::message)
}

fn action_against_remote(root: &Path, remote_ref: &str) -> Result<DeployAction, CommandError> {
    let comparison = format!("HEAD...{remote_ref}");
    let divergence = git(root, ["rev-list", "--left-right", "--count", &comparison])?;
    let (ahead, behind) = parse_divergence(&divergence)?;
    deployment_action(ahead, behind)
}

fn initial_push_action(root: &Path) -> Result<DeployAction, CommandError> {
    let commits = git(root, ["rev-list", "--count", "HEAD"])?
        .trim()
        .parse::<usize>()
        .map_err(|_| CommandError::Message("Git returned an invalid commit count".to_owned()))?;
    Ok(DeployAction::Push { commits })
}

fn verify_repository_root(root: &Path) -> Result<(), CommandError> {
    let reported = git(root, ["rev-parse", "--show-toplevel"])?;
    let reported = Path::new(reported.trim())
        .canonicalize()
        .map_err(CommandError::message)?;
    let expected = root.canonicalize().map_err(CommandError::message)?;
    if reported != expected {
        return Err(CommandError::Message(format!(
            "configuration directory `{}` is not the Git repository root `{}`",
            expected.display(),
            reported.display()
        )));
    }
    Ok(())
}

fn require_clean(root: &Path) -> Result<(), CommandError> {
    let status = git(
        root,
        ["status", "--porcelain=v1", "--untracked-files=normal"],
    )?;
    if status.trim().is_empty() {
        Ok(())
    } else {
        Err(CommandError::Message(format!(
            "deployment requires a clean Git worktree; commit or remove these changes:\n{}",
            status.trim_end()
        )))
    }
}

fn verify_workflow(root: &Path, relative: &str, branch: &str) -> Result<(), CommandError> {
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        CommandError::Message(format!(
            "deployment workflow `{}` is unavailable: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CommandError::Message(format!(
            "deployment workflow `{}` must be a regular file",
            path.display()
        )));
    }
    let source = fs::read_to_string(&path).map_err(CommandError::message)?;
    let workflow: serde_json::Value = serde_saphyr::from_str(&source).map_err(|error| {
        CommandError::Message(format!(
            "deployment workflow `{}` is invalid YAML: {error}",
            path.display()
        ))
    })?;
    let Some(triggers) = workflow.as_object().and_then(|mapping| mapping.get("on")) else {
        return Err(CommandError::Message(format!(
            "deployment workflow `{}` must declare an `on` trigger",
            path.display()
        )));
    };
    let (has_dispatch, has_push) = workflow_triggers(triggers, branch);
    if !has_dispatch || !has_push {
        return Err(CommandError::Message(format!(
            "deployment workflow `{}` must declare `workflow_dispatch` and a `push` trigger that includes branch `{branch}`",
            path.display(),
        )));
    }
    Ok(())
}

fn workflow_triggers(triggers: &serde_json::Value, branch: &str) -> (bool, bool) {
    match triggers {
        serde_json::Value::String(trigger) => (trigger == "workflow_dispatch", trigger == "push"),
        serde_json::Value::Array(triggers) => (
            triggers
                .iter()
                .any(|trigger| trigger.as_str() == Some("workflow_dispatch")),
            triggers
                .iter()
                .any(|trigger| trigger.as_str() == Some("push")),
        ),
        serde_json::Value::Object(triggers) => {
            let dispatch = triggers
                .get("workflow_dispatch")
                .is_some_and(valid_trigger_configuration);
            let push = triggers
                .get("push")
                .is_some_and(|configuration| push_includes_branch(configuration, branch));
            (dispatch, push)
        }
        _ => (false, false),
    }
}

fn valid_trigger_configuration(configuration: &serde_json::Value) -> bool {
    configuration.is_null() || configuration.is_object()
}

fn push_includes_branch(configuration: &serde_json::Value, branch: &str) -> bool {
    if configuration.is_null() {
        return true;
    }
    let Some(filters) = configuration.as_object() else {
        return false;
    };
    if filters.contains_key("paths") || filters.contains_key("paths-ignore") {
        return false;
    }
    if let Some(branches) = filters.get("branches") {
        return branch_patterns_include(branches, branch);
    }
    if let Some(ignored) = filters.get("branches-ignore") {
        return branch_patterns_exclude(ignored, branch);
    }
    !filters.contains_key("tags") && !filters.contains_key("tags-ignore")
}

fn branch_patterns_include(value: &serde_json::Value, branch: &str) -> bool {
    let Some(patterns) = branch_patterns(value) else {
        return false;
    };
    let mut included = false;
    for pattern in patterns {
        let (negative, pattern) = pattern
            .strip_prefix('!')
            .map_or((false, pattern), |pattern| (true, pattern));
        let Some(matches) = conservative_branch_match(pattern, branch) else {
            return false;
        };
        if matches {
            included = !negative;
        }
    }
    included
}

fn branch_patterns_exclude(value: &serde_json::Value, branch: &str) -> bool {
    let Some(patterns) = branch_patterns(value) else {
        return false;
    };
    patterns.into_iter().all(|pattern| {
        !pattern.starts_with('!') && conservative_branch_match(pattern, branch) == Some(false)
    })
}

fn branch_patterns(value: &serde_json::Value) -> Option<Vec<&str>> {
    match value {
        serde_json::Value::String(pattern) => Some(vec![pattern]),
        serde_json::Value::Array(patterns) => {
            patterns.iter().map(serde_json::Value::as_str).collect()
        }
        _ => None,
    }
}

fn conservative_branch_match(pattern: &str, branch: &str) -> Option<bool> {
    if pattern == "**" {
        return Some(true);
    }
    if pattern == "*" {
        return Some(!branch.contains('/'));
    }
    if let Some(prefix) = pattern
        .strip_suffix("/**")
        .filter(|prefix| !has_glob(prefix))
    {
        return Some(
            branch == prefix
                || branch
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/')),
        );
    }
    if let Some(prefix) = pattern
        .strip_suffix("/*")
        .filter(|prefix| !has_glob(prefix))
    {
        return Some(branch.strip_prefix(prefix).is_some_and(|rest| {
            rest.strip_prefix('/')
                .is_some_and(|rest| !rest.contains('/'))
        }));
    }
    (!has_glob(pattern)).then_some(pattern == branch)
}

fn has_glob(value: &str) -> bool {
    value
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
}

fn parse_divergence(value: &str) -> Result<(usize, usize), CommandError> {
    let mut fields = value.split_whitespace();
    let ahead = fields
        .next()
        .and_then(|field| field.parse().ok())
        .ok_or_else(|| CommandError::Message("Git returned an invalid ahead count".to_owned()))?;
    let behind = fields
        .next()
        .and_then(|field| field.parse().ok())
        .ok_or_else(|| CommandError::Message("Git returned an invalid behind count".to_owned()))?;
    if fields.next().is_some() {
        return Err(CommandError::Message(
            "Git returned an invalid divergence result".to_owned(),
        ));
    }
    Ok((ahead, behind))
}

fn deployment_action(ahead: usize, behind: usize) -> Result<DeployAction, CommandError> {
    if behind > 0 {
        Err(CommandError::Message(format!(
            "local branch is behind its deployment branch by {behind} commit(s); update it before deployment"
        )))
    } else if ahead > 0 {
        Ok(DeployAction::Push { commits: ahead })
    } else {
        Ok(DeployAction::Dispatch)
    }
}

fn github_repository(remote: &str) -> Option<String> {
    let remote = remote.trim().trim_end_matches('/');
    let path = remote
        .strip_prefix("git@github.com:")
        .or_else(|| remote.strip_prefix("ssh://git@github.com/"))
        .or_else(|| remote.strip_prefix("https://github.com/"))?;
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut segments = path.split('/');
    let owner = segments.next()?;
    let repository = segments.next()?;
    if owner.is_empty() || repository.is_empty() || segments.next().is_some() {
        return None;
    }
    Some(format!("{owner}/{repository}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_github_remote_urls() {
        for (remote, expected) in [
            (
                "git@github.com:ProjectMambo/MamboSite.git",
                "ProjectMambo/MamboSite",
            ),
            (
                "https://github.com/ProjectMambo/MamboSite.git",
                "ProjectMambo/MamboSite",
            ),
            (
                "ssh://git@github.com/ProjectMambo/MamboSite",
                "ProjectMambo/MamboSite",
            ),
        ] {
            assert_eq!(github_repository(remote).as_deref(), Some(expected));
        }
        assert_eq!(github_repository("git@gitlab.com:group/site.git"), None);
        assert_eq!(github_repository("https://github.com/a/b/extra"), None);
    }

    #[test]
    fn parses_git_divergence_counts() {
        assert_eq!(parse_divergence("3\t0\n").unwrap(), (3, 0));
        assert_eq!(parse_divergence("0 2").unwrap(), (0, 2));
        assert!(parse_divergence("unknown").is_err());
    }

    #[test]
    fn pushes_ahead_commits_and_dispatches_an_unchanged_head() {
        assert_eq!(
            deployment_action(2, 0).unwrap(),
            DeployAction::Push { commits: 2 }
        );
        assert_eq!(deployment_action(0, 0).unwrap(), DeployAction::Dispatch);
        assert!(deployment_action(1, 1).is_err());
    }

    #[test]
    fn requires_a_real_dispatchable_workflow() {
        let temporary = tempfile::tempdir().unwrap();
        let workflow = temporary.path().join(".github/workflows/pages.yml");
        fs::create_dir_all(workflow.parent().unwrap()).unwrap();
        fs::write(
            &workflow,
            "on:\n  push:\n    branches: [main]\n  workflow_dispatch:\n",
        )
        .unwrap();
        assert!(verify_workflow(temporary.path(), ".github/workflows/pages.yml", "main").is_ok());
        assert!(
            verify_workflow(temporary.path(), ".github/workflows/pages.yml", "release").is_err()
        );

        fs::write(&workflow, "on: [push, workflow_dispatch]\n").unwrap();
        assert!(
            verify_workflow(
                temporary.path(),
                ".github/workflows/pages.yml",
                "release/v1"
            )
            .is_ok()
        );

        fs::write(
            &workflow,
            "on:\n  push:\n    branches: release/**\n  workflow_dispatch: {}\n",
        )
        .unwrap();
        assert!(
            verify_workflow(
                temporary.path(),
                ".github/workflows/pages.yml",
                "release/v1"
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_textual_trigger_false_positives_and_tag_only_pushes() {
        let temporary = tempfile::tempdir().unwrap();
        let workflow = temporary.path().join(".github/workflows/pages.yml");
        fs::create_dir_all(workflow.parent().unwrap()).unwrap();

        fs::write(
            &workflow,
            "jobs:\n  push:\n    runs-on: ubuntu-latest\n  workflow_dispatch:\n    runs-on: ubuntu-latest\n",
        )
        .unwrap();
        assert!(verify_workflow(temporary.path(), ".github/workflows/pages.yml", "main").is_err());

        fs::write(
            &workflow,
            "on:\n  push:\n    tags: ['v*']\n  workflow_dispatch:\n",
        )
        .unwrap();
        assert!(verify_workflow(temporary.path(), ".github/workflows/pages.yml", "main").is_err());

        for path_filter in ["paths: ['docs/**']", "paths-ignore: ['README.md']"] {
            fs::write(
                &workflow,
                format!(
                    "on:\n  push:\n    branches: [main]\n    {path_filter}\n  workflow_dispatch:\n"
                ),
            )
            .unwrap();
            assert!(
                verify_workflow(temporary.path(), ".github/workflows/pages.yml", "main").is_err(),
                "push filter must not suppress deployment: {path_filter}"
            );
        }
    }

    fn run_git(root: &Path, args: &[&str]) {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn absent_remote_branch_uses_initial_push_and_explicit_fetch_updates_tracking_ref() {
        let temporary = tempfile::tempdir().unwrap();
        let remote = temporary.path().join("remote.git");
        let work = temporary.path().join("work");
        fs::create_dir(&work).unwrap();
        let remote_text = remote.to_str().unwrap();
        run_git(temporary.path(), &["init", "--bare", remote_text]);
        run_git(&work, &["init", "--initial-branch=main"]);
        run_git(&work, &["config", "user.name", "MamboSite Test"]);
        run_git(
            &work,
            &["config", "user.email", "mambosite@example.invalid"],
        );
        run_git(&work, &["remote", "add", "origin", remote_text]);

        assert!(!remote_branch_exists(&work, "origin", "main").unwrap());
        fs::write(work.join("README.md"), "test\n").unwrap();
        run_git(&work, &["add", "README.md"]);
        run_git(&work, &["commit", "-m", "initial"]);
        assert_eq!(
            initial_push_action(&work).unwrap(),
            DeployAction::Push { commits: 1 }
        );

        run_git(&work, &["push", "origin", "HEAD:refs/heads/main"]);
        run_git(&work, &["update-ref", "-d", "refs/remotes/origin/main"]);
        assert!(remote_branch_exists(&work, "origin", "main").unwrap());
        fetch_remote_branch(
            &work,
            "origin",
            "main",
            "refs/remotes/origin/main",
            ChildStdout::Stdout,
        )
        .unwrap();
        assert_eq!(
            git(&work, ["rev-parse", "HEAD"]).unwrap(),
            git(&work, ["rev-parse", "refs/remotes/origin/main"]).unwrap()
        );
    }
}
