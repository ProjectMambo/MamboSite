mod css;
mod error;
mod model;
mod resolve;
mod typescript;
mod validate;

use std::fs;
use std::path::{Path, PathBuf};

pub use error::{ThemeDiagnostic, ThemeError};
pub use model::{
    BorderStyle, BorderTokens, Breakpoints, CollectionBehavior, ColorPalette, ColorScheme,
    ColorSchemes, DimensionTokens, FontDisplay, FontFace, FontFormat, FontStyle, FontTokens,
    HeaderBehavior, HeaderMode, MotionTokens, RadiusTokens, Responsive, ResponsiveComponents,
    ResponsivePoints, ShadowTokens, SidebarBehavior, SidebarMode, SpacingTokens,
    THEME_SCHEMA_VERSION, TextStyle, Theme, TypographyTokens, Visibility, WidthTokens,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTheme {
    pub theme: Theme,
    pub css: String,
    pub typescript: String,
}

/// Returns the complete, human-editable default `mambo.theme.toml`.
///
/// `mambosite init` can write this value directly, keeping the scaffold in
/// lockstep with the schema and Rust defaults.
///
/// # Errors
///
/// Returns a TOML serialization error if a future schema adds a value that
/// TOML cannot represent.
pub fn default_theme_toml() -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(&Theme::default())
}

impl Theme {
    /// Loads, defaults, and validates a `mambo.theme.toml` file.
    ///
    /// # Errors
    ///
    /// Returns a read, TOML parse, or structured validation error.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| ThemeError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml(&source, path)
    }

    /// Parses a theme with built-in defaults and validates the resolved value.
    ///
    /// # Errors
    ///
    /// Returns a TOML parse or structured validation error labelled with `path`.
    pub fn from_toml(source: &str, path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let path = path.as_ref();
        let theme: Self = resolve::from_toml(source).map_err(|source| ThemeError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        let diagnostics = theme.validate();
        if diagnostics.is_empty() {
            Ok(theme)
        } else {
            Err(ThemeError::Validation {
                path: path.to_path_buf(),
                diagnostics,
            })
        }
    }

    pub fn validate(&self) -> Vec<ThemeDiagnostic> {
        validate::validate(self)
    }

    /// Generates deterministic CSS and TypeScript metadata from a valid theme.
    ///
    /// # Errors
    ///
    /// Returns all validation diagnostics if the public theme value was changed
    /// after parsing.
    pub fn compile(&self) -> Result<CompiledTheme, Vec<ThemeDiagnostic>> {
        let diagnostics = self.validate();
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        Ok(CompiledTheme {
            theme: self.clone(),
            css: css::render(self),
            typescript: typescript::render(self),
        })
    }
}

/// Loads and compiles a `mambo.theme.toml` file without writing outputs.
///
/// The CLI owns output policy. The intended destinations are
/// `{assets_out}/theme.css` and `{typescript_out}/theme.ts`.
///
/// # Errors
///
/// Returns a read, TOML parse, or structured validation error.
pub fn compile_theme_file(path: impl AsRef<Path>) -> Result<CompiledTheme, ThemeError> {
    let path: PathBuf = path.as_ref().to_path_buf();
    let theme = Theme::load(&path)?;
    theme
        .compile()
        .map_err(|diagnostics| ThemeError::Validation { path, diagnostics })
}
