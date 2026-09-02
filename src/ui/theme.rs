// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{Color, Theme, theme::Palette};
use mundy::{ColorScheme, Interest, Preferences, Srgba};
use std::time::Duration;

fn srgba_to_color(color: Srgba) -> Color {
    Color {
        r: color.red as f32,
        g: color.green as f32,
        b: color.blue as f32,
        a: color.alpha as f32,
    }
}

fn accent() -> Option<Color> {
    let prefs = Preferences::once_blocking(Interest::AccentColor, Duration::from_millis(100))?;
    let accent = prefs.accent_color.0?;
    let color = srgba_to_color(accent);

    Some(color)
}

fn prefs() -> Option<(Color, bool)> {
    let prefs = Preferences::once_blocking(
        Interest::AccentColor | Interest::ColorScheme,
        Duration::from_millis(100),
    )?;

    let accent = prefs.accent_color.0?;
    let color = srgba_to_color(accent);
    let is_dark = prefs.color_scheme == ColorScheme::Dark;

    Some((color, is_dark))
}

pub fn light() -> Theme {
    let Some(primary) = accent() else {
        return Theme::Light;
    };

    Theme::custom(
        "twbm-light",
        Palette {
            primary,
            ..Palette::LIGHT
        },
    )
}

pub fn dark() -> Theme {
    let Some(primary) = accent() else {
        return Theme::Dark;
    };

    Theme::custom(
        "twbm-dark",
        Palette {
            primary,
            ..Palette::DARK
        },
    )
}

pub fn system() -> Option<Theme> {
    let (primary, is_dark) = prefs()?;

    if is_dark {
        Some(Theme::custom(
            "twbm-dark",
            Palette {
                primary,
                ..Palette::DARK
            },
        ))
    } else {
        Some(Theme::custom(
            "twbm-light",
            Palette {
                primary,
                ..Palette::LIGHT
            },
        ))
    }
}
