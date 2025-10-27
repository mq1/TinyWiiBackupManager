// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui;
use std::process::Command;

pub fn system_accent_color() -> Option<egui::Color32> {
    if cfg!(target_os = "macos") {
        return system_accent_color_macos();
    }

    if cfg!(target_os = "windows") {
        return system_accent_color_windows();
    }

    None
}

pub fn system_accent_color_macos() -> Option<egui::Color32> {
    let output = Command::new("defaults")
        .args(&["read", "-g", "AppleAccentColor"])
        .output()
        .ok()?;

    match String::from_utf8_lossy(&output.stdout).trim() {
        "0" => Some(egui::Color32::from_rgba_unmultiplied(236, 95, 93, 127)), // red
        "1" => Some(egui::Color32::from_rgba_unmultiplied(232, 136, 58, 127)), // orange
        "2" => Some(egui::Color32::from_rgba_unmultiplied(246, 200, 68, 127)), // yellow
        "3" => Some(egui::Color32::from_rgba_unmultiplied(120, 184, 86, 127)), // green
        "4" => Some(egui::Color32::from_rgba_unmultiplied(52, 120, 246, 127)), // blue
        "5" => Some(egui::Color32::from_rgba_unmultiplied(154, 85, 163, 127)), // purple
        "6" => Some(egui::Color32::from_rgba_unmultiplied(228, 92, 156, 127)), // pink
        "-1" => Some(egui::Color32::from_rgba_unmultiplied(140, 140, 140, 127)), // gray
        _ => None,
    }
}

pub fn system_accent_color_windows() -> Option<egui::Color32> {
    let output = Command::new("reg")
        .args(&[
            "query",
            "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent",
            "/v",
            "AccentColorMenu",
        ])
        .output()
        .ok()?;

    let hex_str = String::from_utf8_lossy(&output.stdout)
        .trim()
        .split_whitespace()
        .last()?
        .replace("0x", "");

    if hex_str.len() != 8 {
        return None;
    }

    let r = u8::from_str_radix(&hex_str[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex_str[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex_str[4..6], 16).ok()?;

    Some(egui::Color32::from_rgba_unmultiplied(r, g, b, 127))
}
