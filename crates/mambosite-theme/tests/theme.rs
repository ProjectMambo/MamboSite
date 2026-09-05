use std::fs;
use std::path::Path;

use mambosite_theme::{SidebarMode, Theme, ThemeError, compile_theme_file, default_theme_toml};

#[test]
fn empty_file_resolves_to_the_generic_default() {
    let theme = Theme::from_toml("", "mambo.theme.toml").unwrap();

    assert_eq!(theme, Theme::default());
    assert!(theme.fonts.faces.is_empty());
    assert!(theme.fonts.body.contains("MamboFont"));
    assert_eq!(theme.components.collection.max_columns.base(), &1);
    assert_eq!(theme.components.collection.max_columns.compact(), Some(&2));
    assert_eq!(theme.components.collection.max_columns.content(), Some(&2));
    assert_eq!(theme.components.collection.max_columns.wide(), Some(&6));
    assert_eq!(theme.components.sidebar.mode.base(), &SidebarMode::Inline);
    assert_eq!(theme.typography.body.size.base(), "1.125rem");
    assert_eq!(theme.typography.navigation.size.base(), "1.2rem");
    assert_eq!(theme.widths.gallery_image_max, "24rem");
    assert_eq!(theme.dimensions.control_min_height, "2.75rem");
    for radius in [
        &theme.radii.small,
        &theme.radii.medium,
        &theme.radii.large,
        &theme.radii.pill,
    ] {
        assert_eq!(radius, "0");
    }
    let css = theme.compile().unwrap().css;
    assert!(css.contains("--mambo-color-brand-active:"));
    assert!(css.contains("--mambo-width-gallery-image-max: 24rem;"));
    assert!(css.contains("--mambo-sidebar-inline-display: block;"));
    assert!(css.contains("--mambo-sidebar-rail-display: none;"));
    assert!(media_section(&css, 900, Some(1200)).contains("--mambo-sidebar-inline-display: none;"));
    assert!(media_section(&css, 900, Some(1200)).contains("--mambo-sidebar-rail-display: block;"));
}

#[test]
fn default_interaction_colours_meet_contrast_thresholds() {
    let theme = Theme::default();
    for palette in [&theme.colors.dark, &theme.colors.light] {
        for colour in [&palette.brand, &palette.brand_hover, &palette.brand_active] {
            assert!(contrast(colour, &palette.background) >= 4.5);
            assert!(contrast(colour, &palette.on_brand) >= 4.5);
        }
        assert!(contrast(&palette.focus, &palette.background) >= 3.0);
    }
}

#[test]
fn canonical_default_toml_round_trips() {
    let source = default_theme_toml().unwrap();
    let parsed = Theme::from_toml(&source, "mambo.theme.toml").unwrap();

    assert_eq!(parsed, Theme::default());
    assert!(source.contains("[breakpoints]"));
    assert!(source.contains("[components.header]"));
    assert!(source.contains("[typography.heading_1]"));
    assert!(source.contains("[layout.page_with_sidebar_columns]"));
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
    assert_eq!(theme.typography.body.line_height, "1.72");
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
    for declaration in [
        "--mambo-header-navigation-display: none;",
        "--mambo-header-navigation-display: flex;",
        "--mambo-header-navigation-direction: column;",
        "--mambo-header-navigation-direction: row;",
        "--mambo-header-navigation-position: absolute;",
        "--mambo-header-navigation-position: static;",
        "--mambo-header-toggle-display: inline-flex;",
        "--mambo-header-toggle-display: none;",
    ] {
        assert!(output.css.contains(declaration));
    }
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
        "--mambo-color-brand-active",
        "--mambo-font-body",
        "--mambo-type-heading-1-size",
        "--mambo-space-md",
        "--mambo-width-reading",
        "--mambo-width-gallery-image-max",
        "--mambo-dimension-header-height",
        "--mambo-border-strong",
        "--mambo-radius-medium",
        "--mambo-shadow-medium",
        "--mambo-motion-normal",
        "--mambo-layout-page-with-sidebar-columns",
        "--mambo-layout-hero-split-columns",
        "--mambo-layout-list-card-columns",
        "--mambo-layout-metadata-columns",
        "--mambo-layout-footer-direction",
        "--mambo-layout-header-inline-padding",
        "--mambo-layout-page-inline-padding",
    ] {
        assert!(css.contains(variable), "missing {variable}");
    }
}

