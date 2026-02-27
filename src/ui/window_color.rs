// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::ThemePreference;
use crate::message::Message;
use iced::Task;
use iced::window::{self, raw_window_handle::RawWindowHandle};
use std::ffi::c_void;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DWMWA_CAPTION_COLOR;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;

pub fn set(mut theme: ThemePreference) -> Task<Message> {
    if theme == ThemePreference::System
        && let Ok(mode) = dark_light::detect()
    {
        if mode == dark_light::Mode::Light {
            theme = ThemePreference::Light;
        } else if mode == dark_light::Mode::Dark {
            theme = ThemePreference::Dark;
        }
    }

    let color: u32 = match theme {
        ThemePreference::Light => 0x00_FF_FF_FF,
        ThemePreference::Dark => 0x00_31_2D_2B,
        ThemePreference::System => return Task::none(),
    };

    window::oldest()
        .and_then(move |id| {
            window::run(id, move |w| {
                let Ok(handle) = w.window_handle() else {
                    return;
                };

                if let RawWindowHandle::Win32(handle) = handle.as_raw() {
                    let hwnd = HWND(handle.hwnd.get() as *mut _);

                    unsafe {
                        let _ = DwmSetWindowAttribute(
                            hwnd,
                            DWMWA_CAPTION_COLOR,
                            &color as *const u32 as *const c_void,
                            std::mem::size_of::<u32>() as u32,
                        );
                    }
                }
            })
        })
        .discard()
}
