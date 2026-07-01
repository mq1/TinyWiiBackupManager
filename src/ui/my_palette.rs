// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::{Background, Color, Theme, color};

pub const RED: Color = color!(0xff0000);
pub const GREEN: Color = color!(0x00ff00);
pub const BLUE: Color = color!(0x0000ff);
pub const YELLOW: Color = color!(0xffff00);

pub fn card_bg(theme: &Theme) -> Background {
    match theme {
        Theme::Light => Background::Color(color!(0xffffff)),
        Theme::Dark => Background::Color(color!(0x25252c)),
        _ => unreachable!(),
    }
}