#[test]
fn configured_breakpoints_drive_layout_and_finite_column_rules() {
    let theme = Theme::from_toml(
        r#"
            [breakpoints]
            compact = 520
            content = 760
            wide = 1120

            [layout]
            page_with_sidebar_columns = { base = "1fr", content = "3fr 1fr" }
            hero_split_columns = { base = "1fr", content = "2fr 3fr" }
            list_card_columns = { base = "1fr", compact = "9rem 1fr" }
            metadata_columns = { base = "1fr", compact = "6rem 1fr" }
            footer_direction = { base = "column", wide = "row" }
            header_inline_padding = { base = "0.5rem", compact = "2rem" }
            page_inline_padding = { base = "0.75rem", content = "3rem" }
            footer_inline_padding = { base = "1rem", wide = "4rem" }

            [components.collection]
            max_columns = { base = 1, compact = 3, content = 4, wide = 5 }
        "#,
        "responsive.theme.toml",
    )
    .unwrap();
    let css = theme.compile().unwrap().css;
    let compact = media_section(&css, 520, Some(760));
    let content = media_section(&css, 760, Some(1120));
    let wide = media_section(&css, 1120, None);

    assert!(css.contains("--mambo-layout-page-with-sidebar-columns: 1fr;"));
    assert!(css.contains("--mambo-layout-footer-direction: column;"));
    assert!(compact.contains("--mambo-layout-list-card-columns: 9rem 1fr;"));
    assert!(compact.contains("--mambo-layout-metadata-columns: 6rem 1fr;"));
    assert!(compact.contains("--mambo-layout-header-inline-padding: 2rem;"));
    assert!(content.contains("--mambo-layout-page-with-sidebar-columns: 3fr 1fr;"));
    assert!(content.contains("--mambo-layout-hero-split-columns: 2fr 3fr;"));
    assert!(content.contains("--mambo-layout-page-inline-padding: 3rem;"));
    assert!(wide.contains("--mambo-layout-footer-direction: row;"));
    assert!(wide.contains("--mambo-layout-footer-inline-padding: 4rem;"));

    assert!(css.contains(
        "[data-mambo-collection][data-columns=\"6\"] { --mambo-collection-columns: 1; }"
    ));
    assert!(compact.contains(
        "[data-mambo-collection][data-columns=\"6\"] { --mambo-collection-columns: 3; }"
    ));
    assert!(content.contains(
        "[data-mambo-collection][data-columns=\"6\"] { --mambo-collection-columns: 4; }"
    ));
    assert!(wide.contains(
        "[data-mambo-collection][data-columns=\"6\"] { --mambo-collection-columns: 5; }"
    ));
    assert_eq!(
        css.lines()
            .filter(|line| line.contains("--mambo-collection-columns:"))
            .count(),
        24
    );
    assert!(css.contains(
        ":is([data-mambo-collection], [data-mambo-columns]) > [data-mambo-accent-item]:nth-child("
    ));

    assert!(css.contains("[data-mambo-columns][data-collapse=\"never\"]"));
    assert!(compact.contains("[data-mambo-columns][data-collapse=\"compact\"]"));
    assert!(content.contains("[data-mambo-columns][data-collapse=\"content\"]"));
    assert!(wide.contains("[data-mambo-columns][data-collapse=\"wide\"]"));
    assert!(!css.contains("@media (max-width:"));
}

fn media_section(css: &str, width: u32, next_width: Option<u32>) -> &str {
    let start_marker = format!("@media (min-width: {width}px)");
    let start = css.find(&start_marker).expect("media query should exist");
    let end = next_width
        .and_then(|next| {
            css[start + start_marker.len()..].find(&format!("@media (min-width: {next}px)"))
        })
        .map_or(css.len(), |offset| start + start_marker.len() + offset);
    &css[start..end]
}

fn contrast(first: &str, second: &str) -> f64 {
    let first = luminance(first);
    let second = luminance(second);
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

fn luminance(hex: &str) -> f64 {
    let channel = |start| {
        let value = f64::from(u8::from_str_radix(&hex[start..start + 2], 16).unwrap()) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5)
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
fn explicit_accent_seeds_change_only_the_css_assignment() {
    let theme = Theme::default();
    let first = theme.compile_with_accent_seed(1).unwrap();
    let repeated = theme.compile_with_accent_seed(1).unwrap();
    let second = theme.compile_with_accent_seed(2).unwrap();

    assert_eq!(first, repeated);
    assert_ne!(first.css, second.css);
    assert_eq!(first.typescript, second.typescript);
    assert_eq!(first.theme, second.theme);
}

#[test]
fn preserves_single_and_non_hex_css_accents() {
    let theme = Theme::from_toml(
        "[colors.dark]\naccents=[\"red\"]\n[colors.light]\naccents=[\"oklch(50% 0.1 30)\"]\n",
        "accent.theme.toml",
    )
    .unwrap();
    let compiled = theme.compile_with_accent_seed(42).unwrap();

    assert!(compiled.css.contains("--mambo-color-accent-1: red"));
    assert!(
        compiled
            .css
            .contains("--mambo-color-accent-1: oklch(50% 0.1 30)")
    );
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
