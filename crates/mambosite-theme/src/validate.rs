use crate::error::ThemeDiagnostic;
use crate::model::{Responsive, THEME_SCHEMA_VERSION, TextStyle, Theme};

#[allow(clippy::too_many_lines)]
pub(crate) fn validate(theme: &Theme) -> Vec<ThemeDiagnostic> {
    let mut diagnostics = Vec::new();

    if theme.schema != THEME_SCHEMA_VERSION {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1001",
            "schema",
            format!(
                "unsupported theme schema {}; expected {THEME_SCHEMA_VERSION}",
                theme.schema
            ),
        ));
    }
    if theme.extends != "default" {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1002",
            "extends",
            "only the built-in `default` theme can currently be extended",
        ));
    }
    if theme.id.is_empty()
        || !theme.id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1003",
            "id",
            "theme ids must contain only lowercase ASCII letters, digits, and hyphens",
        ));
    }

    let breakpoints = theme.breakpoints;
    if breakpoints.compact == 0
        || breakpoints.compact >= breakpoints.content
        || breakpoints.content >= breakpoints.wide
    {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1004",
            "breakpoints",
            "breakpoints must be positive and strictly ordered: compact < content < wide",
        ));
    }

    validate_palette("colors.dark", &theme.colors.dark, &mut diagnostics);
    validate_palette("colors.light", &theme.colors.light, &mut diagnostics);
    if theme.colors.dark.accents.len() != theme.colors.light.accents.len() {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1106",
            "colors",
            "dark and light schemes must define the same number of accent slots",
        ));
    }

    for (index, face) in theme.fonts.faces.iter().enumerate() {
        validate_css(
            &format!("fonts.faces[{index}].family"),
            &face.family,
            &mut diagnostics,
        );
        validate_css(
            &format!("fonts.faces[{index}].source"),
            &face.source,
            &mut diagnostics,
        );
        validate_weight(
            &format!("fonts.faces[{index}].weight"),
            face.weight,
            &mut diagnostics,
        );
    }
    for (field, value) in [
        ("fonts.body", &theme.fonts.body),
        ("fonts.heading", &theme.fonts.heading),
        ("fonts.mono", &theme.fonts.mono),
    ] {
        validate_css(field, value, &mut diagnostics);
    }

    for (name, style) in typography(theme) {
        validate_text_style(name, style, &mut diagnostics);
    }

    for (field, value) in [
        ("spacing.xxs", &theme.spacing.xxs),
        ("spacing.xs", &theme.spacing.xs),
        ("spacing.sm", &theme.spacing.sm),
        ("spacing.md", &theme.spacing.md),
        ("spacing.lg", &theme.spacing.lg),
        ("spacing.xl", &theme.spacing.xl),
        ("spacing.xxl", &theme.spacing.xxl),
        ("spacing.section", &theme.spacing.section),
        ("widths.reading", &theme.widths.reading),
        ("widths.normal", &theme.widths.normal),
        ("widths.wide", &theme.widths.wide),
        ("widths.full", &theme.widths.full),
        ("widths.shell", &theme.widths.shell),
        ("widths.sidebar", &theme.widths.sidebar),
        ("widths.card_min", &theme.widths.card_min),
        ("widths.hero_image_min", &theme.widths.hero_image_min),
        ("dimensions.header_blur", &theme.dimensions.header_blur),
        (
            "dimensions.hero_min_height",
            &theme.dimensions.hero_min_height,
        ),
        (
            "dimensions.control_min_height",
            &theme.dimensions.control_min_height,
        ),
        ("borders.thin", &theme.borders.thin),
        ("borders.strong", &theme.borders.strong),
        ("borders.accent", &theme.borders.accent),
        ("radii.small", &theme.radii.small),
        ("radii.medium", &theme.radii.medium),
        ("radii.large", &theme.radii.large),
        ("radii.pill", &theme.radii.pill),
        ("shadows.small", &theme.shadows.small),
        ("shadows.medium", &theme.shadows.medium),
        ("shadows.large", &theme.shadows.large),
        ("motion.fast", &theme.motion.fast),
        ("motion.normal", &theme.motion.normal),
        ("motion.slow", &theme.motion.slow),
        ("motion.easing", &theme.motion.easing),
    ] {
        validate_css(field, value, &mut diagnostics);
    }
    for (field, value) in [
        (
            "dimensions.main_top_padding",
            &theme.dimensions.main_top_padding,
        ),
        ("dimensions.header_height", &theme.dimensions.header_height),
        ("dimensions.toc_offset", &theme.dimensions.toc_offset),
    ] {
        validate_responsive_css(field, value, &mut diagnostics);
    }

    validate_columns(
        "components.collection.max_columns",
        &theme.components.collection.max_columns,
        &mut diagnostics,
    );

    diagnostics
}

