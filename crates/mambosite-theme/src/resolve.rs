use crate::model::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AccentOverrides {
    dark: bool,
    light: bool,
}

impl AccentOverrides {
    pub(crate) const fn provider_managed(self) -> bool {
        !self.dark && !self.light
    }

    pub(crate) const fn one_sided(self) -> bool {
        self.dark != self.light
    }
}

pub(crate) fn from_toml(source: &str) -> Result<(Theme, AccentOverrides), toml::de::Error> {
    let overrides: toml::Value = toml::from_str(source)?;
    let accent_overrides = AccentOverrides {
        dark: has_accent_override(&overrides, "dark"),
        light: has_accent_override(&overrides, "light"),
    };
    let mut resolved = toml::Value::try_from(Theme::default())
        .expect("the built-in theme must always serialize as TOML");
    merge(&mut resolved, overrides);
    Ok((resolved.try_into()?, accent_overrides))
}

fn has_accent_override(value: &toml::Value, scheme: &str) -> bool {
    value
        .get("colors")
        .and_then(|colors| colors.get(scheme))
        .and_then(toml::Value::as_table)
        .is_some_and(|palette| palette.contains_key("accents"))
}

fn merge(base: &mut toml::Value, overrides: toml::Value) {
    match (base, overrides) {
        (toml::Value::Table(base), toml::Value::Table(overrides)) => {
            for (key, value) in overrides {
                if let Some(base_value) = base.get_mut(&key) {
                    merge(base_value, value);
                } else {
                    base.insert(key, value);
                }
            }
        }
        (base, value) => *base = value,
    }
}

#[cfg(test)]
mod tests {
    use super::from_toml;

    #[test]
    fn merges_nested_overrides_into_their_specific_defaults() {
        let (theme, _) = from_toml(
            r##"
                [colors.light]
                background = "#ffffff"

                [typography.heading_1]
                weight = 800
            "##,
        )
        .unwrap();

        assert_eq!(theme.colors.light.background, "#ffffff");
        assert_eq!(theme.colors.light.surface, "#f3ece2");
        assert_eq!(theme.typography.heading_1.weight, 800);
        assert_eq!(theme.typography.heading_1.line_height, "1.1");
    }

    #[test]
    fn preserves_accent_key_presence() {
        let (_, accents) = from_toml("[colors.dark]\naccents = []").unwrap();

        assert!(!accents.provider_managed());
        assert!(accents.one_sided());
    }
}
