// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{
    Color, Theme,
    theme::{
        Palette,
        palette::{EXTENDED_DARK, Extended},
    },
};
use mundy::{ColorScheme, Interest, Preferences, Srgba};
use std::time::Duration;

fn accent() -> Option<Srgba> {
    let prefs = Preferences::once_blocking(Interest::AccentColor, Duration::from_millis(100))?;
    let accent = prefs.accent_color.0?;

    Some(accent)
}

fn accent_and_dark() -> Option<(Srgba, bool)> {
    let prefs = Preferences::once_blocking(
        Interest::AccentColor | Interest::ColorScheme,
        Duration::from_millis(100),
    )?;

    let accent = prefs.accent_color.0?;
    let is_dark = prefs.color_scheme == ColorScheme::Dark;

    Some((accent, is_dark))
}

fn generate(palette: Palette) -> Extended {
    let mut extended = Extended::generate(palette);
    extended.primary.base.text = EXTENDED_DARK.primary.base.text;
    extended
}

fn make_theme(accent: Srgba, is_dark: bool) -> Theme {
    let accent = Color {
        r: accent.red as f32,
        g: accent.green as f32,
        b: accent.blue as f32,
        a: 1.0,
    };

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
    let (accent, is_dark) = accent_and_dark()?;

    #[cfg(target_os = "windows")]
    crate::ui::window_color::set(is_dark);

    Some(make_theme(accent, is_dark))
}
