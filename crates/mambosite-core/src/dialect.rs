use crate::{
    Diagnostic, DirectiveDiagnostic, MarkdownNode, NodeKind, SourcePosition, SourceSpan,
    parse_container_directive_info, parse_leaf_directive,
};

/// Lower MamboSite-specific directive syntax retained by the Markdown parser.
///
/// This pass owns syntax recognition only. Registry validation (known names,
/// properties, defaults, and contexts) is a separate semantic stage.
pub(crate) fn lower_directives(
    root: &mut MarkdownNode,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    lower_node(
        root,
        body,
        body_start_line,
        body_start_byte,
        path,
        diagnostics,
    );
}

#[derive(Default)]
struct ObsidianState {
    comment_start: Option<SourceSpan>,
}

/// Lower Obsidian comments, embeds, and block identifiers retained as plain
/// `CommonMark` text. Code and directive invocations are protected, while the
/// Markdown children of container directives still participate in lowering.
pub(crate) fn lower_obsidian(
    root: &mut MarkdownNode,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut state = ObsidianState::default();
    process_obsidian_children(
        &mut root.children,
        &mut state,
        body,
        body_start_line,
        body_start_byte,
        path,
        diagnostics,
    );
    if let Some(span) = state.comment_start {
        diagnostics.push(
            Diagnostic::error("MS3305", "Obsidian comment is missing its closing `%%`")
                .at(path, span),
        );
    }
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn process_obsidian_children(
    children: &mut Vec<MarkdownNode>,
    state: &mut ObsidianState,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let original = std::mem::take(children);
    let mut output = Vec::new();
    for mut child in original {
        if matches!(child.kind, NodeKind::Text { .. }) {
            output.extend(process_obsidian_text(
                child,
                state,
                body,
                body_start_line,
                body_start_byte,
                path,
                diagnostics,
            ));
            continue;
        }

        if matches!(child.kind, NodeKind::Directive { .. }) {
            let comment_was_open = state.comment_start.is_some();
            process_obsidian_children(
                &mut child.children,
                state,
                body,
                body_start_line,
                body_start_byte,
                path,
                diagnostics,
            );
            // The invocation is syntax, not authored Markdown content, so it
            // must not be scanned for Obsidian markers. A container that begins
            // inside an open comment remains hidden unless that comment closes
            // within the container; otherwise preserve the container itself,
            // even when removing comments leaves it with no children.
            if !(comment_was_open && state.comment_start.is_some()) {
                output.push(child);
            }
            continue;
        }

        let protected = matches!(
            child.kind,
            NodeKind::CodeBlock { .. }
                | NodeKind::InlineCode { .. }
                | NodeKind::HtmlBlock { .. }
                | NodeKind::HtmlInline { .. }
                | NodeKind::Raw { .. }
        );
        if protected {
            if state.comment_start.is_none() {
                output.push(child);
            }
            continue;
        }

        let had_children = !child.children.is_empty();
        let comment_was_open = state.comment_start.is_some();
        process_obsidian_children(
            &mut child.children,
            state,
            body,
            body_start_line,
            body_start_byte,
            path,
            diagnostics,
        );

        if let Some(marker) = standalone_block_marker(&child) {
            match marker {
                Ok((id, span)) => {
                    if let Some(previous) = output.last_mut() {
                        if let Some(existing) = &previous.block_id {
                            let mut diagnostic = Diagnostic::error(
                                "MS3309",
                                format!(
                                    "preceding block already has identifier `^{existing}` and cannot also use `^{id}`"
                                ),
                            )
                            .at(path, span);
                            if let Some(previous_span) = previous.span {
                                diagnostic = diagnostic.with_related(path, previous_span);
                            }
                            diagnostics.push(diagnostic);
                        } else {
                            previous.block_id = Some(id);
                        }
                    } else {
                        diagnostics.push(
                            Diagnostic::error(
                                "MS3307",
                                "block identifier has no preceding block to identify",
                            )
                            .at(path, span),
                        );
                    }
                }
                Err((marker, span)) => diagnostics.push(
                    Diagnostic::error(
                        "MS3307",
                        format!("invalid Obsidian block identifier `{marker}`"),
                    )
                    .at(path, span),
                ),
            }
            continue;
        }

        extract_trailing_block_id(&mut child, path, diagnostics);
        let emptied_by_comment = had_children && child.children.is_empty();
        if !(emptied_by_comment || comment_was_open && state.comment_start.is_some()) {
            output.push(child);
        }
    }
    *children = output;
}

#[allow(clippy::too_many_arguments)]
fn process_obsidian_text(
    node: MarkdownNode,
    state: &mut ObsidianState,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MarkdownNode> {
    let NodeKind::Text { value } = &node.kind else {
        return vec![node];
    };
    let value = value.clone();
    let mut output = Vec::new();
    let mut cursor = 0;
    let span_context = SpanContext {
        body,
        body_start_line,
        body_start_byte,
    };
    while cursor < value.len() {
        if state.comment_start.is_some() {
            let Some(relative_end) = value[cursor..].find("%%") else {
                return output;
            };
            cursor += relative_end + 2;
            state.comment_start = None;
            continue;
        }

        let comment = value[cursor..].find("%%").map(|offset| cursor + offset);
        let embed = value[cursor..].find("![[").map(|offset| cursor + offset);
        let next = match (comment, embed) {
            (Some(comment), Some(embed)) => Some(comment.min(embed)),
            (Some(comment), None) => Some(comment),
            (None, Some(embed)) => Some(embed),
            (None, None) => None,
        };
        let Some(marker) = next else {
            push_text_piece(
                &mut output,
                &node,
                &value,
                cursor,
                value.len(),
                span_context,
            );
            break;
        };
        push_text_piece(&mut output, &node, &value, cursor, marker, span_context);

        if value[marker..].starts_with("%%") {
            let marker_span = span_context.subspan(node.span, marker, marker + 2);
            state.comment_start = Some(marker_span);
            cursor = marker + 2;
            continue;
        }

        let content_start = marker + 3;
        let Some(relative_end) = value[content_start..].find("]]") else {
            let span = span_context.subspan(node.span, marker, value.len());
            diagnostics.push(
                Diagnostic::error("MS3306", "Obsidian embed is missing its closing `]]`")
                    .at(path, span),
            );
            push_text_piece(
                &mut output,
                &node,
                &value,
                marker,
                value.len(),
                span_context,
            );
            break;
        };
        let end = content_start + relative_end + 2;
        let inner = &value[content_start..content_start + relative_end];
        let (destination, option) = inner
            .split_once('|')
            .map_or((inner, None), |(target, option)| {
                (target, Some(option.trim().to_owned()))
            });
        let destination = destination.trim();
        let span = span_context.subspan(node.span, marker, end);
        if destination.is_empty() {
            diagnostics.push(
                Diagnostic::error("MS3306", "Obsidian embed target cannot be empty").at(path, span),
            );
        } else {
            output.push(MarkdownNode {
                kind: NodeKind::ObsidianEmbed {
                    destination: destination.to_owned(),
                    option,
                },
                span: Some(span),
                children: Vec::new(),
                block_id: None,
            });
        }
        cursor = end;
    }
    output
}

#[derive(Clone, Copy)]
struct SpanContext<'source> {
    body: &'source str,
    body_start_line: usize,
    body_start_byte: usize,
}

impl SpanContext<'_> {
    fn subspan(self, parent: Option<SourceSpan>, start: usize, end: usize) -> SourceSpan {
        subspan(
            parent,
            self.body,
            self.body_start_line,
            self.body_start_byte,
            start,
            end,
        )
    }
}

