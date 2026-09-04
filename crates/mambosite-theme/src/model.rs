use serde::{Deserialize, Serialize};

pub const THEME_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    THEME_SCHEMA_VERSION
}

fn default_theme_id() -> String {
    "default".to_owned()
}

fn default_extends() -> String {
    "default".to_owned()
}

/// A complete theme after the built-in defaults have been applied.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    #[serde(default = "default_schema_version")]
    pub schema: u32,
    #[serde(default = "default_theme_id")]
    pub id: String,
    #[serde(default = "default_extends")]
    pub extends: String,
    pub default_scheme: ColorScheme,
    pub breakpoints: Breakpoints,
    pub colors: ColorSchemes,
    pub fonts: FontTokens,
    pub typography: TypographyTokens,
    pub spacing: SpacingTokens,
    pub widths: WidthTokens,
    pub dimensions: DimensionTokens,
    pub layout: LayoutTokens,
    pub borders: BorderTokens,
    pub radii: RadiusTokens,
    pub shadows: ShadowTokens,
    pub motion: MotionTokens,
    pub components: ResponsiveComponents,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            schema: THEME_SCHEMA_VERSION,
            id: default_theme_id(),
            extends: default_extends(),
            default_scheme: ColorScheme::Dark,
            breakpoints: Breakpoints::default(),
            colors: ColorSchemes::default(),
            fonts: FontTokens::default(),
            typography: TypographyTokens::default(),
            spacing: SpacingTokens::default(),
            widths: WidthTokens::default(),
            dimensions: DimensionTokens::default(),
            layout: LayoutTokens::default(),
            borders: BorderTokens::default(),
            radii: RadiusTokens::default(),
            shadows: ShadowTokens::default(),
            motion: MotionTokens::default(),
            components: ResponsiveComponents::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

impl ColorScheme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// Mobile-first viewport thresholds, expressed as CSS pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Breakpoints {
    pub compact: u32,
    pub content: u32,
    pub wide: u32,
}

