// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{Background, Theme, color};

pub struct MyPalette;

impl MyPalette {
    pub fn card_bg(theme: &Theme) -> Background {
        match theme {
            Theme::Light => Background::Color(color!(0xffffff)),
            Theme::Dark => Background::Color(color!(0x1e1e1e)),
            _ => unreachable!(),
        }
    }
}
