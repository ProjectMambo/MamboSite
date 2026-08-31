use serde::{Deserialize, Serialize};

/// Diagnostic importance. Errors prevent generation; warnings and notes do not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// A one-based location in a UTF-8 source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

/// A half-open source range. `start` is inclusive and `end` is exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
    /// Zero-based byte offset into the original UTF-8 source, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_byte: Option<usize>,
    /// Exclusive zero-based byte offset into the original UTF-8 source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_byte: Option<usize>,
}

impl SourceSpan {
    pub const fn point(line: usize, column: usize) -> Self {
        let point = SourcePosition { line, column };
        Self {
            start: point,
            end: point,
            start_byte: None,
            end_byte: None,
        }
    }

    #[must_use]
    pub const fn with_bytes(mut self, start_byte: usize, end_byte: usize) -> Self {
        self.start_byte = Some(start_byte);
        self.end_byte = Some(end_byte);
        self
    }
}

/// A logical content-root-relative source location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceLocation {
    pub path: String,
    pub span: SourceSpan,
}

/// A stable, serializable compiler diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<SourceLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl Diagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code, message)
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code, message)
    }

    pub fn note(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Note, code, message)
    }

    fn new(severity: Severity, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity,
            code: code.into(),
            message: message.into(),
            primary: None,
            related: Vec::new(),
            help: None,
            notes: Vec::new(),
        }
    }

    #[must_use]
    pub fn at(mut self, path: impl Into<String>, span: SourceSpan) -> Self {
        self.primary = Some(SourceLocation {
            path: path.into(),
            span,
        });
        self
    }

    #[must_use]
    pub fn at_path(self, path: impl Into<String>) -> Self {
        self.at(path, SourceSpan::point(1, 1))
    }

    #[must_use]
    pub fn with_related(mut self, path: impl Into<String>, span: SourceSpan) -> Self {
        self.related.push(SourceLocation {
            path: path.into(),
            span,
        });
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub const fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }
}
