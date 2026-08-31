use crate::model::Theme;

pub(crate) fn from_toml(source: &str) -> Result<Theme, toml::de::Error> {
    let overrides: toml::Value = toml::from_str(source)?;
    let mut resolved = toml::Value::try_from(Theme::default())
        .expect("the built-in theme must always serialize as TOML");
    merge(&mut resolved, overrides);
    resolved.try_into()
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
        let theme = from_toml(
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
}
