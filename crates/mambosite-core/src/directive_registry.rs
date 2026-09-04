//! Semantic validation for the schema-1 core directive registry.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    Diagnostic, DirectiveForm, DirectiveScalar, DirectiveSpan, DirectiveValue, MarkdownNode,
    NodeKind, ParsedDirective, SourcePosition, SourceSpan,
};

/// Source context needed to report directive-property diagnostics precisely.
#[derive(Debug, Clone, Copy)]
pub struct DirectiveValidationContext<'source> {
    pub logical_path: &'source str,
    /// Markdown body text after frontmatter removal.
    pub body: &'source str,
    /// One-based original-source line on which `body` begins.
    pub body_start_line: usize,
    /// Zero-based original-source byte at which `body` begins.
    pub body_start_byte: usize,
    pub is_index: bool,
}

/// One validated invocation with explicit schema defaults applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedDirective {
    pub name: String,
    pub form: DirectiveForm,
    pub properties: BTreeMap<String, DirectiveValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

/// Preorder validated invocations plus all semantic diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectiveValidationOutcome {
    pub directives: Vec<ValidatedDirective>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Validate every lowered `NodeKind::Directive` in a page tree.
///
/// This pass assumes syntax tokenization has completed and comments that may
/// precede `page` have already been removed or represented explicitly.
pub fn validate_directives(
    root: &MarkdownNode,
    context: DirectiveValidationContext<'_>,
) -> DirectiveValidationOutcome {
    let mut validator = Validator {
        context,
        directives: Vec::new(),
        diagnostics: Vec::new(),
        page_count: 0,
    };

    if matches!(root.kind, NodeKind::Document) {
        for (index, child) in root.children.iter().enumerate() {
            validator.visit(child, None, index == 0);
        }
    } else {
        validator.visit(root, None, true);
    }

    DirectiveValidationOutcome {
        directives: validator.directives,
        diagnostics: validator.diagnostics,
    }
}

struct Validator<'source> {
    context: DirectiveValidationContext<'source>,
    directives: Vec<ValidatedDirective>,
    diagnostics: Vec<Diagnostic>,
    page_count: usize,
}

