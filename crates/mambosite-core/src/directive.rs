//! Schema-1 directive syntax parsing.
//!
//! This module recognizes invocations and typed property values. The versioned
//! component registry is a separate semantic-validation pass responsible for
//! unknown directive/property names, required values, defaults, and context.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Number;

/// A half-open UTF-8 byte range in the original Markdown source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectiveSpan {
    pub start: usize,
    pub end: usize,
}

impl DirectiveSpan {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// Whether an invocation came from a leaf block or a container's info string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DirectiveForm {
    Leaf,
    Container,
}

/// A scalar accepted by directive arrays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum DirectiveScalar {
    String(String),
    Number(Number),
    Boolean(bool),
}

/// A schema-1 directive property value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum DirectiveValue {
    String(String),
    Number(Number),
    Boolean(bool),
    Array(Vec<DirectiveScalar>),
}

/// One property in authored order, including its exact source range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectiveProperty {
    pub name: String,
    pub value: DirectiveValue,
    pub span: DirectiveSpan,
    pub name_span: DirectiveSpan,
    pub value_span: DirectiveSpan,
    pub raw: String,
}

/// A syntactically valid directive invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDirective {
    pub form: DirectiveForm,
    pub name: String,
    pub properties: Vec<DirectiveProperty>,
    pub span: DirectiveSpan,
    pub name_span: DirectiveSpan,
    pub raw: String,
}

impl ParsedDirective {
    /// Look up a property without discarding authored property order.
    pub fn property(&self, name: &str) -> Option<&DirectiveValue> {
        self.properties
            .iter()
            .find(|property| property.name == name)
            .map(|property| &property.value)
    }
}

/// A syntax diagnostic whose range is relative to the original Markdown file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectiveDiagnostic {
    pub code: String,
    pub message: String,
    pub span: DirectiveSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

/// The result of testing and parsing one candidate directive block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectiveParseOutcome {
    /// False means the input was ordinary Markdown, an escaped marker, an
    /// indented code block, or a container fence presented to the leaf parser.
    pub recognized: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directive: Option<ParsedDirective>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DirectiveDiagnostic>,
}

impl DirectiveParseOutcome {
    fn not_recognized() -> Self {
        Self {
            recognized: false,
            directive: None,
            diagnostics: Vec::new(),
        }
    }
}

/// Parse one complete leaf-directive block.
///
/// `source_offset` is the byte offset at which `source` begins in the original
/// Markdown file. The caller remains responsible for presenting only regions
/// outside frontmatter and code spans/blocks.
pub fn parse_leaf_directive(source: &str, source_offset: usize) -> DirectiveParseOutcome {
    let leading_spaces = source.bytes().take_while(|byte| *byte == b' ').count();
    if leading_spaces > 3 {
        return DirectiveParseOutcome::not_recognized();
    }
    let candidate = &source[leading_spaces..];
    if candidate.starts_with("\\::") || !candidate.starts_with("::") || candidate.starts_with(":::")
    {
        return DirectiveParseOutcome::not_recognized();
    }

    Parser::new(
        source,
        source_offset,
        leading_spaces + 2,
        DirectiveForm::Leaf,
    )
    .parse()
}

/// Parse the info string retained by a Comrak container block-directive node.
///
/// The info string begins with the directive name; the opening fence is not
/// part of this input. `source_offset` should point at the first info byte in
/// the original Markdown source.
pub fn parse_container_directive_info(info: &str, source_offset: usize) -> DirectiveParseOutcome {
    Parser::new(info, source_offset, 0, DirectiveForm::Container).parse()
}

struct Parser<'source> {
    source: &'source str,
    source_offset: usize,
    cursor: usize,
    form: DirectiveForm,
    diagnostics: Vec<DirectiveDiagnostic>,
}

