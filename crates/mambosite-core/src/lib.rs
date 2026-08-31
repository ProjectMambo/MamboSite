//! Core parsing and content-model APIs for `MamboSite`.
//!
//! The crate deliberately owns every public content type. Parser-library nodes,
//! filesystem paths, and renderer-specific values stop at their adapter
//! boundaries.

mod ast;
mod compiler;
mod config;
mod diagnostic;
mod dialect;
mod directive;
mod directive_registry;
mod frontmatter;
mod markdown;
mod model;
mod reference;
mod route;
mod source;

pub use ast::{AlertKind, ListDelimiter, ListKind, MarkdownNode, NodeKind, TableAlignment};
pub use compiler::{CompileOutcome, Compiler};
pub use config::{Config, FrontmatterConfig, MarkdownConfig, SiteConfig};
pub use diagnostic::{Diagnostic, Severity, SourceLocation, SourcePosition, SourceSpan};
pub use directive::{
    DirectiveDiagnostic, DirectiveForm, DirectiveParseOutcome, DirectiveProperty, DirectiveScalar,
    DirectiveSpan, DirectiveValue, ParsedDirective, parse_container_directive_info,
    parse_leaf_directive,
};
pub use directive_registry::{
    DirectiveValidationContext, DirectiveValidationOutcome, ValidatedDirective, validate_directives,
};
pub use frontmatter::{FrontmatterOutcome, parse_frontmatter};
pub use markdown::{ComrakMarkdownParser, MarkdownParser};
pub use model::{
    BlockRecord, HeadingRecord, Mount, Page, PageMetadata, PageStatus, ReferenceSyntax,
    ResolvedEmbed, ResolvedFragment, ResolvedLink, ResolvedLinkTarget, Site, SiteMetadata,
};
pub use route::{derive_route, normalize_route, slugify_segment};

/// Schema shared by the compiler output and TypeScript runtime.
pub const SCHEMA_VERSION: u32 = 1;
