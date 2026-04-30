// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use winsafe::{COLORREF, DwmAttr, HWND, co::DWMWCP};

const LIGHT: COLORREF = COLORREF::from_rgb(0xff, 0xff, 0xff);
const DARK: COLORREF = COLORREF::from_rgb(0x1e, 0x1e, 0x1e);

pub fn set(is_dark: bool) {
    let Some(hwnd) = HWND::GetActiveWindow() else {
        return;
    };

    // disable backdrop blur
    let attr = DwmAttr::UseHostBackdropBrush(false);
    let _ = hwnd.DwmSetWindowAttribute(attr);

    // set window color to mimick macos
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
