// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ThemePreference;
use anyhow::{Result, bail};
use slint::Window;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DWMWA_CAPTION_COLOR;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;

pub fn set(window: &Window, mut theme: ThemePreference) -> Result<()> {
    if theme == ThemePreference::System {
        match dark_light::detect() {
            Ok(dark_light::Mode::Light) => {
                theme = ThemePreference::Light;
            }
            Ok(dark_light::Mode::Dark) => {
                theme = ThemePreference::Dark;
            }
            _ => {}
        }
    }

    let color: u32 = match theme {
        ThemePreference::Light => 0x00_ff_ff_ff,
        ThemePreference::Dark => 0x00_1e_1e_1e,
        ThemePreference::System => bail!("Could not determine system theme"),
    };

    let RawWindowHandle::Win32(handle) = window.window_handle().as_raw() else {
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
