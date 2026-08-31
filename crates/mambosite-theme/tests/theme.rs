use std::fs;
use std::path::Path;

use mambosite_theme::{Theme, ThemeError, compile_theme_file, default_theme_toml};

#[test]
fn empty_file_resolves_to_the_generic_default() {
    let theme = Theme::from_toml("", "mambo.theme.toml").unwrap();

    assert_eq!(theme, Theme::default());
    assert!(theme.fonts.faces.is_empty());
    assert!(!theme.fonts.body.contains("MamboFont"));
}

#[test]
fn canonical_default_toml_round_trips() {
    let source = default_theme_toml().unwrap();
    let parsed = Theme::from_toml(&source, "mambo.theme.toml").unwrap();

    assert_eq!(parsed, Theme::default());
    assert!(source.contains("[breakpoints]"));
    assert!(source.contains("[components.header]"));
    assert!(source.contains("[typography.heading_1]"));
}

#[test]
fn nested_settings_override_without_erasing_default_siblings() {
    let theme = Theme::from_toml(
        r##"
            id = "folio"
            default_scheme = "light"

            [breakpoints]
            compact = 600
            content = 800
            wide = 1000

            [colors.light]
            background = "#ffffff"

            [typography.body]
            size = { base = "0.95rem", content = "1.1rem" }

            [components.collection]
            max_columns = { base = 1, compact = 2, content = 4, wide = 6 }

            [components.sidebar]
            mode = { base = "hidden", content = "inline" }

            [components.header]
            mode = { base = "compact", content = "full" }
            clock = { base = "hidden", content = "visible" }
            hide_after = 96
        "##,
        "mambo.theme.toml",
    )
    .unwrap();
    let output = theme.compile().unwrap();

    assert_eq!(theme.colors.light.surface, "#f3ece2");
    assert_eq!(theme.typography.body.line_height, "1.78");
    assert!(output.css.contains(":root,\n[data-theme=\"light\"]"));
    assert!(output.css.contains("@media (min-width: 600px)"));
    assert!(output.css.contains("@media (min-width: 800px)"));
    assert!(output.css.contains("@media (min-width: 1000px)"));
    assert!(output.css.contains("--mambo-type-body-size: 1.1rem;"));
    assert!(output.css.contains("--mambo-collection-max-columns: 6;"));
    assert!(output.css.contains("--mambo-sidebar-position: static;"));
    assert!(
        output
            .css
            .contains("--mambo-header-grid-template: auto 1fr auto;")
    );
    assert!(!output.css.contains("min-width: var("));
    assert!(output.typescript.contains("\"headerHideAfter\": 96"));
}

#[test]
fn emits_configured_font_faces_and_semantic_token_groups() {
    let theme = Theme::from_toml(
        r#"
            [fonts]
            body = '"Example", sans-serif'

            [[fonts.faces]]
            family = "Example"
            source = "/assets/example.woff2"
            weight = 600
            style = "italic"
            format = "woff2"
            display = "swap"
        "#,
        "mambo.theme.toml",
    )
    .unwrap();
    let css = theme.compile().unwrap().css;

    assert!(css.contains("@font-face"));
    assert!(css.contains("font-family: \"Example\";"));
    assert!(css.contains("url(\"/assets/example.woff2\") format(\"woff2\")"));
    assert!(css.contains("font-weight: 600;"));
    assert!(css.contains("font-style: italic;"));
    for variable in [
        "--mambo-color-background",
        "--mambo-font-body",
        "--mambo-type-heading-1-size",
        "--mambo-space-md",
        "--mambo-width-reading",
        "--mambo-dimension-header-height",
        "--mambo-border-strong",
        "--mambo-radius-medium",
        "--mambo-shadow-medium",
        "--mambo-motion-normal",
    ] {
        assert!(css.contains(variable), "missing {variable}");
    }
}

#[test]
fn reports_all_semantic_validation_failures() {
    let error = Theme::from_toml(
        r##"
            schema = 2
            id = "Not Valid"
            extends = "remote"

            [breakpoints]
            compact = 900
            content = 800
            wide = 700

            [colors.dark]
            background = "red; color: blue"
            accents = ["#fff"]

            [components.collection]
            max_columns = 9
        "##,
        "broken.theme.toml",
    )
    .unwrap_err();
    let ThemeError::Validation { diagnostics, .. } = error else {
        panic!("expected validation error");
    };
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();

    assert!(codes.contains(&"MST1001"));
    assert!(codes.contains(&"MST1002"));
    assert!(codes.contains(&"MST1003"));
    assert!(codes.contains(&"MST1004"));
    assert!(codes.contains(&"MST1103"));
    assert!(codes.contains(&"MST1105"));
    assert!(codes.contains(&"MST1106"));
}

#[test]
fn unknown_fields_and_incomplete_font_faces_fail_during_parsing() {
    for source in [
        "mystery = true",
        "[[fonts.faces]]\nfamily = \"MissingSource\"",
    ] {
        assert!(matches!(
            Theme::from_toml(source, "broken.theme.toml"),
            Err(ThemeError::Parse { .. })
        ));
    }
}

#[test]
fn compilation_is_byte_for_byte_deterministic() {
    let theme = Theme::from_toml("id = \"deterministic\"", "mambo.theme.toml").unwrap();

    assert_eq!(theme.compile().unwrap(), theme.compile().unwrap());
}

#[test]
fn file_api_loads_without_owning_output_policy() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("mambo.theme.toml");
    fs::write(&path, "id = \"file-theme\"").unwrap();

    let output = compile_theme_file(&path).unwrap();

    assert_eq!(output.theme.id, "file-theme");
    assert!(output.css.starts_with("/* Generated by MamboSite."));
    assert!(!Path::new(directory.path()).join("theme.css").exists());
}