fn validate_palette(
    prefix: &str,
    palette: &crate::model::ColorPalette,
    diagnostics: &mut Vec<ThemeDiagnostic>,
) {
    for (field, value) in [
        ("background", &palette.background),
        ("surface", &palette.surface),
        ("surface_strong", &palette.surface_strong),
        ("border", &palette.border),
        ("text", &palette.text),
        ("text_muted", &palette.text_muted),
        ("text_subtle", &palette.text_subtle),
        ("brand", &palette.brand),
        ("brand_hover", &palette.brand_hover),
        ("on_brand", &palette.on_brand),
        ("selection", &palette.selection),
        ("focus", &palette.focus),
        ("success", &palette.success),
        ("warning", &palette.warning),
        ("danger", &palette.danger),
        ("header_background", &palette.header_background),
        ("shadow", &palette.shadow),
    ] {
        validate_css(&format!("{prefix}.{field}"), value, diagnostics);
    }
    if palette.accents.is_empty() || palette.accents.len() > 12 {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1101",
            format!("{prefix}.accents"),
            "accent palettes must contain between 1 and 12 colors",
        ));
    }
    for (index, value) in palette.accents.iter().enumerate() {
        validate_css(&format!("{prefix}.accents[{index}]"), value, diagnostics);
    }
}

fn typography(theme: &Theme) -> [(&'static str, &TextStyle); 9] {
    [
        ("typography.body", &theme.typography.body),
        ("typography.small", &theme.typography.small),
        ("typography.label", &theme.typography.label),
        ("typography.navigation", &theme.typography.navigation),
        ("typography.heading_1", &theme.typography.heading_1),
        ("typography.heading_2", &theme.typography.heading_2),
        ("typography.heading_3", &theme.typography.heading_3),
        ("typography.hero_lead", &theme.typography.hero_lead),
        ("typography.card_title", &theme.typography.card_title),
    ]
}

fn validate_text_style(prefix: &str, style: &TextStyle, diagnostics: &mut Vec<ThemeDiagnostic>) {
    validate_responsive_css(&format!("{prefix}.size"), &style.size, diagnostics);
    validate_css(
        &format!("{prefix}.line_height"),
        &style.line_height,
        diagnostics,
    );
    validate_weight(&format!("{prefix}.weight"), style.weight, diagnostics);
    validate_css(
        &format!("{prefix}.letter_spacing"),
        &style.letter_spacing,
        diagnostics,
    );
}

fn validate_weight(field: &str, weight: u16, diagnostics: &mut Vec<ThemeDiagnostic>) {
    if !(1..=1_000).contains(&weight) {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1102",
            field,
            "font weights must be between 1 and 1000",
        ));
    }
}

fn validate_responsive_css(
    field: &str,
    value: &Responsive<String>,
    diagnostics: &mut Vec<ThemeDiagnostic>,
) {
    validate_css(&format!("{field}.base"), value.base(), diagnostics);
    for (point, override_value) in [
        ("compact", value.compact()),
        ("content", value.content()),
        ("wide", value.wide()),
    ] {
        if let Some(override_value) = override_value {
            validate_css(&format!("{field}.{point}"), override_value, diagnostics);
        }
    }
}

fn validate_columns(field: &str, value: &Responsive<u8>, diagnostics: &mut Vec<ThemeDiagnostic>) {
    for (point, columns) in [
        ("base", Some(value.base())),
        ("compact", value.compact()),
        ("content", value.content()),
        ("wide", value.wide()),
    ] {
        if columns.is_some_and(|columns| !(1..=6).contains(columns)) {
            diagnostics.push(ThemeDiagnostic::new(
                "MST1103",
                format!("{field}.{point}"),
                "collection column limits must be between 1 and 6",
            ));
        }
    }
}

fn validate_css(field: &str, value: &str, diagnostics: &mut Vec<ThemeDiagnostic>) {
    if value.trim().is_empty() {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1104",
            field,
            "CSS token values cannot be empty",
        ));
    } else if value.contains("/*")
        || value.contains("*/")
        || value
            .chars()
            .any(|character| matches!(character, ';' | '{' | '}' | '\\' | '\n' | '\r' | '\0'))
    {
        diagnostics.push(ThemeDiagnostic::new(
            "MST1105",
            field,
            "CSS token values cannot contain declarations, blocks, or line breaks",
        ));
    }
}
