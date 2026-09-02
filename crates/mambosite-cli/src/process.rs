use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildStdout {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProcessSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl ProcessSpec {
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            env: Vec::new(),
        }
    }

    pub fn with_env(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((name.into(), value.into()));
        self
    }

    pub fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .map(quote_argument)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("could not start `{command}`: {source}")]
    Start {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not forward stdout from `{command}` to stderr: {source}")]
    Forward {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not wait for `{command}`: {source}")]
    Wait {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("`{command}` failed with {status}{details}")]
    Exit {
        command: String,
        status: String,
        details: String,
    },
    #[error("`{command}` wrote non-UTF-8 output")]
    NonUtf8 { command: String },
}

pub fn run_inherited(
    spec: &ProcessSpec,
    cwd: &Path,
    stdout: ChildStdout,
) -> Result<(), ProcessError> {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .envs(spec.env.iter().map(|(name, value)| (name, value)))
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(match stdout {
            ChildStdout::Stdout => Stdio::inherit(),
            ChildStdout::Stderr => Stdio::piped(),
        })
        .stderr(Stdio::inherit());
    let mut child = command.spawn().map_err(|source| ProcessError::Start {
        command: spec.display(),
        source,
    })?;

    let stream_result = if stdout == ChildStdout::Stderr {
        let mut child_stdout = child.stdout.take().expect("piped child stdout");
        std::io::copy(&mut child_stdout, &mut std::io::stderr().lock())
            .map(|_| ())
            .map_err(|source| ProcessError::Forward {
                command: spec.display(),
                source,
            })
    } else {
        Ok(())
    };
    let status = child.wait().map_err(|source| ProcessError::Wait {
        command: spec.display(),
        source,
    })?;
    stream_result?;

    if status.success() {
        Ok(())
    } else {
        Err(ProcessError::Exit {
            command: spec.display(),
            status: status_label(status.code()),
            details: String::new(),
        })
    }
}

pub fn capture(spec: &ProcessSpec, cwd: &Path) -> Result<String, ProcessError> {
    let output = Command::new(&spec.program)
        .args(&spec.args)
        .envs(spec.env.iter().map(|(name, value)| (name, value)))
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .map_err(|source| ProcessError::Start {
            command: spec.display(),
            source,
        })?;

    let stdout = String::from_utf8(output.stdout).map_err(|_| ProcessError::NonUtf8 {
        command: spec.display(),
    })?;
    let stderr = String::from_utf8(output.stderr).map_err(|_| ProcessError::NonUtf8 {
        command: spec.display(),
    })?;
    if output.status.success() {
        Ok(stdout)
    } else {
        let details = if stderr.trim().is_empty() {
            String::new()
        } else {
            format!(": {}", stderr.trim())
        };
        Err(ProcessError::Exit {
            command: spec.display(),
            status: status_label(output.status.code()),
            details,
        })
    }
}

fn status_label(code: Option<i32>) -> String {
    code.map_or_else(|| "a signal".to_owned(), |code| format!("exit code {code}"))
}

fn quote_argument(value: &str) -> String {
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-._/:=@".contains(character))
    {
        value.to_owned()
    } else {
        format!("{value:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_commands_for_diagnostics_without_a_shell() {
        let spec = ProcessSpec::new("npm", ["run", "site:build", "two words"]);
        assert_eq!(spec.display(), "npm run site:build \"two words\"");
    }
}
