use comrak::nodes::{
    AlertType, AstNode, ListDelimType, ListType, NodeValue, TableAlignment as ComrakTableAlignment,
};
use comrak::{Arena, Options, parse_document};

use crate::{
    AlertKind, ListDelimiter, ListKind, MarkdownNode, NodeKind, SourcePosition, SourceSpan,
    TableAlignment,
};

/// Adapter boundary for Markdown engines.
pub trait MarkdownParser {
    /// Parse a body whose first line occurs after `line_offset` source lines.
    fn parse(&self, source: &str, line_offset: usize) -> MarkdownNode;
}

/// CommonMark/GFM parser with the documented `MamboSite` syntax extensions.
#[derive(Debug, Clone)]
pub struct ComrakMarkdownParser {
    options: Options<'static>,
}

impl Default for ComrakMarkdownParser {
    fn default() -> Self {
        let mut options = Options::default();
        options.extension.strikethrough = true;
        options.extension.tagfilter = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.footnotes = true;
        options.extension.inline_footnotes = true;
        options.extension.description_lists = true;
        options.extension.multiline_block_quotes = true;
        options.extension.alerts = true;
        options.extension.math_dollars = true;
        options.extension.math_latex = true;
        options.extension.math_code = true;
        options.extension.wikilinks_title_after_pipe = true;
        options.extension.highlight = true;
        options.extension.insert = true;
        options.extension.subtext = true;
        options.extension.block_directive = true;
        options.parse.tasklist_in_table = true;
        options.parse.leave_footnote_definitions = true;
        options.parse.escaped_char_spans = true;
        options.parse.sourcepos_chars = true;
        Self { options }
    }
}

impl MarkdownParser for ComrakMarkdownParser {
    fn parse(&self, source: &str, line_offset: usize) -> MarkdownNode {
        let arena = Arena::new();
        let root = parse_document(&arena, source, &self.options);
        let source_index = SourceIndex::new(source);
        convert_node(root, line_offset, &source_index)
    }
}

fn convert_node<'arena>(
    node: &'arena AstNode<'arena>,
    line_offset: usize,
    source_index: &SourceIndex<'_>,
) -> MarkdownNode {
    let data = node.data();
    let kind = convert_kind(&data.value);
    let span = convert_span(data.sourcepos, line_offset, source_index);
    drop(data);
    let children = node
        .children()
        .map(|child| convert_node(child, line_offset, source_index))
        .collect();
    MarkdownNode {
        kind,
        span,
        children,
        block_id: None,
    }
}