fn push_text_piece(
    output: &mut Vec<MarkdownNode>,
    source: &MarkdownNode,
    value: &str,
    start: usize,
    end: usize,
    context: SpanContext<'_>,
) {
    if start == end {
        return;
    }
    output.push(MarkdownNode {
        kind: NodeKind::Text {
            value: value[start..end].to_owned(),
        },
        span: source
            .span
            .map(|_| context.subspan(source.span, start, end)),
        children: Vec::new(),
        block_id: None,
    });
}

fn standalone_block_marker(
    node: &MarkdownNode,
) -> Option<Result<(String, SourceSpan), (String, SourceSpan)>> {
    if !matches!(node.kind, NodeKind::Paragraph) || node.children.len() != 1 {
        return None;
    }
    let child = &node.children[0];
    let NodeKind::Text { value } = &child.kind else {
        return None;
    };
    let marker = value.trim();
    if !marker.starts_with('^') || marker.chars().any(char::is_whitespace) {
        return None;
    }
    let span = child.span.or(node.span)?;
    validate_block_id(marker)
        .map(|id| (id, span))
        .map_err(|()| (marker.to_owned(), span))
        .into()
}

fn extract_trailing_block_id(
    node: &mut MarkdownNode,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !matches!(node.kind, NodeKind::Paragraph) {
        return;
    }
    let Some(last) = node.children.last_mut() else {
        return;
    };
    let NodeKind::Text { value } = &mut last.kind else {
        return;
    };
    let trimmed = value.trim_end();
    let token_start = trimmed.rfind(char::is_whitespace).map_or(0, |position| {
        position + trimmed[position..].chars().next().map_or(0, char::len_utf8)
    });
    let marker = &trimmed[token_start..];
    if !marker.starts_with('^') {
        return;
    }
    let span = last
        .span
        .or(node.span)
        .unwrap_or_else(|| SourceSpan::point(1, 1));
    match validate_block_id(marker) {
        Ok(id) => {
            node.block_id = Some(id);
            value.truncate(token_start);
            *value = value.trim_end().to_owned();
            if value.is_empty() {
                node.children.pop();
            }
        }
        Err(()) => diagnostics.push(
            Diagnostic::error(
                "MS3307",
                format!("invalid Obsidian block identifier `{marker}`"),
            )
            .at(path, span),
        ),
    }
}

