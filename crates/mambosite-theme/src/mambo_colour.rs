use mambocolour::{Colour, Scheme, theme};

use crate::model::{ColorPalette, ColorSchemes};

const ACCENT_COUNT: usize = 6;

pub(crate) fn dark() -> ColorPalette {
    palette(Scheme::Dark)
}

pub(crate) fn light() -> ColorPalette {
    palette(Scheme::Light)
}

pub(crate) fn uses_provider_accents(colours: &ColorSchemes) -> bool {
    colours.dark.accents.is_empty() && colours.light.accents.is_empty()
}

pub(crate) fn resolve_accents(colours: &mut ColorSchemes, seed: u64) {
    colours.dark.accents = accents(Scheme::Dark, seed);
    colours.light.accents = accents(Scheme::Light, seed);
}

fn palette(scheme: Scheme) -> ColorPalette {
    let provider = theme(scheme);
    let ui = provider.ui();
    ColorPalette {
        background: hex(ui.bg()),
        surface: hex(ui.bg_surface()),
        surface_strong: hex(ui.border()),
        border: hex(ui.border()),
        text: hex(ui.fg()),
        text_muted: hex(ui.fg_muted()),
        text_subtle: hex(ui.fg_subtle()),
        brand: hex(ui.brand()),
        brand_hover: hex(ui.brand_hover()),
        brand_active: hex(ui.brand_active()),
        on_brand: hex(ui.on_brand()),
        selection: hex(ui.selection()),
        focus: hex(ui.focus()),
        success: hex(ui.success()),
        warning: hex(ui.warning()),
        danger: hex(ui.error()),
        header_background: alpha(ui.bg(), 92),
        shadow: match scheme {
            Scheme::Dark => alpha(ui.bg(), 28),
            Scheme::Light => alpha(ui.fg(), 14),
        },
        accents: Vec::new(),
    }
}

fn accents(scheme: Scheme, seed: u64) -> Vec<String> {
    let palette = theme(scheme).colour();
    (0..ACCENT_COUNT)
        .map(|slot| hex(palette.random_seeded(provider_seed(seed, slot))))
        .collect()
}

fn provider_seed(seed: u64, slot: usize) -> u32 {
    let mut mixed = seed.wrapping_add(
        u64::try_from(slot + 1)
            .expect("accent slot fits u64")
            .wrapping_mul(0x9e37_79b9_7f4a_7c15),
    );
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    mixed ^= mixed >> 31;
    let bytes = mixed.to_le_bytes();
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        ^ u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
}

fn hex(colour: Colour) -> String {
    colour.hex().to_owned()
}

fn alpha(colour: Colour, percent: u8) -> String {
    let [red, green, blue] = colour.rgb();
    format!("rgb({red} {green} {blue} / {percent}%)")
}

#[cfg(test)]
mod tests {
    use super::provider_seed;

    #[test]
    fn provider_seed_mixes_slots_and_high_build_seed_bits() {
        let first = (0..6)
            .map(|slot| provider_seed(7, slot))
            .collect::<Vec<_>>();

        assert_eq!(
            first,
            (0..6)
                .map(|slot| provider_seed(7, slot))
                .collect::<Vec<_>>()
        );
        assert!(first.windows(2).all(|seeds| seeds[0] != seeds[1]));
        assert!(
            first
                .windows(2)
                .all(|seeds| seeds[1] != seeds[0].wrapping_add(1))
        );
        assert_ne!(provider_seed(7, 0), provider_seed((1_u64 << 48) | 7, 0));
    }
}
