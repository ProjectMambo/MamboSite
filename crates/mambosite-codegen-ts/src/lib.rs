//! Deterministic TypeScript code generation for validated `MamboSite` content.
//!
//! This crate deliberately accepts any serializable site model. That keeps the
//! writer dependent on the versioned wire representation rather than the
//! parser's internal Rust types.

mod serializer;
mod writer;

pub use serializer::{GeneratedFile, GeneratedSite, generate};
pub use writer::write;

use std::path::Path;

use serde::Serialize;

pub(crate) const OUTPUT_MARKER: &str = ".mambosite-generated";
pub(crate) const OUTPUT_MARKER_CONTENT: &str = "MamboSite generated output; schema=1\n";

/// Generate and atomically publish TypeScript modules under `output_dir`.
///
/// The source model must serialize to an object with a `pages` array. Every
/// page must have a unique string `id`; IDs determine stable module names and
/// output order.
///
/// # Errors
///
/// Returns an error when the wire model is invalid, serialization fails, or
/// the completed output tree cannot be published.
pub fn generate_to<T>(site: &T, output_dir: impl AsRef<Path>) -> Result<(), Error>
where
    T: Serialize,
{
    let generated = generate(site)?;
    write(&generated, output_dir.as_ref())
}

/// A code-generation or output publication failure.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not serialize the compiled site: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("the compiled site has an invalid shape: {0}")]
    InvalidSite(String),

    #[error("page id `{0}` occurs more than once")]
    DuplicatePageId(String),

    #[error("generated path `{0}` occurs more than once")]
    DuplicatePath(String),

    #[error("output directory `{0}` has no usable parent or file name")]
    InvalidOutputDirectory(String),

    #[error("refusing to replace unmanaged output path `{0}`")]
    UnmanagedOutputDirectory(String),

    #[error("could not publish generated output at `{output}`: {source}")]
    PublishFailed {
        output: String,
        #[source]
        source: std::io::Error,
    },

    #[error(
        "could not publish generated output at `{output}`: {publish}; the previous output was restored from `{backup}`"
    )]
    PublishFailedPreviousRestored {
        output: String,
        backup: String,
        #[source]
        publish: std::io::Error,
    },

    #[error(
        "could not publish generated output at `{output}`: {publish}; restoring the previous output from `{backup}` also failed: {rollback}"
    )]
    PublishAndRollbackFailed {
        output: String,
        backup: String,
        publish: std::io::Error,
        #[source]
        rollback: std::io::Error,
    },

    #[error(
        "published generated output at `{output}`, but could not remove the previous-output backup `{backup}`: {cleanup}"
    )]
    PublishedButBackupCleanupFailed {
        output: String,
        backup: String,
        #[source]
        cleanup: Box<Self>,
    },

    #[error("could not {action} `{path}`: {source}")]
    Io {
        action: &'static str,
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    pub(crate) fn io(action: &'static str, path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            action,
            path: path.display().to_string(),
            source,
        }
    }
}