impl<'source> Parser<'source> {
    fn new(source: &'source str, source_offset: usize, cursor: usize, form: DirectiveForm) -> Self {
        Self {
            source,
            source_offset,
            cursor,
            form,
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> DirectiveParseOutcome {
        let Some((name, name_span)) = self.parse_name("directive") else {
            return self.finish(None);
        };
        self.skip_horizontal_whitespace();

        let properties = if self.peek() == Some(b'{') {
            match self.parse_properties() {
                Some(properties) => properties,
                None => return self.finish(None),
            }
        } else {
            Vec::new()
        };

        self.skip_whitespace();
        if !self.is_eof() {
            let start = self.cursor;
            self.error(
                "MS3401",
                "unexpected text after directive",
                start,
                self.next_char_end(start),
                Some("a directive must occupy its complete block"),
            );
        }

        if !self.diagnostics.is_empty() {
            return self.finish(None);
        }

        let directive = ParsedDirective {
            form: self.form,
            name,
            properties,
            span: self.span(0, self.source.len()),
            name_span,
            raw: self.source.to_owned(),
        };
        self.finish(Some(directive))
    }

    fn parse_properties(&mut self) -> Option<Vec<DirectiveProperty>> {
        self.cursor += 1;
        let mut properties = Vec::new();
        let mut seen = BTreeSet::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.cursor += 1;
            return Some(properties);
        }

        loop {
            if self.is_eof() {
                self.error_at_eof(
                    "MS3403",
                    "directive attributes are missing a closing `}`",
                    Some("close the attribute block before Markdown body content"),
                );
                return None;
            }

            let property_start = self.cursor;
            let (name, name_span) = self.parse_name("property")?;
            self.skip_horizontal_whitespace();
            if self.peek() != Some(b'=') {
                self.error(
                    "MS3403",
                    format!("property `{name}` is missing `=`"),
                    self.cursor,
                    self.next_char_end(self.cursor),
                    Some("write properties as `name=value`"),
                );
                return None;
            }
            self.cursor += 1;
            self.skip_horizontal_whitespace();
            let value_start = self.cursor;
            let value = self.parse_value()?;
            let value_end = self.cursor;
            let property_end = self.cursor;

            if seen.insert(name.clone()) {
                properties.push(DirectiveProperty {
                    name,
                    value,
                    span: self.span(property_start, property_end),
                    name_span,
                    value_span: self.span(value_start, value_end),
                    raw: self.source[property_start..property_end].to_owned(),
                });
            } else {
                self.error(
                    "MS3404",
                    format!("property `{name}` is declared more than once"),
                    name_span.start - self.source_offset,
                    name_span.end - self.source_offset,
                    Some("remove one declaration; duplicate properties are never overwritten"),
                );
            }

            let separator_start = self.cursor;
            self.skip_whitespace();
            match self.peek() {
                Some(b'}') => {
                    self.cursor += 1;
                    return Some(properties);
                }
                None => {
                    self.error_at_eof(
                        "MS3403",
                        "directive attributes are missing a closing `}`",
                        Some("close the attribute block before Markdown body content"),
                    );
                    return None;
                }
                Some(_) if self.cursor == separator_start => {
                    self.error(
                        "MS3401",
                        "directive properties must be separated by whitespace",
                        self.cursor,
                        self.next_char_end(self.cursor),
                        None::<String>,
                    );
                    return None;
                }
                Some(_) => {}
            }
        }
    }

    fn parse_value(&mut self) -> Option<DirectiveValue> {
        match self.peek() {
            Some(b'"') => self.parse_string().map(DirectiveValue::String),
            Some(b'[') => self.parse_array().map(DirectiveValue::Array),
            Some(b't') if self.consume_keyword("true") => Some(DirectiveValue::Boolean(true)),
            Some(b'f') if self.consume_keyword("false") => Some(DirectiveValue::Boolean(false)),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(DirectiveValue::Number),
            Some(_) => {
                let start = self.cursor;
                self.error(
                    "MS3405",
                    "expected a quoted string, finite number, boolean, or scalar array",
                    start,
                    self.next_token_end(start),
                    Some("strings must use double quotes"),
                );
                None
            }
            None => {
                self.error_at_eof(
                    "MS3405",
                    "expected a directive property value",
                    None::<String>,
                );
                None
            }
        }
    }

    fn parse_scalar(&mut self) -> Option<DirectiveScalar> {
        match self.peek() {
            Some(b'"') => self.parse_string().map(DirectiveScalar::String),
            Some(b't') if self.consume_keyword("true") => Some(DirectiveScalar::Boolean(true)),
            Some(b'f') if self.consume_keyword("false") => Some(DirectiveScalar::Boolean(false)),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(DirectiveScalar::Number),
            Some(b'[' | b'{') => {
                let start = self.cursor;
                self.error(
                    "MS3406",
                    "directive arrays may contain scalar values only",
                    start,
                    self.next_char_end(start),
                    None::<String>,
                );
                None
            }
            Some(_) => {
                let start = self.cursor;
                self.error(
                    "MS3405",
                    "expected a scalar array value",
                    start,
                    self.next_token_end(start),
                    Some("array strings must use double quotes"),
                );
                None
            }
            None => {
                self.error_at_eof("MS3403", "directive array is not closed", None::<String>);
                None
            }
        }
    }

    fn parse_array(&mut self) -> Option<Vec<DirectiveScalar>> {
        self.cursor += 1;
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.cursor += 1;
            return Some(values);
        }

        loop {
            values.push(self.parse_scalar()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b']') => {
                    self.cursor += 1;
                    return Some(values);
                }
                Some(b',') => {
                    self.cursor += 1;
                    self.skip_whitespace();
                    if self.peek() == Some(b']') {
                        self.error(
                            "MS3401",
                            "directive arrays may not have a trailing comma",
                            self.cursor,
                            self.cursor + 1,
                            None::<String>,
                        );
                        return None;
                    }
                }
                Some(_) => {
                    self.error(
                        "MS3401",
                        "expected `,` or `]` after directive array value",
                        self.cursor,
                        self.next_char_end(self.cursor),
                        None::<String>,
                    );
                    return None;
                }
                None => {
                    self.error_at_eof("MS3403", "directive array is not closed", None::<String>);
                    return None;
                }
            }
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        self.cursor += 1;
        let mut value = String::new();
        while let Some(character) = self.source[self.cursor..].chars().next() {
            let start = self.cursor;
            self.cursor += character.len_utf8();
            match character {
                '"' => return Some(value),
                '\\' => {
                    let Some(escaped) = self.source[self.cursor..].chars().next() else {
                        self.error_at_eof(
                            "MS3403",
                            "quoted directive string is not closed",
                            None::<String>,
                        );
                        return None;
                    };
                    match escaped {
                        '"' | '\\' => {
                            value.push(escaped);
                            self.cursor += escaped.len_utf8();
                        }
                        _ => {
                            let end = self.cursor + escaped.len_utf8();
                            self.error(
                                "MS3407",
                                format!("unsupported string escape `\\{escaped}`"),
                                start,
                                end,
                                Some("only `\\\"` and `\\\\` are supported"),
                            );
                            return None;
                        }
                    }
                }
                '\n' | '\r' => {
                    self.error(
                        "MS3401",
                        "quoted directive strings cannot contain line endings",
                        start,
                        self.cursor,
                        None::<String>,
                    );
                    return None;
                }
                _ => value.push(character),
            }
        }
        self.error_at_eof(
            "MS3403",
            "quoted directive string is not closed",
            None::<String>,
        );
        None
    }

