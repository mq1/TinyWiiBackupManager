// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::ThemePreference;
use iced::Window;
use std::ffi::c_void;
use wgpu::rwh::RawWindowHandle;
use winsafe::{COLORREF, DwmAttr, HWND};

pub fn set(window: &dyn Window, mut theme: ThemePreference) {
    if theme == ThemePreference::System
        && let Ok(mode) = dark_light::detect()
    {
        if mode == dark_light::Mode::Light {
            theme = ThemePreference::Light;
        } else if mode == dark_light::Mode::Dark {
            theme = ThemePreference::Dark;
        }
    }

    let color = match theme {
        ThemePreference::Light => COLORREF::from_rgb(0xff, 0xff, 0xff),
        ThemePreference::Dark => COLORREF::from_rgb(0x2b, 0x2d, 0x31),
        ThemePreference::System => {
            return;
        }
    };

    let handle = window.window_handle().unwrap().as_raw();

    if let RawWindowHandle::Win32(handle) = handle {
        let hwnd_ptr = handle.hwnd.get() as *mut c_void;
        let hwnd = unsafe { HWND::from_ptr(hwnd_ptr) };
        let attr = DwmAttr::CaptionColor(color);
        let _ = hwnd.DwmSetWindowAttribute(attr);
    }
}
