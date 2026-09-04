use crate::model::ColorPalette;

pub(crate) const DIFFERENCE_THRESHOLD: f64 = 0.10;

pub(crate) fn safe_cycle(
    dark: &ColorPalette,
    light: &ColorPalette,
    seed: u64,
) -> Option<Vec<usize>> {
    if dark.accents.len() != light.accents.len() || dark.accents.len() < 2 {
        return None;
    }
    let dark = dark
        .accents
        .iter()
        .map(|colour| oklab(colour))
        .collect::<Option<Vec<_>>>()?;
    let light = light
        .accents
        .iter()
        .map(|colour| oklab(colour))
        .collect::<Option<Vec<_>>>()?;
    let count = dark.len();
    let compatible = (0..count)
        .map(|first| {
            (0..count)
                .map(|second| {
                    first != second
                        && difference(dark[first], dark[second]) >= DIFFERENCE_THRESHOLD
                        && difference(light[first], light[second]) >= DIFFERENCE_THRESHOLD
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let candidates = shuffled_indices(count, seed);

    for &start in &candidates {
        let mut path = vec![start];
        let mut failed = vec![false; count * (1_usize << count)];
        if extend_cycle(
            start,
            &candidates,
            &compatible,
            &mut path,
            1_u16 << start,
            &mut failed,
        ) {
            return Some(path);
        }
    }
    None
}

pub(crate) fn slot_at(cycle: &[usize], columns: usize, index: usize) -> usize {
    cycle[(index / columns + index % columns) % cycle.len()]
}

fn extend_cycle(
    start: usize,
    candidates: &[usize],
    compatible: &[Vec<bool>],
    path: &mut Vec<usize>,
    used: u16,
    failed: &mut [bool],
) -> bool {
    if path.len() == candidates.len() {
        return compatible[*path.last().expect("a cycle has a start")][start];
    }
    let previous = *path.last().expect("a cycle has a start");
    let state = usize::from(used) * candidates.len() + previous;
    if failed[state] {
        return false;
    }
    for &candidate in candidates {
        let candidate_bit = 1_u16 << candidate;
        if used & candidate_bit != 0 || !compatible[previous][candidate] {
            continue;
        }
        path.push(candidate);
        if extend_cycle(
            start,
            candidates,
            compatible,
            path,
            used | candidate_bit,
            failed,
        ) {
            return true;
        }
        path.pop();
    }
    failed[state] = true;
    false
}

pub(crate) fn shuffled_indices(count: usize, seed: u64) -> Vec<usize> {
    let mut indices = (0..count).collect::<Vec<_>>();
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    for upper in (1..count).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let random = state.wrapping_mul(0x2545_f491_4f6c_dd1d);
        let choices = u64::try_from(upper + 1).expect("accent palettes contain at most 12 slots");
        let selected = usize::try_from(random % choices).expect("the selected slot fits usize");
        indices.swap(upper, selected);
    }
    indices
}

fn difference(first: [f64; 3], second: [f64; 3]) -> f64 {
    first
        .into_iter()
        .zip(second)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

fn oklab(value: &str) -> Option<[f64; 3]> {
    let hex = value.strip_prefix('#')?;
    if !hex.is_ascii() {
        return None;
    }
    let [red, green, blue] = match hex.len() {
        3 => {
            let mut channels = hex.chars().map(|character| {
                character
                    .to_digit(16)
                    .map(|channel| f64::from(channel * 17) / 255.0)
            });
            [channels.next()??, channels.next()??, channels.next()??]
        }
        6 => {
            let channel = |start| {
                u8::from_str_radix(&hex[start..start + 2], 16)
                    .ok()
                    .map(|channel| f64::from(channel) / 255.0)
            };
            [channel(0)?, channel(2)?, channel(4)?]
        }
        _ => return None,
    };
    let linear = |channel: f64| {
        if channel <= 0.040_45 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    let red = linear(red);
    let green = linear(green);
    let blue = linear(blue);
    let long = (0.412_221_470_8 * red + 0.536_332_536_3 * green + 0.051_445_992_9 * blue).cbrt();
    let medium = (0.211_903_498_2 * red + 0.680_699_545_1 * green + 0.107_396_956_6 * blue).cbrt();
    let short = (0.088_302_461_9 * red + 0.281_718_837_6 * green + 0.629_978_700_5 * blue).cbrt();
    Some([
        0.210_454_255_3 * long + 0.793_617_785 * medium - 0.004_072_046_8 * short,
        1.977_998_495_1 * long - 2.428_592_205 * medium + 0.450_593_709_9 * short,
        0.025_904_037_1 * long + 0.782_771_766_2 * medium - 0.808_675_766 * short,
    ])
}

#[cfg(test)]
mod tests {
    use super::{DIFFERENCE_THRESHOLD, difference, oklab, safe_cycle, slot_at};
    use crate::model::Theme;

    #[test]
    fn default_palette_is_safe_at_every_grid_width() {
        let theme = Theme::default();
        let cycle = safe_cycle(&theme.colors.dark, &theme.colors.light, 42).unwrap();

        assert_eq!(cycle.len(), theme.colors.dark.accents.len());
        for columns in 1..=6 {
            for index in 0..100 {
                let slot = slot_at(&cycle, columns, index);
                if index % columns > 0 {
                    assert_safe(&theme, slot, slot_at(&cycle, columns, index - 1));
                }
                if index >= columns {
                    assert_safe(&theme, slot, slot_at(&cycle, columns, index - columns));
                }
            }
        }
    }

    #[test]
    fn seed_is_stable_and_can_change_the_cycle() {
        let theme = Theme::default();
        let first = safe_cycle(&theme.colors.dark, &theme.colors.light, 1).unwrap();
        assert_eq!(
            first,
            safe_cycle(&theme.colors.dark, &theme.colors.light, 1).unwrap()
        );
        assert_ne!(
            first,
            safe_cycle(&theme.colors.dark, &theme.colors.light, 2).unwrap()
        );
    }

    fn assert_safe(theme: &Theme, first: usize, second: usize) {
        for palette in [&theme.colors.dark, &theme.colors.light] {
            let first = oklab(&palette.accents[first]).unwrap();
            let second = oklab(&palette.accents[second]).unwrap();
            assert!(difference(first, second) >= DIFFERENCE_THRESHOLD);
        }
    }
}