impl Validator<'_> {
    fn visit(
        &mut self,
        node: &MarkdownNode,
        direct_parent_directive: Option<&str>,
        is_first_top_level: bool,
    ) {
        let current_name = if let NodeKind::Directive { invocation, .. } = &node.kind {
            self.validate_invocation(
                invocation,
                node,
                direct_parent_directive,
                is_first_top_level,
            );
            Some(invocation.name.as_str())
        } else {
            None
        };

        for child in &node.children {
            self.visit(child, current_name, false);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn validate_invocation(
        &mut self,
        invocation: &ParsedDirective,
        node: &MarkdownNode,
        direct_parent_directive: Option<&str>,
        is_first_top_level: bool,
    ) {
        let diagnostic_start = self.diagnostics.len();
        let Some(spec) = directive_spec(&invocation.name) else {
            let mut diagnostic = Diagnostic::error(
                "MS3501",
                format!("unknown schema-1 directive `{}`", invocation.name),
            );
            if let Some(suggestion) = closest_name(&invocation.name, DIRECTIVE_NAMES) {
                diagnostic = diagnostic.with_help(format!("did you mean `{suggestion}`?"));
            }
            self.push_invocation_diagnostic(diagnostic, invocation, node);
            return;
        };

        if invocation.form != spec.form {
            self.push_invocation_diagnostic(
                Diagnostic::error(
                    "MS3502",
                    format!(
                        "directive `{}` must use the {} form",
                        invocation.name,
                        form_name(spec.form)
                    ),
                ),
                invocation,
                node,
            );
        }

        let mut normalized = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for property in &invocation.properties {
            if !seen.insert(property.name.as_str()) {
                self.push_span_diagnostic(
                    Diagnostic::error(
                        "MS3511",
                        format!("property `{}` is declared more than once", property.name),
                    ),
                    property.name_span,
                    node.span,
                );
                continue;
            }
            let Some(rule) = spec
                .properties
                .iter()
                .find(|rule| rule.name == property.name)
            else {
                let names: Vec<_> = spec.properties.iter().map(|rule| rule.name).collect();
                let mut diagnostic = Diagnostic::error(
                    "MS3503",
                    format!(
                        "unknown property `{}` on directive `{}`",
                        property.name, invocation.name
                    ),
                );
                if let Some(suggestion) = closest_name(&property.name, &names) {
                    diagnostic = diagnostic.with_help(format!("did you mean `{suggestion}`?"));
                }
                self.push_span_diagnostic(diagnostic, property.name_span, node.span);
                continue;
            };

            if let Err(message) = validate_value(&property.value, rule.kind) {
                self.push_span_diagnostic(
                    Diagnostic::error(
                        "MS3504",
                        format!(
                            "invalid value for `{}.{}`: {message}",
                            invocation.name, property.name
                        ),
                    ),
                    property.value_span,
                    node.span,
                );
            } else {
                normalized.insert(property.name.clone(), property.value.clone());
            }
        }

        for rule in spec.properties {
            if rule.required && !seen.contains(rule.name) {
                self.push_invocation_diagnostic(
                    Diagnostic::error(
                        "MS3506",
                        format!(
                            "directive `{}` requires property `{}`",
                            invocation.name, rule.name
                        ),
                    ),
                    invocation,
                    node,
                );
            }
            if !seen.contains(rule.name)
                && let Some(default) = rule.default
            {
                normalized.insert(rule.name.to_owned(), default.into_value());
            }
        }

        self.validate_cross_property_rules(invocation, node, &normalized);
        self.validate_context(
            invocation,
            node,
            direct_parent_directive,
            is_first_top_level,
            &normalized,
        );

        if self.diagnostics.len() == diagnostic_start {
            self.directives.push(ValidatedDirective {
                name: invocation.name.clone(),
                form: invocation.form,
                properties: normalized,
                span: node.span,
            });
        }
    }

    fn validate_cross_property_rules(
        &mut self,
        invocation: &ParsedDirective,
        node: &MarkdownNode,
        normalized: &BTreeMap<String, DirectiveValue>,
    ) {
        if invocation.name == "toc"
            && let (Some(minimum), Some(maximum)) = (
                integer_property(normalized, "min-depth"),
                integer_property(normalized, "max-depth"),
            )
            && minimum > maximum
        {
            self.push_invocation_diagnostic(
                Diagnostic::error("MS3505", "`toc.min-depth` may not exceed `toc.max-depth`"),
                invocation,
                node,
            );
        }
    }

    fn validate_context(
        &mut self,
        invocation: &ParsedDirective,
        node: &MarkdownNode,
        direct_parent_directive: Option<&str>,
        is_first_top_level: bool,
        normalized: &BTreeMap<String, DirectiveValue>,
    ) {
        match invocation.name.as_str() {
            "page" => {
                self.page_count += 1;
                if self.page_count > 1 {
                    self.push_invocation_diagnostic(
                        Diagnostic::error("MS3507", "a page may declare `page` at most once"),
                        invocation,
                        node,
                    );
                }
                if !is_first_top_level {
                    self.push_invocation_diagnostic(
                        Diagnostic::error(
                            "MS3508",
                            "`page` must be the first body node other than comments or blank lines",
                        ),
                        invocation,
                        node,
                    );
                }
            }
            "children" if !self.context.is_index => self.push_invocation_diagnostic(
                Diagnostic::error(
                    "MS3509",
                    "`children` is valid only on a page named `index.md`",
                ),
                invocation,
                node,
            ),
            "column" if direct_parent_directive != Some("columns") => {
                self.push_invocation_diagnostic(
                    Diagnostic::error("MS3509", "`column` must be a direct child of `columns`"),
                    invocation,
                    node,
                );
            }
            "columns" => self.validate_columns(node, invocation, normalized),
            _ => {}
        }
    }

    fn validate_columns(
        &mut self,
        node: &MarkdownNode,
        invocation: &ParsedDirective,
        normalized: &BTreeMap<String, DirectiveValue>,
    ) {
        let mut column_count = 0_i64;
        for child in &node.children {
            if let NodeKind::Directive {
                invocation: child_invocation,
                ..
            } = &child.kind
            {
                if child_invocation.name == "column" {
                    column_count += 1;
                } else {
                    self.push_invocation_diagnostic(
                        Diagnostic::error(
                            "MS3510",
                            format!(
                                "direct directive child `{}` of `columns` must be `column`",
                                child_invocation.name
                            ),
                        ),
                        child_invocation,
                        child,
                    );
                }
            }
        }

        if let Some(expected) = integer_property(normalized, "count")
            && column_count < expected
        {
            self.push_invocation_diagnostic(
                Diagnostic::error(
                    "MS3510",
                    format!(
                        "`columns.count` is {expected}, but the container has only {column_count} direct `column` children"
                    ),
                ),
                invocation,
                node,
            );
        }
    }

    fn push_invocation_diagnostic(
        &mut self,
        diagnostic: Diagnostic,
        invocation: &ParsedDirective,
        node: &MarkdownNode,
    ) {
        self.push_span_diagnostic(diagnostic, invocation.name_span, node.span);
    }

    fn push_span_diagnostic(
        &mut self,
        diagnostic: Diagnostic,
        span: DirectiveSpan,
        fallback: Option<SourceSpan>,
    ) {
        let diagnostic = if let Some(span) = directive_source_span(span, self.context) {
            diagnostic.at(self.context.logical_path, span)
        } else if let Some(span) = fallback {
            diagnostic.at(self.context.logical_path, span)
        } else {
            diagnostic.at_path(self.context.logical_path)
        };
        self.diagnostics.push(diagnostic);
    }
}

#[derive(Debug, Clone, Copy)]
struct DirectiveSpec {
    name: &'static str,
    form: DirectiveForm,
    properties: &'static [PropertyRule],
}

#[derive(Debug, Clone, Copy)]
struct PropertyRule {
    name: &'static str,
    kind: ValueRule,
    required: bool,
    default: Option<DefaultValue>,
}

#[derive(Debug, Clone, Copy)]
enum ValueRule {
    String,
    Boolean,
    StringArray,
    Enumeration(&'static [&'static str]),
    Integer {
        minimum: i64,
        maximum: Option<i64>,
        allow_negative_one: bool,
    },
}

