// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::Window;
use wgpu::rwh::RawWindowHandle;

pub fn set(window: &dyn Window, mode: iced::theme::Mode) {
    let color: u32 = match mode {
        iced::theme::Mode::Light => 0xFFFFFF,
        iced::theme::Mode::Dark => 0x312D2B,
        iced::theme::Mode::None => 0xFFFFFF,
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
