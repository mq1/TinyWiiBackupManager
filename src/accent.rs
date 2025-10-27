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
        "0" => Some(egui::Color32::RED.gamma_multiply_u8(127)),
        "1" => Some(egui::Color32::ORANGE.gamma_multiply_u8(127)),
        "2" => Some(egui::Color32::YELLOW.gamma_multiply_u8(127)),
        "3" => Some(egui::Color32::GREEN.gamma_multiply_u8(127)),
        "4" => Some(egui::Color32::BLUE.gamma_multiply_u8(127)),
        "5" => Some(egui::Color32::PURPLE.gamma_multiply_u8(127)),
        "6" => Some(egui::Color32::MAGENTA.gamma_multiply_u8(127)),
        "-1" => Some(egui::Color32::GRAY.gamma_multiply_u8(127)),
        _ => None,
    }
}

#[cfg(windows)]
pub fn system_accent_color() -> Option<String> {
    use windows::UI::ViewManagemen::UIColorType;
    use windows::UI::ViewManagement::UISettings;

    let settings = UISettings::new().ok()?;
    let color = settings.GetColorValue(UIColorType::Accent).ok()?;

    let color = egui::Color32::from_rgba_unmultiplied(color.R, color.G, color.B, color.A);
    Some(color)
}
