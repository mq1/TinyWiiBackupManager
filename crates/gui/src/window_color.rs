// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ffi::c_void;
use winsafe::{COLORREF, DwmAttr, HWND};

const LIGHT: COLORREF = COLORREF::from_rgb(0xff, 0xff, 0xff);
const DARK: COLORREF = COLORREF::from_rgb(0x1e, 0x1e, 0x1e);

pub fn set(window_handle: &slint::WindowHandle, is_dark: bool) {
    let Ok(handle) = window_handle.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };

    let hwnd = unsafe { HWND::from_ptr(handle.hwnd.get() as *mut c_void) };
    let color = if is_dark { DARK } else { LIGHT };
    let attr = DwmAttr::CaptionColor(color);
    let _ = hwnd.DwmSetWindowAttribute(attr);
}
