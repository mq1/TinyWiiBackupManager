// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui;

#[cfg(target_os = "macos")]
pub fn system_accent_color() -> Option<egui::Color32> {
    use std::process::Command;

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

#[cfg(windows)]
pub fn system_accent_color() -> Option<egui::Color32> {
    use windows::UI::ViewManagement::UIColorType;
    use windows::UI::ViewManagement::UISettings;

    let settings = UISettings::new().ok()?;
    let color = settings.GetColorValue(UIColorType::Accent).ok()?;

    Some(egui::Color32::from_rgba_unmultiplied(
        color.R, color.G, color.B, 127,
    ))
}
