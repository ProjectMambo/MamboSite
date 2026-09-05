pub(crate) fn shuffled_indices(count: usize, seed: u64) -> Vec<usize> {
    let mut indices = (0..count).collect::<Vec<_>>();
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    for upper in (1..count).rev() {
        let selected = usize::try_from(
            next_random(&mut state) % u64::try_from(upper + 1).expect("accent count fits u64"),
        )
        .expect("the selected slot fits usize");
        indices.swap(upper, selected);
    }
    indices
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_f491_4f6c_dd1d)
}

#[cfg(test)]
mod tests {
    use super::shuffled_indices;

    #[test]
    fn shuffle_uses_every_slot_once_and_is_seeded() {
        let first = shuffled_indices(6, 42);
        let mut sorted = first.clone();
        sorted.sort_unstable();

        assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(first, shuffled_indices(6, 42));
        assert_ne!(first, shuffled_indices(6, 43));
    }
}
