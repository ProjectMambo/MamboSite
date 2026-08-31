use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::{Diagnostic, FrontmatterConfig, Mount, PageMetadata, PageStatus, SourceSpan};

#[derive(Debug, Clone)]
pub struct FrontmatterOutcome {
    pub metadata: PageMetadata,
    pub body: String,
    /// One-based source line on which `body` begins.
    pub body_start_line: usize,
    /// Zero-based byte offset of `body` in the original source.
    pub body_start_byte: usize,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_frontmatter(
    source: &str,
    logical_path: &str,
    options: &FrontmatterConfig,
) -> FrontmatterOutcome {
    let (source, bom_bytes) = source
        .strip_prefix('\u{feff}')
        .map_or((source, 0), |source| (source, '\u{feff}'.len_utf8()));
    let mut chunks = line_chunks(source).into_iter();
    let Some(first) = chunks.next() else {
        return FrontmatterOutcome {
            metadata: PageMetadata::default(),
            body: String::new(),
            body_start_line: 1,
            body_start_byte: bom_bytes,
            diagnostics: Vec::new(),
        };
    };
    if trim_line_ending(first) != "---" {
        return FrontmatterOutcome {
            metadata: PageMetadata::default(),
            body: source.to_owned(),
            body_start_line: 1,
            body_start_byte: bom_bytes,
            diagnostics: Vec::new(),
        };
    }

    let mut offset = first.len();
    let yaml_start = offset;
    let mut closing = None;
    for (index, chunk) in chunks.enumerate() {
        if trim_line_ending(chunk) == "---" {
            closing = Some((offset, offset + chunk.len(), index + 2));
            break;
        }
        offset += chunk.len();
    }

    let Some((yaml_end, body_offset, closing_line)) = closing else {
        return FrontmatterOutcome {
            metadata: PageMetadata::default(),
            body: source.to_owned(),
            body_start_line: 1,
            body_start_byte: bom_bytes,
            diagnostics: vec![
                Diagnostic::error(
                    "MS2201",
                    "frontmatter is missing its closing `---` delimiter",
                )
                .at(logical_path, SourceSpan::point(1, 1)),
            ],
        };
    };

    let yaml = &source[yaml_start..yaml_end];
    let body = source[body_offset..].to_owned();
    let body_start_line = closing_line + 1;
    let body_start_byte = body_offset + bom_bytes;
    let mut diagnostics = Vec::new();
    let value = if yaml.trim().is_empty() {
        Value::Object(Map::new())
    } else {
        match parse_yaml_value(yaml) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(
                    Diagnostic::error("MS2202", format!("invalid YAML frontmatter: {error}"))
                        .at(logical_path, SourceSpan::point(2, 1)),
                );
                Value::Object(Map::new())
            }
        }
    };

    let metadata = if let Value::Object(map) = value {
        normalize_map(map, logical_path, options, &mut diagnostics)
    } else {
        diagnostics.push(
            Diagnostic::error("MS2203", "frontmatter must be a YAML mapping")
                .at(logical_path, SourceSpan::point(2, 1)),
        );
        PageMetadata::default()
    };

    FrontmatterOutcome {
        metadata,
        body,
        body_start_line,
        body_start_byte,
        diagnostics,
    }
}

fn parse_yaml_value(yaml: &str) -> Result<Value, String> {
    validate_yaml_subset(yaml)?;

    let mut options = serde_saphyr::Options::default();
    options.merge_keys = serde_saphyr::MergeKeyPolicy::Error;
    options.strict_booleans = true;
    options.alias_limits = serde_saphyr::alias_limits! {
        max_total_replayed_events: 0,
        max_replay_stack_depth: 0,
        max_alias_expansions_per_anchor: 0,
    };
    serde_saphyr::from_str_with_options(yaml, options).map_err(|error| error.to_string())
}

