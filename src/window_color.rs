// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::Window;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DWMWA_CAPTION_COLOR;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;

pub fn set(window: &Window, theme_preference: &str) -> Result<()> {
    let is_dark = match theme_preference {
        "dark" => true,
        "light" => false,
        _ => match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => true,
            Ok(dark_light::Mode::Light) => false,
            _ => bail!("Could not determine system theme"),
        },
    };

    let color: u32 = match is_dark {
        true => 0x00_1e_1e_1e,
        false => 0x00_ff_ff_ff,
    };

    let RawWindowHandle::Win32(handle) = window.window_handle().window_handle()?.as_raw() else {
        bail!("Could not get window handle");
    };

    let hwnd = HWND(handle.hwnd.get() as *mut _);

    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &color as *const u32 as *const _,
            std::mem::size_of::<u32>() as u32,
        )?;
    }

    Ok(())
}
