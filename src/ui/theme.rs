// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{
    Color, Theme,
    theme::{
        Palette,
        palette::{EXTENDED_DARK, Extended},
    },
};

#[cfg(any(target_os = "linux", all(target_os = "macos", target_arch = "aarch64")))]
/// works on macos 10.14+, macos arm64 starts at 11
/// enable this on x86_64 if we're dropping support for < 10.14
fn accent() -> Option<Color> {
    let prefs = mundy::Preferences::once_blocking(
        mundy::Interest::AccentColor,
        std::time::Duration::from_millis(100),
    )?;

    let accent = prefs.accent_color.0?;

    Some(Color::from_rgb(
        accent.red as f32,
        accent.green as f32,
        accent.blue as f32,
    ))
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn accent() -> Option<Color> {
    None
}

#[cfg(target_os = "windows")]
fn accent() -> Option<Color> {
    let argb = winsafe::DwmGetColorizationColor().ok()?;
    let [_, r, g, b] = argb.to_be_bytes();
    Some(Color::from_rgb8(r, g, b))
}

#[cfg(any(target_os = "linux", all(target_os = "macos", target_arch = "aarch64")))]
/// works on macos 10.14+, macos arm64 starts at 11
/// enable this on x86_64 if we're dropping support for < 10.14
fn is_dark() -> bool {
    mundy::Preferences::once_blocking(
        mundy::Interest::ColorScheme,
        std::time::Duration::from_millis(100),
    )
    .is_some_and(|prefs| prefs.color_scheme == mundy::ColorScheme::Dark)
}

#[cfg(any(
    target_os = "windows",
    all(target_os = "macos", target_arch = "x86_64")
))]
fn is_dark() -> bool {
    dark_light::detect().is_ok_and(|mode| mode == dark_light::Mode::Dark)
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
    let is_dark = is_dark();

    #[cfg(target_os = "windows")]
    crate::ui::window_color::set(is_dark);

    accent().map(move |accent| make_theme(accent, is_dark))
}
