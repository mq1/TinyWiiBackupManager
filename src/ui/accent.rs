// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use eframe::egui::Color32;
use std::process::Command;

const RED: Color32 = Color32::from_rgba_unmultiplied_const(236, 95, 93, 127);
const ORANGE: Color32 = Color32::from_rgba_unmultiplied_const(232, 136, 58, 127);
const YELLOW: Color32 = Color32::from_rgba_unmultiplied_const(246, 200, 68, 127);
const GREEN: Color32 = Color32::from_rgba_unmultiplied_const(120, 184, 86, 127);
const BLUE: Color32 = Color32::from_rgba_unmultiplied_const(52, 120, 246, 127);
const PURPLE: Color32 = Color32::from_rgba_unmultiplied_const(154, 85, 163, 127);
const PINK: Color32 = Color32::from_rgba_unmultiplied_const(228, 92, 156, 127);
const GRAY: Color32 = Color32::from_rgba_unmultiplied_const(140, 140, 140, 127);

pub fn get_accent_color() -> Color32 {
    if cfg!(target_os = "macos") {
        get_accent_macos().unwrap_or(ORANGE)
    } else {
        ORANGE
    }
}

fn get_accent_macos() -> Result<Color32> {
    let output = Command::new("defaults")
        .args(["read", "-g", "AppleAccentColor"])
        .output()?;

    match String::from_utf8_lossy(&output.stdout).trim() {
        "0" => Ok(RED),
        "1" => Ok(ORANGE),
        "2" => Ok(YELLOW),
        "3" => Ok(GREEN),
        "4" => Ok(BLUE),
        "5" => Ok(PURPLE),
        "6" => Ok(PINK),
        "-1" => Ok(GRAY),
        _ => Err(anyhow!("Unknown accent color")),
    }
}
