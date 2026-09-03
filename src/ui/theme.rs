// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use dark_light::Mode;
use iced::{
    Color, Theme,
    theme::{
        Palette,
        palette::{EXTENDED_DARK, Extended},
    },
};

#[cfg(target_os = "linux")]
fn accent() -> Option<Color> {
    // TODO
    None
}

#[cfg(target_os = "macos")]
fn accent() -> Option<Color> {
    // TODO
    None
}

#[cfg(target_os = "windows")]
fn accent() -> Option<Color> {
    let argb = winsafe::DwmGetColorizationColor().ok()?;
    let [_, r, g, b] = argb.to_be_bytes();
    Some(Color::from_rgb8(r, g, b))
}

fn generate(palette: Palette) -> Extended {
    let mut extended = Extended::generate(palette);
    extended.primary.base.text = EXTENDED_DARK.primary.base.text;
    extended
}

fn make_theme(accent: Color, is_dark: bool) -> Theme {
    let (name, base) = if is_dark {
        ("twbm-dark", Palette::DARK)
    } else {
        ("twbm-light", Palette::LIGHT)
    };

    Theme::custom_with_fn(
        name,
        Palette {
            primary: accent,
            ..base
        },
        generate,
    )
}

pub fn light() -> Theme {
    let Some(primary) = accent() else {
        return Theme::Light;
    };

    #[cfg(target_os = "windows")]
    crate::ui::window_color::set(false);

    make_theme(primary, false)
}

pub fn dark() -> Theme {
    let Some(primary) = accent() else {
        return Theme::Dark;
    };

    #[cfg(target_os = "windows")]
    crate::ui::window_color::set(true);

    make_theme(primary, true)
}

pub fn system() -> Option<Theme> {
    let is_dark = dark_light::detect().is_ok_and(|mode| mode == Mode::Dark);

    #[cfg(target_os = "windows")]
    crate::ui::window_color::set(is_dark);

    let accent = accent()?;
    Some(make_theme(accent, is_dark))
}
