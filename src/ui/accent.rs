// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};

const RED: Color32 = Color32::from_rgba_unmultiplied_const(236, 95, 93, 127);
const ORANGE: Color32 = Color32::from_rgba_unmultiplied_const(232, 136, 58, 127);
const YELLOW: Color32 = Color32::from_rgba_unmultiplied_const(246, 200, 68, 127);
const GREEN: Color32 = Color32::from_rgba_unmultiplied_const(120, 184, 86, 127);
const BLUE: Color32 = Color32::from_rgba_unmultiplied_const(52, 120, 246, 127);
const PURPLE: Color32 = Color32::from_rgba_unmultiplied_const(154, 85, 163, 127);
const PINK: Color32 = Color32::from_rgba_unmultiplied_const(228, 92, 156, 127);
const GRAY: Color32 = Color32::from_rgba_unmultiplied_const(140, 140, 140, 127);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AccentColor {
    System,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Gray,
}

impl AccentColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccentColor::System => "System",
            AccentColor::Red => "Red",
            AccentColor::Orange => "Orange",
            AccentColor::Yellow => "Yellow",
            AccentColor::Green => "Green",
            AccentColor::Blue => "Blue",
            AccentColor::Purple => "Purple",
            AccentColor::Pink => "Pink",
            AccentColor::Gray => "Gray",
        }
    }
}

impl From<AccentColor> for Color32 {
    fn from(accent_color: AccentColor) -> Self {
        match accent_color {
            AccentColor::System => get_accent().unwrap_or(ORANGE),
            AccentColor::Red => RED,
            AccentColor::Orange => ORANGE,
            AccentColor::Yellow => YELLOW,
            AccentColor::Green => GREEN,
            AccentColor::Blue => BLUE,
            AccentColor::Purple => PURPLE,
            AccentColor::Pink => PINK,
            AccentColor::Gray => GRAY,
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn get_accent() -> Option<Color32> {
    None
}

#[cfg(target_os = "macos")]
fn get_accent() -> Option<Color32> {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleAccentColor"])
        .output()
        .ok()?;

    match String::from_utf8_lossy(&output.stdout).trim() {
        "0" => Some(RED),
        "1" => Some(ORANGE),
        "2" => Some(YELLOW),
        "3" => Some(GREEN),
        "4" => Some(BLUE),
        "5" => Some(PURPLE),
        "6" => Some(PINK),
        "-1" => Some(GRAY),
        _ => None,
    }
}