fn validate_block_id(marker: &str) -> Result<String, ()> {
    let id = marker.strip_prefix('^').ok_or(())?;
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(());
    }
    Ok(id.to_owned())
}

fn subspan(
    parent: Option<SourceSpan>,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    relative_start: usize,
    relative_end: usize,
) -> SourceSpan {
    let Some(parent) = parent else {
        return SourceSpan::point(body_start_line, 1);
    };
    let parent_start = parent.start_byte.unwrap_or(body_start_byte);
    source_span(
        body,
        body_start_line,
        body_start_byte,
        parent_start + relative_start,
        parent_start + relative_end,
    )
}

fn lower_node(
    node: &mut MarkdownNode,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let replacement = match &node.kind {
        NodeKind::Paragraph => lower_leaf(
            node.span,
            body,
            body_start_line,
            body_start_byte,
            path,
            diagnostics,
        ),
        NodeKind::BlockDirective { info, fence_length } => lower_container(
            node.span,
            info,
            *fence_length,
            body,
            body_start_line,
            body_start_byte,
            path,
            diagnostics,
        ),
        _ => None,
    };
    if let Some(kind) = replacement {
        let is_leaf = matches!(
            &kind,
            NodeKind::Directive {
                invocation,
                fence_length: None,
            } if invocation.form == crate::DirectiveForm::Leaf
        );
        node.kind = kind;
        if is_leaf {
            node.children.clear();
        }
    }

    for child in &mut node.children {
        lower_node(
            child,
            body,
            body_start_line,
            body_start_byte,
            path,
            diagnostics,
        );
    }
}

