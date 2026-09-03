// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{Color, theme::Palette};
use winsafe::{COLORREF, DwmAttr, HWND, co::DWMWCP};

pub const fn colorref(color: Color) -> COLORREF {
    COLORREF::from_rgb(
        (color.r * 255.0).round() as u8,
        (color.g * 255.0).round() as u8,
        (color.b * 255.0).round() as u8,
    )
}

const LIGHT: COLORREF = colorref(Palette::LIGHT.background);
const DARK: COLORREF = colorref(Palette::DARK.background);

pub fn set(is_dark: bool) {
    let Some(hwnd) = HWND::GetActiveWindow() else {
        return;
    };

    // disable backdrop blur
    let attr = DwmAttr::UseHostBackdropBrush(false);
    let _ = hwnd.DwmSetWindowAttribute(attr);

    // set window color to match my_card background
    let color = if is_dark { DARK } else { LIGHT };
    let attr = DwmAttr::CaptionColor(color);
    let _ = hwnd.DwmSetWindowAttribute(attr);

    // set immersive dark mode
    let attr = DwmAttr::UseImmersiveDarkMode(is_dark);
    let _ = hwnd.DwmSetWindowAttribute(attr);

    // rounded corners
    let dwmwcp = DWMWCP::ROUND;
    let attr = DwmAttr::WindowCornerPreference(dwmwcp);
    let _ = hwnd.DwmSetWindowAttribute(attr);
}
