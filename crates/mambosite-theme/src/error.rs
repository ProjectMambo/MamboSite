use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeDiagnostic {
    pub code: &'static str,
    pub field: String,
    pub message: String,
}

impl ThemeDiagnostic {
    pub(crate) fn new(
        code: &'static str,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            field: field.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("could not read theme configuration `{path}`: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid theme TOML in `{path}`: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("theme configuration `{path}` failed validation")]
    Validation {
        path: PathBuf,
        diagnostics: Vec<ThemeDiagnostic>,
    },
}

impl ThemeError {
    pub fn diagnostics(&self) -> &[ThemeDiagnostic] {
        match self {
            Self::Validation { diagnostics, .. } => diagnostics,
            Self::Read { .. } | Self::Parse { .. } => &[],
        }
    }
}