impl Default for Breakpoints {
    fn default() -> Self {
        Self {
            compact: 640,
            content: 900,
            wide: 1_200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Responsive<T> {
    Value(T),
    Points(ResponsivePoints<T>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResponsivePoints<T> {
    pub base: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compact: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wide: Option<T>,
}

impl<T> From<T> for Responsive<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T> Responsive<T> {
    pub const fn base(&self) -> &T {
        match self {
            Self::Value(value) => value,
            Self::Points(points) => &points.base,
        }
    }

    pub const fn compact(&self) -> Option<&T> {
        match self {
            Self::Value(_) => None,
            Self::Points(points) => points.compact.as_ref(),
        }
    }

    pub const fn content(&self) -> Option<&T> {
        match self {
            Self::Value(_) => None,
            Self::Points(points) => points.content.as_ref(),
        }
    }

    pub const fn wide(&self) -> Option<&T> {
        match self {
            Self::Value(_) => None,
            Self::Points(points) => points.wide.as_ref(),
        }
    }
}

fn responsive<T>(
    base: T,
    compact: Option<T>,
    content: Option<T>,
    wide: Option<T>,
) -> Responsive<T> {
    Responsive::Points(ResponsivePoints {
        base,
        compact,
        content,
        wide,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorSchemes {
    pub dark: ColorPalette,
    pub light: ColorPalette,
}

impl Default for ColorSchemes {
    fn default() -> Self {
        Self {
            dark: ColorPalette::dark(),
            light: ColorPalette::light(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColorPalette {
    pub background: String,
    pub surface: String,
    pub surface_strong: String,
    pub border: String,
    pub text: String,
    pub text_muted: String,
    pub text_subtle: String,
    pub brand: String,
    pub brand_hover: String,
    pub on_brand: String,
    pub selection: String,
    pub focus: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
    pub header_background: String,
    pub shadow: String,
    pub accents: Vec<String>,
}

impl ColorPalette {
    fn dark() -> Self {
        Self {
            background: "#181615".to_owned(),
            surface: "#24201e".to_owned(),
            surface_strong: "#312b28".to_owned(),
            border: "#4a403b".to_owned(),
            text: "#faf7f2".to_owned(),
            text_muted: "#d5c9b8".to_owned(),
            text_subtle: "#969085".to_owned(),
            brand: "#d44b36".to_owned(),
            brand_hover: "#ef674f".to_owned(),
            on_brand: "#ffffff".to_owned(),
            selection: "#d44b36".to_owned(),
            focus: "#ffa775".to_owned(),
            success: "#9cbaac".to_owned(),
            warning: "#ffa775".to_owned(),
            danger: "#bf8087".to_owned(),
            header_background: "rgb(24 22 21 / 92%)".to_owned(),
            shadow: "rgb(0 0 0 / 28%)".to_owned(),
            accents: [
                "#9cbaac", "#ffa775", "#c2cca8", "#8b9cbd", "#e08a4f", "#bf8087",
            ]
            .map(str::to_owned)
            .to_vec(),
        }
    }

    fn light() -> Self {
        Self {
            background: "#faf7f2".to_owned(),
            surface: "#f3ece2".to_owned(),
            surface_strong: "#eadfce".to_owned(),
            border: "#c9bba7".to_owned(),
            text: "#1c1111".to_owned(),
            text_muted: "#5c4d4a".to_owned(),
            text_subtle: "#765d3f".to_owned(),
            brand: "#a93622".to_owned(),
            brand_hover: "#822727".to_owned(),
            on_brand: "#ffffff".to_owned(),
            selection: "#a93622".to_owned(),
            focus: "#de8554".to_owned(),
            success: "#6b8c85".to_owned(),
            warning: "#de8554".to_owned(),
            danger: "#8c5258".to_owned(),
            header_background: "rgb(250 247 242 / 92%)".to_owned(),
            shadow: "rgb(49 35 31 / 14%)".to_owned(),
            accents: [
                "#6b8c85", "#de8554", "#8fa382", "#4e687d", "#b86935", "#8c5258",
            ]
            .map(str::to_owned)
            .to_vec(),
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FontTokens {
    pub body: String,
    pub heading: String,
    pub mono: String,
    pub faces: Vec<FontFace>,
}

impl Default for FontTokens {
    fn default() -> Self {
        let family = "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
        Self {
            body: family.to_owned(),
            heading: family.to_owned(),
            mono: family.to_owned(),
            faces: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FontFace {
    pub family: String,
    pub source: String,
    #[serde(default = "default_font_weight")]
    pub weight: u16,
    #[serde(default)]
    pub style: FontStyle,
    #[serde(default)]
    pub format: FontFormat,
    #[serde(default)]
    pub display: FontDisplay,
}

const fn default_font_weight() -> u16 {
    400
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl FontStyle {
    pub const fn as_css(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Italic => "italic",
            Self::Oblique => "oblique",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontFormat {
    #[default]
    Woff2,
    Woff,
    Truetype,
    Opentype,
}

impl FontFormat {
    pub const fn as_css(self) -> &'static str {
        match self {
            Self::Woff2 => "woff2",
            Self::Woff => "woff",
            Self::Truetype => "truetype",
            Self::Opentype => "opentype",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FontDisplay {
    Auto,
    Block,
    #[default]
    Swap,
    Fallback,
    Optional,
}

impl FontDisplay {
    pub const fn as_css(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Block => "block",
            Self::Swap => "swap",
            Self::Fallback => "fallback",
            Self::Optional => "optional",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TypographyTokens {
    pub body: TextStyle,
    pub small: TextStyle,
    pub label: TextStyle,
    pub navigation: TextStyle,
    pub heading_1: TextStyle,
    pub heading_2: TextStyle,
    pub heading_3: TextStyle,
    pub hero_lead: TextStyle,
    pub card_title: TextStyle,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            body: TextStyle::new(Responsive::Value("1rem".to_owned()), "1.78", 400, "normal"),
            small: TextStyle::new(Responsive::Value("0.8rem".to_owned()), "1.5", 400, "normal"),
            label: TextStyle::new(
                Responsive::Value("0.75rem".to_owned()),
                "1.35",
                700,
                "0.08em",
            ),
            navigation: TextStyle::new(
                responsive("0.72rem".to_owned(), Some("0.85rem".to_owned()), None, None),
                "1.2",
                700,
                "0.08em",
            ),
            heading_1: TextStyle::new(
                Responsive::Value("clamp(2.5rem, 7vw, 5.5rem)".to_owned()),
                "1.1",
                700,
                "-0.055em",
            ),
            heading_2: TextStyle::new(
                Responsive::Value("clamp(1.75rem, 4vw, 2.7rem)".to_owned()),
                "1.1",
                700,
                "-0.035em",
            ),
            heading_3: TextStyle::new(
                Responsive::Value("1.45rem".to_owned()),
                "1.2",
                700,
                "normal",
            ),
            hero_lead: TextStyle::new(
                Responsive::Value("1.15rem".to_owned()),
                "1.6",
                400,
                "normal",
            ),
            card_title: TextStyle::new(
                Responsive::Value("1.25rem".to_owned()),
                "1.3",
                700,
                "normal",
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TextStyle {
    pub size: Responsive<String>,
    pub line_height: String,
    pub weight: u16,
    pub letter_spacing: String,
}

impl TextStyle {
    fn new(size: Responsive<String>, line_height: &str, weight: u16, letter_spacing: &str) -> Self {
        Self {
            size,
            line_height: line_height.to_owned(),
            weight,
            letter_spacing: letter_spacing.to_owned(),
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new(
            Responsive::Value("1rem".to_owned()),
            "normal",
            400,
            "normal",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpacingTokens {
    pub xxs: String,
    pub xs: String,
    pub sm: String,
    pub md: String,
    pub lg: String,
    pub xl: String,
    pub xxl: String,
    pub section: String,
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xxs: "0.25rem".to_owned(),
            xs: "0.5rem".to_owned(),
            sm: "0.75rem".to_owned(),
            md: "1rem".to_owned(),
            lg: "1.5rem".to_owned(),
            xl: "2.5rem".to_owned(),
            xxl: "4rem".to_owned(),
            section: "4rem".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WidthTokens {
    pub reading: String,
    pub normal: String,
    pub wide: String,
    pub full: String,
    pub shell: String,
    pub sidebar: String,
    pub card_min: String,
    pub hero_image_min: String,
}

impl Default for WidthTokens {
    fn default() -> Self {
        Self {
            reading: "48rem".to_owned(),
            normal: "74rem".to_owned(),
            wide: "88rem".to_owned(),
            full: "none".to_owned(),
            shell: "90rem".to_owned(),
            sidebar: "15rem".to_owned(),
            card_min: "14rem".to_owned(),
            hero_image_min: "14rem".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct DimensionTokens {
    pub main_top_padding: Responsive<String>,
    pub header_height: Responsive<String>,
    pub header_blur: String,
    pub toc_offset: Responsive<String>,
    pub hero_min_height: String,
    pub control_min_height: String,
}

impl Default for DimensionTokens {
    fn default() -> Self {
        Self {
            main_top_padding: responsive("6rem".to_owned(), Some("7.5rem".to_owned()), None, None),
            header_height: responsive("4rem".to_owned(), Some("4.5rem".to_owned()), None, None),
            header_blur: "16px".to_owned(),
            toc_offset: Responsive::Value("6.5rem".to_owned()),
            hero_min_height: "18rem".to_owned(),
            control_min_height: "5rem".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LayoutTokens {
    pub page_with_sidebar_columns: Responsive<String>,
    pub hero_split_columns: Responsive<String>,
    pub list_card_columns: Responsive<String>,
    pub metadata_columns: Responsive<String>,
    pub footer_direction: Responsive<FlexDirection>,
    pub header_inline_padding: Responsive<String>,
    pub page_inline_padding: Responsive<String>,
    pub footer_inline_padding: Responsive<String>,
}

impl Default for LayoutTokens {
    fn default() -> Self {
        Self {
            page_with_sidebar_columns: responsive(
                "minmax(0, 1fr)".to_owned(),
                None,
                Some("minmax(0, 1fr) var(--mambo-width-sidebar)".to_owned()),
                None,
            ),
            hero_split_columns: responsive(
                "minmax(0, 1fr)".to_owned(),
                None,
                Some(
                    "minmax(var(--mambo-width-hero-image-min), 0.75fr) minmax(0, 1.25fr)"
                        .to_owned(),
                ),
                None,
            ),
            list_card_columns: responsive(
                "minmax(0, 1fr)".to_owned(),
                Some("minmax(8rem, 0.3fr) minmax(0, 1fr)".to_owned()),
                None,
                None,
            ),
            metadata_columns: responsive(
                "minmax(0, 1fr)".to_owned(),
                Some("minmax(7rem, 0.25fr) minmax(0, 1fr)".to_owned()),
                None,
                None,
            ),
            footer_direction: responsive(
                FlexDirection::Column,
                Some(FlexDirection::Row),
                None,
                None,
            ),
            header_inline_padding: responsive(
                "1rem".to_owned(),
                Some("1.5rem".to_owned()),
                None,
                None,
            ),
            page_inline_padding: responsive(
                "1rem".to_owned(),
                Some("1.5rem".to_owned()),
                None,
                None,
            ),
            footer_inline_padding: responsive(
                "1rem".to_owned(),
                Some("1.5rem".to_owned()),
                None,
                None,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl FlexDirection {
    pub const fn as_css(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "column",
            Self::RowReverse => "row-reverse",
            Self::ColumnReverse => "column-reverse",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BorderTokens {
    pub thin: String,
    pub strong: String,
    pub accent: String,
    pub style: BorderStyle,
}

impl Default for BorderTokens {
    fn default() -> Self {
        Self {
            thin: "1px".to_owned(),
            strong: "2px".to_owned(),
            accent: "0.4rem".to_owned(),
            style: BorderStyle::Solid,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Double,
}

impl BorderStyle {
    pub const fn as_css(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
            Self::Double => "double",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RadiusTokens {
    pub small: String,
    pub medium: String,
    pub large: String,
    pub pill: String,
}

impl Default for RadiusTokens {
    fn default() -> Self {
        Self {
            small: "0".to_owned(),
            medium: "0".to_owned(),
            large: "0".to_owned(),
            pill: "9999px".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShadowTokens {
    pub small: String,
    pub medium: String,
    pub large: String,
}

impl Default for ShadowTokens {
    fn default() -> Self {
        Self {
            small: "0 1px 2px var(--mambo-color-shadow)".to_owned(),
            medium: "0 0.5rem 1.5rem var(--mambo-color-shadow)".to_owned(),
            large: "0 1rem 3rem var(--mambo-color-shadow)".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MotionTokens {
    pub fast: String,
    pub normal: String,
    pub slow: String,
    pub easing: String,
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            fast: "160ms".to_owned(),
            normal: "180ms".to_owned(),
            slow: "300ms".to_owned(),
            easing: "ease".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResponsiveComponents {
    pub collection: CollectionBehavior,
    pub sidebar: SidebarBehavior,
    pub header: HeaderBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CollectionBehavior {
    pub max_columns: Responsive<u8>,
}

impl Default for CollectionBehavior {
    fn default() -> Self {
        Self {
            max_columns: responsive(1, Some(2), Some(2), Some(6)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SidebarBehavior {
    pub mode: Responsive<SidebarMode>,
}

impl Default for SidebarBehavior {
    fn default() -> Self {
        Self {
            mode: responsive(SidebarMode::Inline, None, Some(SidebarMode::Sticky), None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SidebarMode {
    Hidden,
    Inline,
    Sticky,
}

impl SidebarMode {
    pub const fn display(self) -> &'static str {
        match self {
            Self::Hidden => "none",
            Self::Inline | Self::Sticky => "block",
        }
    }

    pub const fn position(self) -> &'static str {
        match self {
            Self::Sticky => "sticky",
            Self::Hidden | Self::Inline => "static",
        }
    }

    pub const fn order(self) -> i8 {
        match self {
            Self::Inline => -1,
            Self::Hidden | Self::Sticky => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct HeaderBehavior {
    pub mode: Responsive<HeaderMode>,
    pub clock: Responsive<Visibility>,
    pub hide_on_scroll: bool,
    pub hide_after: u32,
}

impl Default for HeaderBehavior {
    fn default() -> Self {
        Self {
            mode: responsive(HeaderMode::Compact, None, Some(HeaderMode::Full), None),
            clock: responsive(Visibility::Hidden, None, Some(Visibility::Visible), None),
            hide_on_scroll: true,
            hide_after: 80,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeaderMode {
    Compact,
    Full,
}

impl HeaderMode {
    pub const fn display(self) -> &'static str {
        match self {
            Self::Compact => "flex",
            Self::Full => "grid",
        }
    }

    pub const fn columns(self) -> &'static str {
        match self {
            Self::Compact => "auto 1fr",
            Self::Full => "auto 1fr auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Hidden,
    Visible,
}

impl Visibility {
    pub const fn flex_display(self) -> &'static str {
        match self {
            Self::Hidden => "none",
            Self::Visible => "flex",
        }
    }
}