    fn parse_number(&mut self) -> Option<Number> {
        let start = self.cursor;
        let end = self.next_token_end(start);
        self.cursor = end;
        let raw = &self.source[start..end];
        if let Ok(number) = raw.parse::<Number>() {
            Some(number)
        } else {
            self.error(
                "MS3408",
                format!("`{raw}` is not a finite decimal number"),
                start,
                end,
                None::<String>,
            );
            None
        }
    }

    fn parse_name(&mut self, subject: &str) -> Option<(String, DirectiveSpan)> {
        let start = self.cursor;
        if !self.peek().is_some_and(|byte| byte.is_ascii_lowercase()) {
            self.error(
                "MS3402",
                format!("{subject} name must begin with a lowercase ASCII letter"),
                start,
                self.next_char_end(start),
                Some("names may contain lowercase letters, digits, and hyphens"),
            );
            return None;
        }
        self.cursor += 1;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            self.cursor += 1;
        }
        let end = self.cursor;
        Some((self.source[start..end].to_owned(), self.span(start, end)))
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if !self.source[self.cursor..].starts_with(keyword) {
            return false;
        }
        let end = self.cursor + keyword.len();
        if self
            .source
            .as_bytes()
            .get(end)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(*byte, b',' | b']' | b'}'))
        {
            return false;
        }
        self.cursor = end;
        true
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.cursor += 1;
        }
    }

    fn skip_horizontal_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.cursor += 1;
        }
    }

    fn next_token_end(&self, start: usize) -> usize {
        self.source[start..]
            .char_indices()
            .find_map(|(offset, character)| {
                (character.is_whitespace() || matches!(character, ',' | ']' | '}'))
                    .then_some(start + offset)
            })
            .unwrap_or(self.source.len())
    }

    fn next_char_end(&self, start: usize) -> usize {
        self.source
            .get(start..)
            .and_then(|tail| tail.chars().next())
            .map_or(start, |character| start + character.len_utf8())
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.cursor).copied()
    }

    fn is_eof(&self) -> bool {
        self.cursor == self.source.len()
    }

    fn span(&self, start: usize, end: usize) -> DirectiveSpan {
        DirectiveSpan::new(self.source_offset + start, self.source_offset + end)
    }

    fn error(
        &mut self,
        code: &str,
        message: impl Into<String>,
        start: usize,
        end: usize,
        help: Option<impl Into<String>>,
    ) {
        self.diagnostics.push(DirectiveDiagnostic {
            code: code.to_owned(),
            message: message.into(),
            span: self.span(start, end),
            help: help.map(Into::into),
        });
    }

    fn error_at_eof(
        &mut self,
        code: &str,
        message: impl Into<String>,
        help: Option<impl Into<String>>,
    ) {
        self.error(code, message, self.cursor, self.cursor, help);
    }

    fn finish(self, directive: Option<ParsedDirective>) -> DirectiveParseOutcome {
        DirectiveParseOutcome {
            recognized: true,
            directive,
            diagnostics: self.diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(source: &str) -> ParsedDirective {
        let outcome = parse_leaf_directive(source, 0);
        assert_eq!(outcome.diagnostics, []);
        outcome.directive.expect("parsed directive")
    }

    #[test]
    fn parses_leaf_values_and_preserves_authored_order() {
        let directive = parsed(
            r#"::children{view="grid" columns=3 ratio=-1.25 enabled=true show=["title",2,false]}"#,
        );
        assert_eq!(directive.form, DirectiveForm::Leaf);
        assert_eq!(directive.name, "children");
        assert_eq!(
            directive
                .properties
                .iter()
                .map(|property| property.name.as_str())
                .collect::<Vec<_>>(),
            ["view", "columns", "ratio", "enabled", "show"]
        );
        assert_eq!(
            directive.property("view"),
            Some(&DirectiveValue::String("grid".into()))
        );
        assert_eq!(
            directive.property("show"),
            Some(&DirectiveValue::Array(vec![
                DirectiveScalar::String("title".into()),
                DirectiveScalar::Number(Number::from(2)),
                DirectiveScalar::Boolean(false),
            ]))
        );
    }

    #[test]
    fn accepts_multiline_attributes_and_offsets_spans() {
        let source = "  ::children{\n  view=\"grid\"\n  columns=3\n}\n";
        let outcome = parse_leaf_directive(source, 100);
        let directive = outcome.directive.unwrap();
        assert_eq!(directive.span, DirectiveSpan::new(100, 142));
        assert_eq!(directive.name_span, DirectiveSpan::new(104, 112));
        assert_eq!(
            directive.properties[0].name_span,
            DirectiveSpan::new(116, 120)
        );
        assert_eq!(directive.raw, source);
    }

    #[test]
    fn parses_container_info_without_its_fence() {
        let outcome = parse_container_directive_info(r#"section{width="wide" tone="subtle"}"#, 42);
        let directive = outcome.directive.unwrap();
        assert_eq!(directive.form, DirectiveForm::Container);
        assert_eq!(directive.name, "section");
        assert_eq!(directive.name_span, DirectiveSpan::new(42, 49));
    }

    #[test]
    fn unescapes_only_documented_string_escapes() {
        let directive = parsed(r#"::button{label="a \"quote\" and \\ slash"}"#);
        assert_eq!(
            directive.property("label"),
            Some(&DirectiveValue::String("a \"quote\" and \\ slash".into()))
        );

        let outcome = parse_leaf_directive(r#"::button{label="bad\n"}"#, 0);
        assert_eq!(outcome.diagnostics[0].code, "MS3407");
    }

    #[test]
    fn ignores_escaped_markers_indented_code_and_container_fences() {
        for source in [r"\::children{}", "    ::children{}", ":::section{}"] {
            let outcome = parse_leaf_directive(source, 0);
            assert!(!outcome.recognized, "{source}");
            assert!(outcome.directive.is_none());
            assert!(outcome.diagnostics.is_empty());
        }
    }

    #[test]
    fn reports_duplicate_properties_without_overwriting() {
        let outcome = parse_leaf_directive(r"::toc{ordered=true ordered=false}", 10);
        assert!(outcome.directive.is_none());
        assert_eq!(outcome.diagnostics.len(), 1);
        assert_eq!(outcome.diagnostics[0].code, "MS3404");
        assert_eq!(outcome.diagnostics[0].span, DirectiveSpan::new(29, 36));
    }

    #[test]
    fn rejects_unquoted_strings_nested_arrays_and_trailing_commas() {
        for source in [
            "::hero{align=split}",
            "::meta{show=[[\"title\"]]}",
            "::meta{show=[\"title\",]}",
        ] {
            let outcome = parse_leaf_directive(source, 0);
            assert!(outcome.recognized, "{source}");
            assert!(outcome.directive.is_none(), "{source}");
            assert!(!outcome.diagnostics.is_empty(), "{source}");
        }
    }

    #[test]
    fn rejects_invalid_names_missing_separators_and_trailing_text() {
        for source in [
            "::Bad{}",
            "::toc{min-depth=2max-depth=4}",
            "::toc{} trailing",
        ] {
            let outcome = parse_leaf_directive(source, 0);
            assert!(outcome.directive.is_none(), "{source}");
            assert!(!outcome.diagnostics.is_empty(), "{source}");
        }
    }

    #[test]
    fn rejects_non_finite_or_non_json_numbers() {
        for source in ["::x{value=01}", "::x{value=1.}", "::x{value=-}"] {
            let outcome = parse_leaf_directive(source, 0);
            assert_eq!(outcome.diagnostics[0].code, "MS3408", "{source}");
        }
        assert!(
            parsed("::x{value=1.25e2}")
                .directive_number("value")
                .is_some()
        );
    }

    trait DirectiveTestExt {
        fn directive_number(&self, name: &str) -> Option<&Number>;
    }

    impl DirectiveTestExt for ParsedDirective {
        fn directive_number(&self, name: &str) -> Option<&Number> {
            match self.property(name) {
                Some(DirectiveValue::Number(number)) => Some(number),
                _ => None,
            }
        }
    }
}