fn validate_yaml_subset(yaml: &str) -> Result<(), String> {
    use serde_saphyr::granit_parser::{Event, Parser};

    for event in Parser::new_from_str(yaml).keep_tags(true) {
        let (event, _) = event.map_err(|error| error.to_string())?;
        match event {
            Event::Alias(_) => return Err("YAML aliases are not supported".to_owned()),
            Event::Scalar(_, _, anchor, tag)
            | Event::SequenceStart(_, anchor, tag)
            | Event::MappingStart(_, anchor, tag) => {
                if anchor != 0 {
                    return Err("YAML anchors are not supported".to_owned());
                }
                if tag.is_some() {
                    return Err("explicit YAML tags are not supported".to_owned());
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn normalize_map(
    mut map: Map<String, Value>,
    path: &str,
    options: &FrontmatterConfig,
    diagnostics: &mut Vec<Diagnostic>,
) -> PageMetadata {
    let mut metadata = PageMetadata {
        title: take_optional_string(&mut map, "title", path, diagnostics),
        description: take_optional_string(&mut map, "description", path, diagnostics),
        slug: take_optional_string(&mut map, "slug", path, diagnostics),
        date: take_optional_string(&mut map, "date", path, diagnostics),
        updated: take_optional_string(&mut map, "updated", path, diagnostics),
        cover: take_optional_string(&mut map, "cover", path, diagnostics),
        tags: take_string_list(&mut map, "tags", true, path, diagnostics),
        aliases: take_string_list(&mut map, "aliases", true, path, diagnostics),
        ..PageMetadata::default()
    };

    if let Some(value) = map.remove("status") {
        metadata.status = match value.as_str() {
            Some("published") => PageStatus::Published,
            Some("draft") => PageStatus::Draft,
            None if value.is_null() => PageStatus::Published,
            _ => {
                wrong_type(path, "status", "`published` or `draft`", diagnostics);
                PageStatus::Published
            }
        };
    }
    if let Some(value) = map.remove("listed") {
        match value.as_bool() {
            Some(value) => metadata.listed = value,
            None if value.is_null() => {}
            None => wrong_type(path, "listed", "a boolean", diagnostics),
        }
    }
    if let Some(value) = map.remove("order") {
        match value.as_i64() {
            Some(value) => metadata.order = Some(value),
            None if value.is_null() => {}
            None => wrong_type(path, "order", "an integer", diagnostics),
        }
    }
    if let Some(value) = map.remove("data") {
        match value {
            Value::Object(values) => metadata.data = values.into_iter().collect(),
            Value::Null => {}
            _ => wrong_type(path, "data", "a mapping", diagnostics),
        }
    }
    if let Some(value) = map.remove("mounts") {
        metadata.mounts = normalize_mounts(value, path, diagnostics);
    }

    let ignored: BTreeSet<_> = options.ignored_fields.iter().map(String::as_str).collect();
    for field in &options.legacy_data_fields {
        if let Some(value) = map.remove(field) {
            if !value.is_null() {
                metadata.data.entry(field.clone()).or_insert(value);
            }
        }
    }

    for (key, value) in map {
        if ignored.contains(key.as_str()) {
            continue;
        }
        metadata.extra.insert(key.clone(), value);
        if options.strict {
            diagnostics.push(
                Diagnostic::error(
                    "MS2206",
                    format!("unknown frontmatter field `{key}`"),
                )
                .at_path(path)
                .with_help("move site-specific values beneath `data`, or configure this compatibility field explicitly"),
            );
        }
    }
    metadata
}

fn take_optional_string(
    map: &mut Map<String, Value>,
    field: &str,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    match map.remove(field) {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_owned())
        }
        Some(_) => {
            wrong_type(path, field, "a string", diagnostics);
            None
        }
    }
}

fn take_string_list(
    map: &mut Map<String, Value>,
    field: &str,
    allow_scalar: bool,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<String> {
    let Some(value) = map.remove(field) else {
        return Vec::new();
    };
    match value {
        Value::Null => Vec::new(),
        Value::String(value) if allow_scalar => nonempty(&value).into_iter().collect(),
        Value::Array(values) => {
            let mut result = Vec::new();
            for value in values {
                match value {
                    Value::String(value) => result.extend(nonempty(&value)),
                    _ => wrong_type(path, field, "a string array", diagnostics),
                }
            }
            result
        }
        _ => {
            wrong_type(path, field, "a string array", diagnostics);
            Vec::new()
        }
    }
}

fn normalize_mounts(value: Value, path: &str, diagnostics: &mut Vec<Diagnostic>) -> Vec<Mount> {
    let Value::Array(values) = value else {
        if !value.is_null() {
            wrong_type(path, "mounts", "an array of mappings", diagnostics);
        }
        return Vec::new();
    };
    let mut mounts = Vec::new();
    for (index, value) in values.into_iter().enumerate() {
        let Value::Object(mut fields) = value else {
            diagnostics.push(
                Diagnostic::error("MS2204", format!("mount {} must be a mapping", index + 1))
                    .at_path(path),
            );
            continue;
        };
        let route = match fields.remove("path") {
            Some(Value::String(value)) if !value.trim().is_empty() => Some(value),
            _ => None,
        };
        let source = match fields.remove("source") {
            Some(Value::String(value)) if !value.trim().is_empty() => Some(value),
            _ => None,
        };
        match (route, source) {
            (Some(route), Some(source)) if fields.is_empty() => mounts.push(Mount {
                path: route,
                source: source.replace('\\', "/"),
            }),
            _ => diagnostics.push(
                Diagnostic::error(
                    "MS2205",
                    format!(
                        "mount {} requires only string `path` and `source` fields",
                        index + 1
                    ),
                )
                .at_path(path),
            ),
        }
    }
    mounts
}

fn nonempty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn wrong_type(path: &str, field: &str, expected: &str, diagnostics: &mut Vec<Diagnostic>) {
    diagnostics.push(
        Diagnostic::error(
            "MS2204",
            format!("frontmatter `{field}` must be {expected}"),
        )
        .at_path(path),
    );
}

fn trim_line_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .or_else(|| line.strip_suffix('\r'))
        .unwrap_or(line)
}

fn line_chunks(source: &str) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut cursor = 0;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\n' => {
                cursor += 1;
                chunks.push(&source[start..cursor]);
                start = cursor;
            }
            b'\r' => {
                cursor += 1;
                if bytes.get(cursor) == Some(&b'\n') {
                    cursor += 1;
                }
                chunks.push(&source[start..cursor]);
                start = cursor;
            }
            _ => cursor += 1,
        }
    }
    if start < source.len() {
        chunks.push(&source[start..]);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::parse_frontmatter;
    use crate::FrontmatterConfig;

    #[test]
    fn parses_yaml_and_preserves_body_line_numbers() {
        let parsed = parse_frontmatter(
            "---\ntitle: Hello\ntags: rust\nperiod: now\n---\n# Body\n",
            "index.md",
            &FrontmatterConfig::default(),
        );
        assert_eq!(parsed.metadata.title.as_deref(), Some("Hello"));
        assert_eq!(parsed.metadata.tags, ["rust"]);
        assert_eq!(parsed.metadata.data["period"], "now");
        assert_eq!(parsed.body, "# Body\n");
        assert_eq!(parsed.body_start_line, 6);
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn reports_unclosed_frontmatter() {
        let parsed = parse_frontmatter(
            "---\ntitle: Never closed\n",
            "bad.md",
            &FrontmatterConfig::default(),
        );
        assert_eq!(parsed.diagnostics[0].code, "MS2201");
    }

    #[test]
    fn supports_crlf_delimiters() {
        let parsed = parse_frontmatter(
            "---\r\ntitle: Windows\r\n---\r\nText\r\n",
            "page.md",
            &FrontmatterConfig::default(),
        );
        assert_eq!(parsed.metadata.title.as_deref(), Some("Windows"));
        assert_eq!(parsed.body_start_line, 4);
    }

    #[test]
    fn supports_bom_and_lone_cr_delimiters_without_changing_offsets() {
        let source = "\u{feff}---\rtitle: CR page\r---\r# Body\r";
        let parsed = parse_frontmatter(source, "index.md", &FrontmatterConfig::default());
        assert_eq!(parsed.metadata.title.as_deref(), Some("CR page"));
        assert_eq!(parsed.body, "# Body\r");
        assert_eq!(parsed.body_start_line, 4);
        assert_eq!(parsed.body_start_byte, source.find("# Body").unwrap());
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn validates_status_and_strict_unknown_fields() {
        let options = FrontmatterConfig {
            strict: true,
            ..FrontmatterConfig::default()
        };
        let parsed = parse_frontmatter(
            "---\nstatus: published\ncustom: kept\n---\nBody\n",
            "page.md",
            &options,
        );
        assert_eq!(parsed.metadata.status, crate::PageStatus::Published);
        assert_eq!(parsed.metadata.extra["custom"], "kept");
        assert!(parsed.diagnostics.iter().any(|item| item.code == "MS2206"));
    }

    #[test]
    fn rejects_yaml_aliases_and_merge_keys() {
        for source in [
            "---\nbase: &base value\ntitle: *base\n---\n",
            "---\nbase: &base\n  title: inherited\n<<: *base\n---\n",
        ] {
            let parsed = parse_frontmatter(source, "page.md", &FrontmatterConfig::default());
            assert!(parsed.diagnostics.iter().any(|item| item.code == "MS2202"));
        }
    }

    #[test]
    fn rejects_custom_yaml_tags() {
        let parsed = parse_frontmatter(
            "---\ntitle: !custom tagged\n---\n",
            "page.md",
            &FrontmatterConfig::default(),
        );
        assert!(parsed.diagnostics.iter().any(|item| item.code == "MS2202"));
    }
}