#[allow(clippy::too_many_lines)]
fn convert_kind(value: &NodeValue) -> NodeKind {
    match value {
        NodeValue::Document => NodeKind::Document,
        NodeValue::FrontMatter(literal) => NodeKind::FrontMatter {
            literal: literal.clone(),
        },
        NodeValue::BlockQuote => NodeKind::BlockQuote,
        NodeValue::List(list) => NodeKind::List {
            kind: match list.list_type {
                ListType::Bullet => ListKind::Bullet,
                ListType::Ordered => ListKind::Ordered,
            },
            start: list.start,
            delimiter: match list.delimiter {
                ListDelimType::Period => ListDelimiter::Period,
                ListDelimType::Paren => ListDelimiter::Parenthesis,
            },
            tight: list.tight,
            is_task_list: list.is_task_list,
        },
        NodeValue::Item(_) => NodeKind::ListItem,
        NodeValue::DescriptionList => NodeKind::DescriptionList,
        NodeValue::DescriptionItem(item) => NodeKind::DescriptionItem { tight: item.tight },
        NodeValue::DescriptionTerm => NodeKind::DescriptionTerm,
        NodeValue::DescriptionDetails => NodeKind::DescriptionDetails,
        NodeValue::CodeBlock(block) => NodeKind::CodeBlock {
            literal: block.literal.clone(),
            info: block.info.clone(),
            fenced: block.fenced,
            closed: block.closed,
        },
        NodeValue::HtmlBlock(block) => NodeKind::HtmlBlock {
            literal: block.literal.clone(),
            block_type: block.block_type,
        },
        NodeValue::Paragraph => NodeKind::Paragraph,
        NodeValue::Heading(heading) => NodeKind::Heading {
            level: heading.level,
            setext: heading.setext,
        },
        NodeValue::ThematicBreak => NodeKind::ThematicBreak,
        NodeValue::FootnoteDefinition(footnote) => NodeKind::FootnoteDefinition {
            name: footnote.name.clone(),
            total_references: footnote.total_references,
        },
        NodeValue::Table(table) => NodeKind::Table {
            alignments: table
                .alignments
                .iter()
                .map(|alignment| match alignment {
                    ComrakTableAlignment::None => TableAlignment::None,
                    ComrakTableAlignment::Left => TableAlignment::Left,
                    ComrakTableAlignment::Center => TableAlignment::Center,
                    ComrakTableAlignment::Right => TableAlignment::Right,
                })
                .collect(),
        },
        NodeValue::TableRow(header) => NodeKind::TableRow { header: *header },
        NodeValue::TableCell => NodeKind::TableCell,
        NodeValue::Text(value) => NodeKind::Text {
            value: value.to_string(),
        },
        NodeValue::TaskItem(item) => NodeKind::TaskItem {
            checked: item.symbol.is_some(),
            marker: item.symbol,
        },
        NodeValue::SoftBreak => NodeKind::SoftBreak,
        NodeValue::LineBreak => NodeKind::LineBreak,
        NodeValue::Code(code) => NodeKind::InlineCode {
            literal: code.literal.clone(),
        },
        NodeValue::HtmlInline(literal) => NodeKind::HtmlInline {
            literal: literal.clone(),
        },
        NodeValue::Raw(literal) => NodeKind::Raw {
            literal: literal.clone(),
        },
        NodeValue::Emph => NodeKind::Emphasis,
        NodeValue::Strong => NodeKind::Strong,
        NodeValue::Strikethrough => NodeKind::Strikethrough,
        NodeValue::Highlight => NodeKind::Highlight,
        NodeValue::Insert => NodeKind::Insert,
        NodeValue::Superscript => NodeKind::Superscript,
        NodeValue::Link(link) => NodeKind::Link {
            destination: link.url.clone(),
            title: link.title.clone(),
        },
        NodeValue::Image(image) => NodeKind::Image {
            source: image.url.clone(),
            title: image.title.clone(),
        },
        NodeValue::FootnoteReference(reference) => NodeKind::FootnoteReference {
            name: reference.name.clone(),
        },
        NodeValue::Math(math) => NodeKind::Math {
            literal: math.literal.clone(),
            display: math.display_math,
            dollar: math.dollar_math,
        },
        NodeValue::MultilineBlockQuote(quote) => NodeKind::MultilineBlockQuote {
            fence_length: quote.fence_length,
        },
        NodeValue::Escaped => NodeKind::Escaped,
        NodeValue::WikiLink(link) => NodeKind::WikiLink {
            destination: link.url.clone(),
        },
        NodeValue::Underline => NodeKind::Underline,
        NodeValue::Subscript => NodeKind::Subscript,
        NodeValue::SpoileredText => NodeKind::SpoileredText,
        NodeValue::EscapedTag(tag) => NodeKind::EscapedTag {
            tag: (*tag).to_owned(),
        },
        NodeValue::Alert(alert) => NodeKind::Alert {
            kind: match alert.alert_type {
                AlertType::Note => AlertKind::Note,
                AlertType::Tip => AlertKind::Tip,
                AlertType::Important => AlertKind::Important,
                AlertType::Warning => AlertKind::Warning,
                AlertType::Caution => AlertKind::Caution,
            },
            title: alert.title.clone(),
        },
        NodeValue::Subtext => NodeKind::Subtext,
        NodeValue::BlockDirective(directive) => NodeKind::BlockDirective {
            info: directive.info.clone(),
            fence_length: directive.fence_length,
        },
    }
}

