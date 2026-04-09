// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::xp_dialogs::get_hwnd;
use windows::Win32::Graphics::Dwm::DWMWA_CAPTION_COLOR;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;

pub fn set(window: &slint::Window, is_dark: bool) {
    let color: u32 = if is_dark {
        0x00_1e_1e_1e
    } else {
        0x00_ff_ff_ff
    };

    let Some(hwnd) = get_hwnd(window) else {
        return;
    };

    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &color as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        );
    }
}
