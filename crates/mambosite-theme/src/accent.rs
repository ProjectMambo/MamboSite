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

pub(crate) fn grid_slots(cycle: &[usize], columns: usize, seed: u64) -> Vec<usize> {
    if cycle.len() == 1 {
        return cycle.to_vec();
    }

    let count = cycle.len();
    let mut state = seed
        ^ u64::try_from(columns)
            .expect("grid columns fit u64")
            .wrapping_mul(0xd1b5_4a32_d192_ed03);
    let mut rows = Vec::with_capacity(count + 1);
    let mut first = Vec::with_capacity(columns);
    first.push(random_below(&mut state, count));
    for column in 1..columns {
        first.push(random_neighbour(first[column - 1], count, &mut state));
    }
    rows.push(first);

    let vertical_step = if next_random(&mut state) & 1 == 0 {
        1
    } else {
        count - 1
    };
    for _ in 0..count {
        let above = rows.last().expect("the grid has a first row");
        let mut row = Vec::with_capacity(columns);
        row.push((above[0] + vertical_step) % count);
        for column in 1..columns {
            let left = row[column - 1];
            let above = above[column];
            let candidates = neighbours(left, count)
                .into_iter()
                .filter(|candidate| are_neighbours(*candidate, above, count))
                .collect::<Vec<_>>();
            row.push(candidates[random_below(&mut state, candidates.len())]);
        }
        rows.push(row);
    }

    // Walk the generated rows back to the first one so the nth-child pattern
    // also remains safe where its period repeats.
    let mut slots = rows
        .iter()
        .flatten()
        .map(|position| cycle[*position])
        .collect::<Vec<_>>();
    for row in rows[1..count].iter().rev() {
        slots.extend(row.iter().map(|position| cycle[*position]));
    }
    slots
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
        let selected = random_below(&mut state, upper + 1);
        indices.swap(upper, selected);
    }
    indices
}

fn random_neighbour(position: usize, count: usize, state: &mut u64) -> usize {
    let neighbours = neighbours(position, count);
    neighbours[random_below(state, neighbours.len())]
}

fn neighbours(position: usize, count: usize) -> Vec<usize> {
    let previous = (position + count - 1) % count;
    let next = (position + 1) % count;
    if previous == next {
        vec![previous]
    } else {
        vec![previous, next]
    }
}

fn are_neighbours(first: usize, second: usize, count: usize) -> bool {
    (first + 1) % count == second || (second + 1) % count == first
}

fn random_below(state: &mut u64, upper: usize) -> usize {
    usize::try_from(next_random(state) % u64::try_from(upper).expect("accent count fits u64"))
        .expect("the selected slot fits usize")
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_f491_4f6c_dd1d)
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
    use super::{DIFFERENCE_THRESHOLD, difference, grid_slots, oklab, safe_cycle};
    use crate::model::Theme;

    #[test]
    fn default_palette_is_safe_at_every_grid_width() {
        let theme = Theme::default();
        let cycle = safe_cycle(&theme.colors.dark, &theme.colors.light, 42).unwrap();

        assert_eq!(cycle.len(), theme.colors.dark.accents.len());
        for columns in 1..=6 {
            let slots = grid_slots(&cycle, columns, 73);
            assert_eq!(slots.len(), cycle.len() * columns * 2);
            for index in 0..slots.len() {
                let slot = slots[index];
                if index % columns > 0 {
                    assert_safe(&theme, slot, slots[index - 1]);
                }
                if index >= columns {
                    assert_safe(&theme, slot, slots[index - columns]);
                }
            }
            for column in 0..columns {
                assert_safe(&theme, slots[column], slots[slots.len() - columns + column]);
            }
            assert!(cycle.iter().all(|slot| slots.contains(slot)));
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

    #[test]
    fn grid_seed_is_stable_without_repeating_the_diagonal_cycle() {
        let theme = Theme::default();
        let cycle = safe_cycle(&theme.colors.dark, &theme.colors.light, 42).unwrap();
        let first = grid_slots(&cycle, 4, 42);

        assert_eq!(first, grid_slots(&cycle, 4, 42));
        assert_ne!(first, grid_slots(&cycle, 4, 43));
        assert_ne!(
            first,
            (0..first.len())
                .map(|index| cycle[(index / 4 + index % 4) % cycle.len()])
                .collect::<Vec<_>>()
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