fn convert_span(
    source: comrak::nodes::Sourcepos,
    line_offset: usize,
    source_index: &SourceIndex<'_>,
) -> Option<SourceSpan> {
    if source.start.line == 0 || source.end.line == 0 {
        return None;
    }
    let (start_byte, end_byte) = source_index.byte_span(source)?;
    let mut start = source_index.position_at(start_byte)?;
    let mut end = source_index.position_at(end_byte)?;
    start.line += line_offset;
    end.line += line_offset;
    Some(SourceSpan {
        start,
        end,
        start_byte: Some(start_byte),
        end_byte: Some(end_byte),
    })
}

struct SourceIndex<'source> {
    source: &'source str,
    line_starts: Vec<usize>,
}

impl<'source> SourceIndex<'source> {
    fn new(source: &'source str) -> Self {
        let mut line_starts = vec![0];
        let bytes = source.as_bytes();
        let mut cursor = 0;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'\n' => {
                    cursor += 1;
                    line_starts.push(cursor);
                }
                b'\r' => {
                    cursor += 1;
                    if bytes.get(cursor) == Some(&b'\n') {
                        cursor += 1;
                    }
                    line_starts.push(cursor);
                }
                _ => cursor += 1,
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    fn byte_span(&self, source: comrak::nodes::Sourcepos) -> Option<(usize, usize)> {
        let start = self.byte_at(source.start.line, source.start.column, false)?;
        let end = self.byte_at(source.end.line, source.end.column, true)?;
        Some((start, end.max(start)))
    }

    fn byte_at(&self, line: usize, column: usize, after_character: bool) -> Option<usize> {
        let line_start = *self.line_starts.get(line.checked_sub(1)?)?;
        let line_end = self
            .line_starts
            .get(line)
            .copied()
            .unwrap_or(self.source.len());
        let line_source = self.source.get(line_start..line_end)?;
        let (relative, character) = line_source.char_indices().nth(column.checked_sub(1)?)?;
        Some(line_start + relative + usize::from(after_character) * character.len_utf8())
    }

    fn position_at(&self, byte: usize) -> Option<SourcePosition> {
        if byte > self.source.len() || !self.source.is_char_boundary(byte) {
            return None;
        }
        let line_index = self
            .line_starts
            .partition_point(|line_start| *line_start <= byte)
            .saturating_sub(1);
        let line_start = *self.line_starts.get(line_index)?;
        Some(SourcePosition {
            line: line_index + 1,
            column: self.source.get(line_start..byte)?.chars().count() + 1,
        })
    }
}

pub(crate) fn plain_text(node: &MarkdownNode) -> String {
    let mut output = String::new();
    append_plain_text(node, &mut output);
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn append_plain_text(node: &MarkdownNode, output: &mut String) {
    match &node.kind {
        NodeKind::Text { value }
        | NodeKind::InlineCode { literal: value }
        | NodeKind::Math { literal: value, .. } => output.push_str(value),
        NodeKind::SoftBreak | NodeKind::LineBreak => output.push(' '),
        NodeKind::Image { .. } => {
            for child in &node.children {
                append_plain_text(child, output);
            }
            return;
        }
        _ => {}
    }
    for child in &node.children {
        append_plain_text(child, output);
    }
}

#[cfg(test)]
mod tests {
    use super::{ComrakMarkdownParser, MarkdownParser};
    use crate::{MarkdownNode, NodeKind};

    fn contains(node: &MarkdownNode, predicate: impl Copy + Fn(&NodeKind) -> bool) -> bool {
        predicate(&node.kind) || node.children.iter().any(|child| contains(child, predicate))
    }

    #[test]
    fn parses_commonmark_gfm_and_documented_extensions() {
        let source = concat!(
            "# Full *Markdown*\n\n",
            "- [x] task\n\n",
            "| a | b |\n| :- | -: |\n| 1 | 2 |\n\n",
            "~~gone~~ ==marked== [link](https://example.com) [[Page|Wiki]]\n\n",
            "```rust\nfn main() {}\n```\n\n",
            "> [!NOTE]\n> alert\n\n",
            "A footnote[^1] and $x^2$.\n\n[^1]: detail\n",
        );
        let tree = ComrakMarkdownParser::default().parse(source, 0);
        assert!(contains(&tree, |kind| matches!(
            kind,
            NodeKind::Table { .. }
        )));
        assert!(contains(&tree, |kind| matches!(
            kind,
            NodeKind::TaskItem { checked: true, .. }
        )));
        assert!(contains(
            &tree,
            |kind| matches!(kind, NodeKind::CodeBlock { info, .. } if info == "rust")
        ));
        assert!(contains(&tree, |kind| matches!(
            kind,
            NodeKind::Alert { .. }
        )));
        assert!(contains(&tree, |kind| matches!(
            kind,
            NodeKind::WikiLink { .. }
        )));
        assert!(contains(&tree, |kind| matches!(
            kind,
            NodeKind::FootnoteDefinition { .. }
        )));
        assert!(contains(&tree, |kind| matches!(
            kind,
            NodeKind::Math { .. }
        )));
    }

    #[test]
    fn offsets_spans_past_frontmatter() {
        let tree = ComrakMarkdownParser::default().parse("# Heading\n", 5);
        let span = tree.children[0].span.expect("heading span");
        assert_eq!(span.start.line, 6);
        assert_eq!(span.start.column, 1);
        assert_eq!(span.end.column, 10);
        assert_eq!(span.start_byte, Some(0));
        assert_eq!(span.end_byte, Some("# Heading".len()));
    }

    #[test]
    fn retains_core_block_and_inline_semantics() {
        let source = concat!(
            "1. ordered\n   - nested\n\n",
            "> quote with **strong**, *emphasis*, and `code`  \n> hard break\n\n",
            "![alternative](image.png \"title\")\n\n",
            "<span>raw inline</span>\n\n",
            "<div>raw block</div>\n\n",
            "Term\n\n: description\n\n",
            "---\n",
        );
        let tree = ComrakMarkdownParser::default().parse(source, 0);
        for expected in [
            "list",
            "blockQuote",
            "strong",
            "emphasis",
            "inlineCode",
            "lineBreak",
            "image",
            "htmlInline",
            "htmlBlock",
            "descriptionList",
            "thematicBreak",
        ] {
            let value = serde_json::to_value(&tree).unwrap();
            assert!(
                value
                    .to_string()
                    .contains(&format!("\"type\":\"{expected}\"")),
                "missing {expected} in {value}"
            );
        }
    }

    #[test]
    fn byte_spans_count_multibyte_utf8() {
        let tree = ComrakMarkdownParser::default().parse("# 好x\n", 0);
        let heading = &tree.children[0];
        assert_eq!(heading.span.unwrap().start_byte, Some(0));
        assert_eq!(heading.span.unwrap().end_byte, Some("# 好x".len()));
        let text = &heading.children[0];
        assert_eq!(text.span.unwrap().start_byte, Some(2));
        assert_eq!(text.span.unwrap().end_byte, Some("# 好x".len()));
    }

    #[test]
    fn byte_spans_support_lone_cr_line_endings() {
        let tree = ComrakMarkdownParser::default().parse("# One\r\r# Two\r", 0);
        let second = &tree.children[1];
        let span = second.span.unwrap();
        assert_eq!(span.start.line, 3);
        assert_eq!(span.start_byte, Some("# One\r\r".len()));
        assert_eq!(span.end_byte, Some("# One\r\r# Two".len()));
    }
}
