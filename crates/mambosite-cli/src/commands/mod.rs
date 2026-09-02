pub mod build;
pub mod deploy;
pub mod init;

use mambosite_core::Diagnostic;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("site validation failed")]
    Diagnostics(Vec<Diagnostic>),
    #[error("{0}")]
    Message(String),
}

impl CommandError {
    pub fn message(error: impl std::fmt::Display) -> Self {
        Self::Message(error.to_string())
    }
}
