// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::ThemePreference;
use iced::Window;
use wgpu::rwh::RawWindowHandle;

pub fn set(window: &dyn Window, mut theme: ThemePreference) {
    if theme == ThemePreference::System
        && let Ok(mode) = dark_light::detect()
    {
        match mode {
            dark_light::Mode::Light => {
                theme = ThemePreference::Light;
            }
            dark_light::Mode::Dark => {
                theme = ThemePreference::Dark;
            }
            dark_light::Mode::Unspecified => {
                return;
            }
        }
    }

    let color: u32 = match theme {
        ThemePreference::Light => 0xFFFFFF,
        ThemePreference::Dark => 0x312D2B,
        ThemePreference::System => {
            return;
        }
    };

    let handle = window.window_handle().unwrap().as_raw();

    if let RawWindowHandle::Win32(handle) = handle {
        let hwnd = windows::Win32::Foundation::HWND(handle.hwnd.get() as *mut _);

        unsafe {
            windows::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                hwnd,
                windows::Win32::Graphics::Dwm::DWMWA_CAPTION_COLOR,
                (&raw const color).cast(),
                4,
            )
            .unwrap();
        }
    }
}