#[derive(Debug, Clone, Copy)]
enum DefaultValue {
    String(&'static str),
    Boolean(bool),
    Integer(i64),
}

impl DefaultValue {
    fn into_value(self) -> DirectiveValue {
        match self {
            Self::String(value) => DirectiveValue::String(value.to_owned()),
            Self::Boolean(value) => DirectiveValue::Boolean(value),
            Self::Integer(value) => DirectiveValue::Number(value.into()),
        }
    }
}

const fn property(name: &'static str, kind: ValueRule) -> PropertyRule {
    PropertyRule {
        name,
        kind,
        required: false,
        default: None,
    }
}

const fn required(name: &'static str, kind: ValueRule) -> PropertyRule {
    PropertyRule {
        name,
        kind,
        required: true,
        default: None,
    }
}

const fn defaulted(name: &'static str, kind: ValueRule, default: DefaultValue) -> PropertyRule {
    PropertyRule {
        name,
        kind,
        required: false,
        default: Some(default),
    }
}

const STRING: ValueRule = ValueRule::String;
const BOOLEAN: ValueRule = ValueRule::Boolean;
const STRING_ARRAY: ValueRule = ValueRule::StringArray;
const POSITIVE: ValueRule = ValueRule::Integer {
    minimum: 1,
    maximum: None,
    allow_negative_one: false,
};
const ONE_TO_SIX: ValueRule = ValueRule::Integer {
    minimum: 1,
    maximum: Some(6),
    allow_negative_one: false,
};

const PAGE: &[PropertyRule] = &[
    defaulted(
        "layout",
        ValueRule::Enumeration(&[
            "default",
            "article",
            "docs",
            "project",
            "collection",
            "home",
            "gallery",
        ]),
        DefaultValue::String("default"),
    ),
    property(
        "width",
        ValueRule::Enumeration(&["narrow", "normal", "wide", "full"]),
    ),
    property("sidebar", BOOLEAN),
];

const HERO: &[PropertyRule] = &[
    property("image", STRING),
    defaulted(
        "align",
        ValueRule::Enumeration(&["left", "center", "split"]),
        DefaultValue::String("left"),
    ),
    defaulted("show-title", BOOLEAN, DefaultValue::Boolean(true)),
    defaulted("show-description", BOOLEAN, DefaultValue::Boolean(true)),
    defaulted("show-meta", BOOLEAN, DefaultValue::Boolean(false)),
];

const BREADCRUMBS: &[PropertyRule] = &[
    property("home", STRING),
    defaulted("separator", STRING, DefaultValue::String("/")),
    defaulted("include-current", BOOLEAN, DefaultValue::Boolean(true)),
];

const META: &[PropertyRule] = &[
    property("show", STRING_ARRAY),
    property(
        "style",
        ValueRule::Enumeration(&["inline", "stack", "table"]),
    ),
    property("empty", ValueRule::Enumeration(&["hide", "placeholder"])),
];

const TOC: &[PropertyRule] = &[
    property("min-depth", ONE_TO_SIX),
    property("max-depth", ONE_TO_SIX),
    property("ordered", BOOLEAN),
    property("title", STRING),
    property("collapse", BOOLEAN),
];

const CHILDREN: &[PropertyRule] = &[
    property("source", STRING),
    defaulted(
        "view",
        ValueRule::Enumeration(&["list", "grid", "cards", "tree", "table", "hidden"]),
        DefaultValue::String("list"),
    ),
    defaulted(
        "depth",
        ValueRule::Integer {
            minimum: 1,
            maximum: None,
            allow_negative_one: true,
        },
        DefaultValue::Integer(1),
    ),
    defaulted(
        "sort",
        ValueRule::Enumeration(&["order", "title", "date", "updated", "path"]),
        DefaultValue::String("order"),
    ),
    property("direction", ValueRule::Enumeration(&["asc", "desc"])),
    property("columns", ONE_TO_SIX),
    property("limit", POSITIVE),
    property("show", STRING_ARRAY),
    defaulted("include-unlisted", BOOLEAN, DefaultValue::Boolean(false)),
    defaulted(
        "empty",
        ValueRule::Enumeration(&["hide", "message"]),
        DefaultValue::String("hide"),
    ),
];

const RELATED: &[PropertyRule] = &[
    property("by", ValueRule::Enumeration(&["tags", "links", "both"])),
    property("view", STRING),
    property("limit", POSITIVE),
    property("show", STRING_ARRAY),
    property("include-unlisted", BOOLEAN),
];

const BACKLINKS: &[PropertyRule] = &[
    property("view", ValueRule::Enumeration(&["list", "cards"])),
    property("limit", POSITIVE),
    property("show", STRING_ARRAY),
    property("empty", STRING),
];

const GALLERY: &[PropertyRule] = &[
    property("source", STRING),
    property(
        "view",
        ValueRule::Enumeration(&["grid", "masonry", "carousel"]),
    ),
    property("columns", ONE_TO_SIX),
    property(
        "fit",
        ValueRule::Enumeration(&["cover", "contain", "natural"]),
    ),
    property("captions", BOOLEAN),
];

const INCLUDE: &[PropertyRule] = &[
    required("source", STRING),
    property("mode", ValueRule::Enumeration(&["embed", "inline"])),
    property(
        "headings",
        ValueRule::Enumeration(&["shift", "keep", "strip-title"]),
    ),
    property("show-title", BOOLEAN),
    property("show-source", BOOLEAN),
];

const BUTTON: &[PropertyRule] = &[
    required("label", STRING),
    required("href", STRING),
    property(
        "variant",
        ValueRule::Enumeration(&["primary", "secondary", "quiet", "card"]),
    ),
    property("external", BOOLEAN),
    property("icon", STRING),
];

const SECTION: &[PropertyRule] = &[
    property(
        "width",
        ValueRule::Enumeration(&["narrow", "normal", "wide", "full"]),
    ),
    property(
        "tone",
        ValueRule::Enumeration(&["plain", "subtle", "brand", "success", "warning", "danger"]),
    ),
    property(
        "align",
        ValueRule::Enumeration(&["left", "center", "right"]),
    ),
    property("id", STRING),
];

const COLUMNS: &[PropertyRule] = &[
    required(
        "count",
        ValueRule::Integer {
            minimum: 2,
            maximum: Some(4),
            allow_negative_one: false,
        },
    ),
    property("gap", ValueRule::Enumeration(&["small", "normal", "large"])),
    property(
        "collapse-at",
        ValueRule::Enumeration(&["sm", "md", "lg", "never"]),
    ),
];

const SPECS: &[DirectiveSpec] = &[
    DirectiveSpec {
        name: "page",
        form: DirectiveForm::Leaf,
        properties: PAGE,
    },
    DirectiveSpec {
        name: "hero",
        form: DirectiveForm::Leaf,
        properties: HERO,
    },
    DirectiveSpec {
        name: "breadcrumbs",
        form: DirectiveForm::Leaf,
        properties: BREADCRUMBS,
    },
    DirectiveSpec {
        name: "meta",
        form: DirectiveForm::Leaf,
        properties: META,
    },
    DirectiveSpec {
        name: "toc",
        form: DirectiveForm::Leaf,
        properties: TOC,
    },
    DirectiveSpec {
        name: "children",
        form: DirectiveForm::Leaf,
        properties: CHILDREN,
    },
    DirectiveSpec {
        name: "related",
        form: DirectiveForm::Leaf,
        properties: RELATED,
    },
    DirectiveSpec {
        name: "backlinks",
        form: DirectiveForm::Leaf,
        properties: BACKLINKS,
    },
    DirectiveSpec {
        name: "gallery",
        form: DirectiveForm::Leaf,
        properties: GALLERY,
    },
    DirectiveSpec {
        name: "include",
        form: DirectiveForm::Leaf,
        properties: INCLUDE,
    },
    DirectiveSpec {
        name: "button",
        form: DirectiveForm::Leaf,
        properties: BUTTON,
    },
    DirectiveSpec {
        name: "section",
        form: DirectiveForm::Container,
        properties: SECTION,
    },
    DirectiveSpec {
        name: "columns",
        form: DirectiveForm::Container,
        properties: COLUMNS,
    },
    DirectiveSpec {
        name: "column",
        form: DirectiveForm::Container,
        properties: &[],
    },
];

const DIRECTIVE_NAMES: &[&str] = &[
    "page",
    "hero",
    "breadcrumbs",
    "meta",
    "toc",
    "children",
    "related",
    "backlinks",
    "gallery",
    "include",
    "button",
    "section",
    "columns",
    "column",
];

fn directive_spec(name: &str) -> Option<&'static DirectiveSpec> {
    SPECS.iter().find(|spec| spec.name == name)
}

