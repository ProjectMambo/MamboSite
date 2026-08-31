use std::path::Path;

use unicode_normalization::UnicodeNormalization;

/// Normalize one human-authored filename or slug into a URL segment.
pub fn slugify_segment(input: &str) -> String {
    let mut output = String::new();
    let mut separator_pending = false;
    for character in input.nfc() {
        if character.is_ascii_alphanumeric() {
            if separator_pending && !output.is_empty() {
                output.push('-');
            }
            separator_pending = false;
            output.push(character.to_ascii_lowercase());
        } else if character.is_alphanumeric() {
            if separator_pending && !output.is_empty() {
                output.push('-');
            }
            separator_pending = false;
            output.push(character);
        } else {
            separator_pending = !output.is_empty();
        }
    }
    output
}

/// Derive a route from a source path relative to its physical or mounted root.
///
/// # Errors
///
/// Returns a message when the path or explicit slug cannot produce a valid
/// route.
pub fn derive_route(
    relative_source: &str,
    slug: Option<&str>,
    trailing_slash: bool,
) -> Result<String, String> {
    let path = Path::new(relative_source);
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "source path has no UTF-8 filename".to_owned())?;
    if Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        != Some("md")
    {
        return Err("source file does not have the exact .md extension".to_owned());
    }

    let is_index = file_name == "index.md";
    let mut raw_segments: Vec<String> = path
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    if !is_index {
        raw_segments.push(file_name.trim_end_matches(".md").to_owned());
    }

    if let Some(slug) = slug {
        let slug = slugify_segment(slug);
        if slug.is_empty() {
            return Err("frontmatter slug produces an empty route segment".to_owned());
        }
        if let Some(last) = raw_segments.last_mut() {
            *last = slug;
        } else {
            return Err("the site root index cannot override its slug".to_owned());
        }
    }

    let mut segments = Vec::with_capacity(raw_segments.len());
    for segment in raw_segments {
        let segment = slugify_segment(&segment);
        if segment.is_empty() {
            return Err("a source path segment produces an empty route segment".to_owned());
        }
        segments.push(segment);
    }
    let route = format!("/{}", segments.join("/"));
    normalize_route(&route, trailing_slash)
}

/// Normalize an explicitly configured absolute route.
///
/// # Errors
///
/// Returns a message when the route is not absolute or contains unsafe or
/// empty normalized segments.
pub fn normalize_route(route: &str, trailing_slash: bool) -> Result<String, String> {
    if !route.starts_with('/') || route.contains(['?', '#', '\\']) {
        return Err(
            "route must begin with `/` and contain no query, fragment, or backslash".to_owned(),
        );
    }
    let mut segments = Vec::new();
    for raw in route.split('/').filter(|segment| !segment.is_empty()) {
        if matches!(raw, "." | "..") {
            return Err("route may not contain `.` or `..` segments".to_owned());
        }
        let segment = slugify_segment(raw);
        if segment.is_empty() {
            return Err("route contains an empty normalized segment".to_owned());
        }
        segments.push(segment);
    }
    if segments.is_empty() {
        return Ok("/".to_owned());
    }
    let mut result = format!("/{}", segments.join("/"));
    if trailing_slash {
        result.push('/');
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{derive_route, normalize_route, slugify_segment};

    #[test]
    fn normalizes_filenames_and_unicode() {
        assert_eq!(slugify_segment(" My__Project  Notes "), "my-project-notes");
        assert_eq!(
            derive_route("第二卷/index.md", None, true).unwrap(),
            "/第二卷/"
        );
        assert_eq!(
            derive_route("Blog/Hello World.md", None, true).unwrap(),
            "/blog/hello-world/"
        );
    }

    #[test]
    fn handles_root_and_trailing_slashes() {
        assert_eq!(derive_route("index.md", None, true).unwrap(), "/");
        assert_eq!(
            normalize_route("/Docs/Start", false).unwrap(),
            "/docs/start"
        );
    }
}
