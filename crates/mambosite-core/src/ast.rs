use serde::{Deserialize, Serialize};

use crate::{ParsedDirective, SourceSpan};

/// Renderer-neutral Markdown tree owned by `MamboSite`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownNode {
    #[serde(flatten)]
    pub kind: NodeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Self>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

/// Complete parser-level node union. Graph results currently sit beside this
/// tree; renderer-oriented lowering can replace or supplement it without
/// leaking Comrak types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum NodeKind {
    Document,
    FrontMatter {
        literal: String,
    },
    BlockQuote,
    List {
        kind: ListKind,
        start: usize,
        delimiter: ListDelimiter,
        tight: bool,
        is_task_list: bool,
    },
    ListItem,
    DescriptionList,
    DescriptionItem {
        tight: bool,
    },
    DescriptionTerm,
    DescriptionDetails,
    CodeBlock {
        literal: String,
        info: String,
        fenced: bool,
        closed: bool,
    },
    HtmlBlock {
        literal: String,
        block_type: u8,
    },
    Paragraph,
    Heading {
        level: u8,
        setext: bool,
    },
    ThematicBreak,
    FootnoteDefinition {
        name: String,
        total_references: u32,
    },
    Table {
        alignments: Vec<TableAlignment>,
    },
    TableRow {
        header: bool,
    },
    TableCell,
    Text {
        value: String,
    },
    TaskItem {
        checked: bool,
        marker: Option<char>,
    },
    SoftBreak,
    LineBreak,
    InlineCode {
        literal: String,
    },
    HtmlInline {
        literal: String,
    },
    Raw {
        literal: String,
    },
    Emphasis,
    Strong,
    Strikethrough,
    Highlight,
    Insert,
    Superscript,
    Link {
        destination: String,
        title: String,
    },
    Image {
        source: String,
        title: String,
    },
    FootnoteReference {
        name: String,
    },
    Math {
        literal: String,
        display: bool,
        dollar: bool,
    },
    MultilineBlockQuote {
        fence_length: usize,
    },
    Escaped,
    WikiLink {
        destination: String,
    },
    ObsidianEmbed {
        destination: String,
        option: Option<String>,
    },
    Underline,
    Subscript,
    SpoileredText,
    EscapedTag {
        tag: String,
    },
    Alert {
        kind: AlertKind,
        title: Option<String>,
    },
    Subtext,
    BlockDirective {
        info: String,
        fence_length: usize,
    },
    /// A `MamboSite` directive after the dialect-tokenization pass.
    Directive {
        invocation: ParsedDirective,
        #[serde(rename = "fenceLength", skip_serializing_if = "Option::is_none")]
        fence_length: Option<usize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ListKind {
    Bullet,
    Ordered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ListDelimiter {
    Period,
    Parenthesis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AlertKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}