fn validate_value(value: &DirectiveValue, rule: ValueRule) -> Result<(), String> {
    match (value, rule) {
        (DirectiveValue::String(_), ValueRule::String)
        | (DirectiveValue::Boolean(_), ValueRule::Boolean) => Ok(()),
        (DirectiveValue::Array(values), ValueRule::StringArray)
            if values
                .iter()
                .all(|value| matches!(value, DirectiveScalar::String(_))) =>
        {
            Ok(())
        }
        (DirectiveValue::String(value), ValueRule::Enumeration(allowed)) => {
            if allowed.contains(&value.as_str()) {
                Ok(())
            } else {
                Err(format!("expected one of {}", format_allowed(allowed)))
            }
        }
        (
            DirectiveValue::Number(number),
            ValueRule::Integer {
                minimum,
                maximum,
                allow_negative_one,
            },
        ) => {
            let Some(value) = number.as_i64() else {
                return Err("expected an integer".to_owned());
            };
            if allow_negative_one && value == -1 {
                return Ok(());
            }
            if value < minimum || maximum.is_some_and(|maximum| value > maximum) {
                let expected = maximum.map_or_else(
                    || format!("an integer greater than or equal to {minimum}"),
                    |maximum| format!("an integer from {minimum} to {maximum}"),
                );
                return Err(format!("expected {expected}"));
            }
            Ok(())
        }
        (_, ValueRule::String) => Err("expected a quoted string".to_owned()),
        (_, ValueRule::Boolean) => Err("expected a boolean".to_owned()),
        (_, ValueRule::StringArray) => Err("expected an array of strings".to_owned()),
        (_, ValueRule::Enumeration(allowed)) => {
            Err(format!("expected one of {}", format_allowed(allowed)))
        }
        (_, ValueRule::Integer { .. }) => Err("expected an integer".to_owned()),
    }
}