fn lower_leaf(
    span: Option<SourceSpan>,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<NodeKind> {
    let (snippet, start_byte) = source_slice(span?, body, body_start_byte)?;
    let outcome = parse_leaf_directive(snippet, start_byte);
    if !outcome.recognized {
        return None;
    }
    push_diagnostics(
        outcome.diagnostics,
        body,
        body_start_line,
        body_start_byte,
        path,
        diagnostics,
    );
    outcome.directive.map(|invocation| NodeKind::Directive {
        invocation,
        fence_length: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_container(
    span: Option<SourceSpan>,
    info: &str,
    fence_length: usize,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<NodeKind> {
    let span = span?;
    let (snippet, start_byte) = source_slice(span, body, body_start_byte)?;
    let info_offset = container_info_offset(snippet).unwrap_or(0);
    let outcome = parse_container_directive_info(info, start_byte + info_offset);
    push_diagnostics(
        outcome.diagnostics,
        body,
        body_start_line,
        body_start_byte,
        path,
        diagnostics,
    );
    if !has_exact_closing_fence(snippet, fence_length) {
        diagnostics.push(
            Diagnostic::error(
                "MS3409",
                format!(
                    "container directive must close with exactly {fence_length} `:` characters"
                ),
            )
            .at(path, span),
        );
        return None;
    }
    outcome.directive.map(|invocation| NodeKind::Directive {
        invocation,
        fence_length: Some(fence_length),
    })
}

fn source_slice(span: SourceSpan, body: &str, body_start_byte: usize) -> Option<(&str, usize)> {
    let start = span.start_byte?.checked_sub(body_start_byte)?;
    let end = span.end_byte?.checked_sub(body_start_byte)?;
    body.get(start..end)
        .map(|source| (source, start + body_start_byte))
}

fn container_info_offset(source: &str) -> Option<usize> {
    let opening = source.lines().next().unwrap_or(source);
    let bytes = opening.as_bytes();
    let mut cursor = bytes.iter().take_while(|byte| **byte == b' ').count();
    let fence_start = cursor;
    while bytes.get(cursor) == Some(&b':') {
        cursor += 1;
    }
    if cursor - fence_start < 3 {
        return None;
    }
    while matches!(bytes.get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    Some(cursor)
}

fn has_exact_closing_fence(source: &str, fence_length: usize) -> bool {
    let Some(last) = source.lines().rev().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    let leading = last.bytes().take_while(|byte| *byte == b' ').count();
    if leading > 3 {
        return false;
    }
    let content = &last[leading..];
    let colons = content.bytes().take_while(|byte| *byte == b':').count();
    colons == fence_length && content[colons..].trim().is_empty()
}

fn push_diagnostics(
    items: Vec<DirectiveDiagnostic>,
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    diagnostics.extend(items.into_iter().map(|item| {
        let mut diagnostic = Diagnostic::error(item.code, item.message).at(
            path,
            source_span(
                body,
                body_start_line,
                body_start_byte,
                item.span.start,
                item.span.end,
            ),
        );
        if let Some(help) = item.help {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic
    }));
}

fn source_span(
    body: &str,
    body_start_line: usize,
    body_start_byte: usize,
    start_byte: usize,
    end_byte: usize,
) -> SourceSpan {
    let local_start = start_byte.saturating_sub(body_start_byte).min(body.len());
    let local_end = end_byte.saturating_sub(body_start_byte).min(body.len());
    SourceSpan {
        start: position_at(body, body_start_line, local_start),
        end: position_at(body, body_start_line, local_end),
        start_byte: Some(start_byte),
        end_byte: Some(end_byte),
    }
}

fn position_at(body: &str, body_start_line: usize, byte: usize) -> SourcePosition {
    let prefix = body.get(..byte).unwrap_or(body);
    let bytes = prefix.as_bytes();
    let mut cursor = 0;
    let mut line_breaks = 0;
    let mut line_start = 0;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\r' => {
                cursor += 1;
                if bytes.get(cursor) == Some(&b'\n') {
                    cursor += 1;
                }
                line_breaks += 1;
                line_start = cursor;
            }
            b'\n' => {
                cursor += 1;
                line_breaks += 1;
                line_start = cursor;
            }
            _ => cursor += 1,
        }
    }
    let column = prefix[line_start..].chars().count() + 1;
    SourcePosition {
        line: body_start_line + line_breaks,
        column,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{lower_directives, lower_obsidian, position_at};
    use crate::{
        Compiler, ComrakMarkdownParser, Config, Diagnostic, DirectiveForm, MarkdownNode,
        MarkdownParser, NodeKind, SourcePosition,
    };

    fn lower_body(body: &str) -> (MarkdownNode, Vec<Diagnostic>) {
        let mut root = ComrakMarkdownParser::default().parse(body, 0);
        let mut diagnostics = Vec::new();
        lower_directives(&mut root, body, 1, 0, "index.md", &mut diagnostics);
        lower_obsidian(&mut root, body, 1, 0, "index.md", &mut diagnostics);
        (root, diagnostics)
    }

    fn text_values(node: &crate::MarkdownNode, output: &mut String) {
        if let NodeKind::Text { value } = &node.kind {
            output.push_str(value);
        }
        for child in &node.children {
            text_values(child, output);
        }
    }

    fn contains_embed(node: &crate::MarkdownNode, target: &str) -> bool {
        matches!(
            &node.kind,
            NodeKind::ObsidianEmbed { destination, .. } if destination == target
        ) || node
            .children
            .iter()
            .any(|child| contains_embed(child, target))
    }

    fn directive_contains_embed(node: &MarkdownNode, name: &str, target: &str) -> bool {
        matches!(
            &node.kind,
            NodeKind::Directive { invocation, .. }
                if invocation.name == name && contains_embed(node, target)
        ) || node
            .children
            .iter()
            .any(|child| directive_contains_embed(child, name, target))
    }

    #[test]
    fn lowers_leaf_and_container_directives_but_not_code() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            concat!(
                "# Home\n\n",
                "::children{view=\"grid\" columns=3}\n\n",
                ":::section{width=\"wide\"}\n\nInside.\n\n:::\n\n",
                "```md\n::not-a-directive{}\n```\n",
            ),
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert_eq!(result.diagnostics, []);
        let body = &result.site.unwrap().pages[0].body;
        assert!(body.children.iter().any(|node| matches!(
            &node.kind,
            NodeKind::Directive { invocation, fence_length: None }
                if invocation.name == "children" && invocation.form == DirectiveForm::Leaf
        )));
        assert!(body.children.iter().any(|node| matches!(
            &node.kind,
            NodeKind::Directive { invocation, fence_length: Some(3) }
                if invocation.name == "section" && invocation.form == DirectiveForm::Container
        )));
        assert!(
            body.children
                .iter()
                .any(|node| matches!(node.kind, NodeKind::CodeBlock { .. }))
        );
    }

    #[test]
    fn malformed_leaf_directive_is_a_compiler_error() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            "# Home\n\n::children{view=\"grid\" view=\"list\"}\n",
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS3404")
        );
        assert!(result.site.is_none());
    }

    #[test]
    fn lowers_obsidian_embeds_comments_and_block_identifiers() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            concat!(
                "# Home\n\nBefore ![[Other#Part]] after.\n\n",
                "%% hidden **secret** %% visible\n\n",
                "Identified paragraph.\n\n^home-block\n\n",
                "[[Other#^target-block]]\n",
            ),
        )
        .unwrap();
        fs::write(
            docs.join("Other.md"),
            "# Other\n\n## Part\n\nTarget paragraph. ^target-block\n",
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert_eq!(result.diagnostics, []);
        let site = result.site.unwrap();
        let home = site.pages.iter().find(|page| page.route == "/").unwrap();
        let other = site
            .pages
            .iter()
            .find(|page| page.route == "/other/")
            .unwrap();
        assert!(contains_embed(&home.body, "Other#Part"));
        let mut visible_text = String::new();
        text_values(&home.body, &mut visible_text);
        assert!(!visible_text.contains("hidden"));
        assert!(!visible_text.contains("secret"));
        assert!(visible_text.contains("visible"));
        assert_eq!(home.blocks[0].id, "home-block");
        assert_eq!(other.blocks[0].id, "target-block");
        assert_eq!(home.embeds.len(), 1);
        assert!(matches!(
            home.outgoing_links[0].target,
            crate::ResolvedLinkTarget::Page {
                fragment: Some(crate::ResolvedFragment::Block { .. }),
                ..
            }
        ));
    }

    #[test]
    fn lowers_obsidian_syntax_inside_directive_container_children() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            concat!(
                "# Home\n\n",
                ":::section{tone=\"subtle\"}\n\n",
                "Before ![[Other]] after %% section secret %% visible.\n\n",
                "Identified inside section. ^inside-section\n\n",
                ":::\n\n",
                "::::columns{count=2}\n\n",
                ":::column\n\nFirst ![[Other]].\n\n:::\n\n",
                ":::column\n\nSecond %% column secret %% shown.\n\n:::\n\n",
                "::::\n",
            ),
        )
        .unwrap();
        fs::write(docs.join("Other.md"), "# Other\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert_eq!(result.diagnostics, []);
        let site = result.site.unwrap();
        let home = site.pages.iter().find(|page| page.route == "/").unwrap();
        assert!(directive_contains_embed(&home.body, "section", "Other"));
        assert!(directive_contains_embed(&home.body, "columns", "Other"));
        assert_eq!(home.embeds.len(), 2);
        assert_eq!(home.blocks[0].id, "inside-section");
        let mut visible_text = String::new();
        text_values(&home.body, &mut visible_text);
        assert!(!visible_text.contains("section secret"));
        assert!(!visible_text.contains("column secret"));
        assert!(visible_text.contains("visible"));
        assert!(visible_text.contains("shown"));
    }

    #[test]
    fn conflicting_standalone_block_markers_preserve_the_first_identifier() {
        let (body, diagnostics) = lower_body(concat!(
            "Trailing. ^first\n\n",
            "^second\n\n",
            "Separate.\n\n",
            "^third\n\n",
            "^fourth\n",
        ));

        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "MS3309")
                .count(),
            2
        );
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic.code != "MS3309" || diagnostic.message.contains("already has identifier")
        }));
        let identifiers: Vec<_> = body
            .children
            .iter()
            .filter_map(|node| node.block_id.as_deref())
            .collect();
        assert_eq!(identifiers, ["first", "third"]);
        let mut visible_text = String::new();
        text_values(&body, &mut visible_text);
        assert!(!visible_text.contains("^second"));
        assert!(!visible_text.contains("^fourth"));
    }

    #[test]
    fn split_text_pieces_receive_exact_line_column_and_byte_spans() {
        let source = "α Before %% hidden %% after ![[Other]] tail.";
        let (body, diagnostics) = lower_body(source);
        assert_eq!(diagnostics, []);
        let paragraph = &body.children[0];
        let pieces: Vec<_> = paragraph
            .children
            .iter()
            .filter_map(|node| match &node.kind {
                NodeKind::Text { value } => Some((value.as_str(), node.span.unwrap())),
                _ => None,
            })
            .collect();

        assert_eq!(
            pieces.iter().map(|(value, _)| *value).collect::<Vec<_>>(),
            ["α Before ", " after ", " tail."]
        );
        for &(value, span) in &pieces {
            let start = source.find(value).unwrap();
            let end = start + value.len();
            assert_eq!(span.start_byte, Some(start));
            assert_eq!(span.end_byte, Some(end));
        }
        assert_eq!(pieces[0].1.start.column, 1);
        assert_eq!(pieces[0].1.end.column, 10);
        assert_eq!(pieces[1].1.start.column, 22);
        assert_eq!(pieces[1].1.end.column, 29);
        assert_eq!(pieces[2].1.start.column, 39);
        assert_eq!(pieces[2].1.end.column, 45);
    }

    #[test]
    fn dialect_positions_treat_lf_crlf_and_cr_as_single_line_endings() {
        let source = "one\r\ntwo\rthree\nfour";
        assert_eq!(
            position_at(source, 10, 0),
            SourcePosition {
                line: 10,
                column: 1
            }
        );
        assert_eq!(
            position_at(source, 10, 4),
            SourcePosition {
                line: 11,
                column: 1
            }
        );
        assert_eq!(
            position_at(source, 10, 5),
            SourcePosition {
                line: 11,
                column: 1
            }
        );
        assert_eq!(
            position_at(source, 10, 9),
            SourcePosition {
                line: 12,
                column: 1
            }
        );
        assert_eq!(
            position_at(source, 10, 15),
            SourcePosition {
                line: 13,
                column: 1
            }
        );
    }

    #[test]
    fn diagnoses_unclosed_comments_and_duplicate_block_ids() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(
            docs.join("index.md"),
            "# Home\n\nOne. ^same\n\nTwo. ^same\n\n%% never closed\n",
        )
        .unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let result = Compiler::new(config).compile();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS3305")
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "MS3308")
        );
        assert!(result.site.is_none());
    }
}
