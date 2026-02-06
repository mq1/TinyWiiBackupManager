// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::Window;
use wgpu::rwh::RawWindowHandle;

pub fn set_color(window: &dyn Window, theme: iced::Theme) {
    let color = if theme == iced::Theme::Light {
        0x0000FFu32
    } else {
        0xFFFFFFu32
    };

    let handle = window.window_handle().unwrap().as_raw();

    if let RawWindowHandle::Win32(handle) = handle {
        let hwnd = windows::Win32::Foundation::HWND(handle.hwnd.get() as _);

        unsafe {
            windows::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                hwnd,
                windows::Win32::Graphics::Dwm::DWMWA_CAPTION_COLOR,
                color as _,
                4,
            )
            .unwrap();
        }
    }
}