fn integer_property(properties: &BTreeMap<String, DirectiveValue>, name: &str) -> Option<i64> {
    match properties.get(name) {
        Some(DirectiveValue::Number(number)) => number.as_i64(),
        _ => None,
    }
}

fn format_allowed(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn form_name(form: DirectiveForm) -> &'static str {
    match form {
        DirectiveForm::Leaf => "leaf",
        DirectiveForm::Container => "container",
    }
}

fn closest_name<'names>(value: &str, names: &'names [&str]) -> Option<&'names str> {
    names
        .iter()
        .min_by_key(|candidate| levenshtein(value, candidate))
        .copied()
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right: Vec<_> = right.chars().collect();
    let mut previous: Vec<_> = (0..=right.len()).collect();
    for (left_index, left_character) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            current.push(
                (current[right_index] + 1)
                    .min(previous[right_index + 1] + 1)
                    .min(previous[right_index] + usize::from(left_character != *right_character)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

fn directive_source_span(
    span: DirectiveSpan,
    context: DirectiveValidationContext<'_>,
) -> Option<SourceSpan> {
    let start = span.start.checked_sub(context.body_start_byte)?;
    let end = span.end.checked_sub(context.body_start_byte)?;
    if start > end || end > context.body.len() {
        return None;
    }
    let start_position = source_position(context.body, start, context.body_start_line)?;
    let end_position = source_position(context.body, end, context.body_start_line)?;
    Some(SourceSpan {
        start: start_position,
        end: end_position,
        start_byte: Some(span.start),
        end_byte: Some(span.end),
    })
}

fn source_position(source: &str, offset: usize, first_line: usize) -> Option<SourcePosition> {
    if offset > source.len() || !source.is_char_boundary(offset) {
        return None;
    }
    let prefix = &source[..offset];
    let bytes = prefix.as_bytes();
    let mut line_breaks = 0;
    let mut line_start = 0;
    let mut cursor = 0;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\n' => {
                cursor += 1;
                line_breaks += 1;
                line_start = cursor;
            }
            b'\r' => {
                cursor += 1;
                if bytes.get(cursor) == Some(&b'\n') {
                    cursor += 1;
                }
                line_breaks += 1;
                line_start = cursor;
            }
            _ => cursor += 1,
        }
    }
    Some(SourcePosition {
        line: first_line + line_breaks,
        column: source[line_start..offset].chars().count() + 1,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{Compiler, Config, parse_container_directive_info, parse_leaf_directive};
    use tempfile::tempdir;

    fn leaf(source: &str) -> MarkdownNode {
        let invocation = parse_leaf_directive(source, 0).directive.unwrap();
        MarkdownNode {
            kind: NodeKind::Directive {
                invocation,
                fence_length: None,
            },
            span: None,
            block_id: None,
            children: Vec::new(),
        }
    }

    fn container(info: &str, children: Vec<MarkdownNode>) -> MarkdownNode {
        let invocation = parse_container_directive_info(info, 0).directive.unwrap();
        MarkdownNode {
            kind: NodeKind::Directive {
                invocation,
                fence_length: Some(3),
            },
            span: None,
            block_id: None,
            children,
        }
    }

    fn document(children: Vec<MarkdownNode>) -> MarkdownNode {
        MarkdownNode {
            kind: NodeKind::Document,
            span: None,
            block_id: None,
            children,
        }
    }

    fn validate(root: &MarkdownNode, body: &str, is_index: bool) -> DirectiveValidationOutcome {
        validate_directives(
            root,
            DirectiveValidationContext {
                logical_path: if is_index { "index.md" } else { "page.md" },
                body,
                body_start_line: 1,
                body_start_byte: 0,
                is_index,
            },
        )
    }

    #[test]
    fn applies_only_explicit_documented_defaults() {
        let root = document(vec![
            leaf("::page{}"),
            leaf("::hero{}"),
            leaf("::children{}"),
        ]);
        let outcome = validate(&root, "::page{}\n::hero{}\n::children{}", true);
        assert_eq!(outcome.diagnostics, []);
        assert_eq!(
            outcome.directives[0].properties["layout"],
            DirectiveValue::String("default".into())
        );
        assert!(!outcome.directives[0].properties.contains_key("width"));
        assert_eq!(
            outcome.directives[1].properties["show-title"],
            DirectiveValue::Boolean(true)
        );
        assert_eq!(
            outcome.directives[2].properties["depth"],
            DirectiveValue::Number(1.into())
        );
        assert!(!outcome.directives[2].properties.contains_key("direction"));
    }

    #[test]
    fn accepts_an_explicit_children_source() {
        let source = "::children{source=\"/project/\" view=\"grid\" limit=3}";
        let root = document(vec![leaf(source)]);
        let outcome = validate(&root, source, true);

        assert!(outcome.diagnostics.is_empty());
        assert_eq!(
            outcome.directives[0].properties["source"],
            DirectiveValue::String("/project/".into())
        );
    }

    #[test]
    fn accepts_a_card_button_variant() {
        let source = "::button{label=\"Contact\" href=\"/contact/\" variant=\"card\"}";
        let root = document(vec![leaf(source)]);
        let outcome = validate(&root, source, true);

        assert!(outcome.diagnostics.is_empty());
        assert_eq!(
            outcome.directives[0].properties["variant"],
            DirectiveValue::String("card".into())
        );
    }

    #[test]
    fn diagnoses_unknown_names_properties_and_form() {
        let root = document(vec![
            leaf("::unknown{}"),
            leaf("::hero{show-titel=true}"),
            leaf("::section{}"),
        ]);
        let outcome = validate(
            &root,
            "::unknown{}\n::hero{show-titel=true}\n::section{}",
            true,
        );
        let codes: Vec<_> = outcome
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, ["MS3501", "MS3503", "MS3502"]);
        assert_eq!(
            outcome.diagnostics[1].help.as_deref(),
            Some("did you mean `show-title`?")
        );
    }

    #[test]
    fn validates_types_enums_ranges_required_values_and_cross_rules() {
        let root = document(vec![
            leaf("::button{label=\"Open\"}"),
            leaf("::children{columns=7 show=[\"title\",2]}"),
            leaf("::toc{min-depth=5 max-depth=2}"),
            leaf("::hero{align=\"bottom\"}"),
        ]);
        let outcome = validate(&root, "", true);
        let codes: Vec<_> = outcome
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, ["MS3506", "MS3504", "MS3504", "MS3505", "MS3504"]);
    }

    #[test]
    fn validates_page_and_index_context() {
        let root = document(vec![
            leaf("::hero{}"),
            leaf("::page{}"),
            leaf("::page{}"),
            leaf("::children{}"),
        ]);
        let outcome = validate(&root, "", false);
        let codes: Vec<_> = outcome
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, ["MS3508", "MS3507", "MS3508", "MS3509"]);
    }

    #[test]
    fn validates_columns_direct_children_and_track_count() {
        let good = container(
            "columns{count=2}",
            vec![container("column", vec![]), container("column", vec![])],
        );
        assert_eq!(validate(&document(vec![good]), "", true).diagnostics, []);

        let wrapped = container(
            "columns{count=2}",
            vec![
                container("column", vec![]),
                container("column", vec![]),
                container("column", vec![]),
                container("column", vec![]),
            ],
        );
        assert_eq!(validate(&document(vec![wrapped]), "", true).diagnostics, []);

        let bad = container(
            "columns{count=2}",
            vec![container("column", vec![]), container("section", vec![])],
        );
        let outcome = validate(&document(vec![bad]), "", true);
        let codes: Vec<_> = outcome
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, ["MS3510", "MS3510"]);

        let orphan = container("column", vec![]);
        assert_eq!(
            validate(&document(vec![orphan]), "", true).diagnostics[0].code,
            "MS3509"
        );
    }

    #[test]
    fn property_diagnostics_use_original_byte_and_line_spans() {
        let source = "\n::hero{show-titel=true}";
        let root = document(vec![leaf(&source[1..])]);
        let mut root = root;
        if let NodeKind::Directive { invocation, .. } = &mut root.children[0].kind {
            invocation.span.start += 1;
            invocation.span.end += 1;
            invocation.name_span.start += 1;
            invocation.name_span.end += 1;
            for property in &mut invocation.properties {
                property.span.start += 1;
                property.span.end += 1;
                property.name_span.start += 1;
                property.name_span.end += 1;
                property.value_span.start += 1;
                property.value_span.end += 1;
            }
        }
        let outcome = validate(&root, source, true);
        let span = outcome.diagnostics[0].primary.as_ref().unwrap().span;
        assert_eq!(span.start.line, 2);
        assert_eq!(span.start.column, 8);
        assert_eq!(span.start_byte, Some(8));
        assert_eq!(span.end_byte, Some(18));
    }

    #[test]
    fn property_diagnostics_count_lone_cr_as_a_line_ending() {
        let source = "\r::hero{show-titel=true}";
        let root = document(vec![leaf(&source[1..])]);
        let mut root = root;
        if let NodeKind::Directive { invocation, .. } = &mut root.children[0].kind {
            invocation.span.start += 1;
            invocation.span.end += 1;
            invocation.name_span.start += 1;
            invocation.name_span.end += 1;
            for property in &mut invocation.properties {
                property.span.start += 1;
                property.span.end += 1;
                property.name_span.start += 1;
                property.name_span.end += 1;
                property.value_span.start += 1;
                property.value_span.end += 1;
            }
        }
        let span = validate(&root, source, true).diagnostics[0]
            .primary
            .as_ref()
            .unwrap()
            .span;
        assert_eq!(span.start.line, 2);
        assert_eq!(span.start.column, 8);
    }

    #[test]
    fn compiler_reports_registry_diagnostics_after_dialect_lowering() {
        let temporary = tempdir().unwrap();
        let docs = temporary.path().join("docs");
        fs::create_dir(&docs).unwrap();
        fs::write(docs.join("index.md"), "::herp{}\n\n# Home\n").unwrap();
        let config = Config::from_toml(
            "schema=1\ncontent_root=\"docs\"",
            temporary.path().join("mambo.toml"),
        )
        .unwrap();

        let outcome = Compiler::new(config).compile();
        assert!(outcome.site.is_none());
        assert_eq!(
            outcome
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "MS3501")
                .count(),
            1
        );
    }
}
