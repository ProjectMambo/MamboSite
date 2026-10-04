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
        .map(|slot| {
            let seed = seed
                .wrapping_mul(ACCENT_COUNT as u64)
                .wrapping_add(slot as u64);
            hex(palette.random_seeded(seed))
        })
        .collect()
}

fn hex(colour: Colour) -> String {
    colour.hex().to_owned()
}

fn alpha(colour: Colour, percent: u8) -> String {
    let [red, green, blue] = colour.rgb();
    format!("rgb({red} {green} {blue} / {percent}%)")
}
