// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[allow(unused_imports)]
use anyhow::{Result, anyhow, bail};
use eframe::egui;
use std::process::Command;

#[cfg(target_os = "macos")]
pub fn system_accent_color() -> Result<egui::Color32> {
    let output = Command::new("defaults")
        .args(&["read", "-g", "AppleAccentColor"])
        .output()?;

    match String::from_utf8_lossy(&output.stdout).trim() {
        "0" => Ok(egui::Color32::from_rgba_unmultiplied(236, 95, 93, 127)), // red
        "1" => Ok(egui::Color32::from_rgba_unmultiplied(232, 136, 58, 127)), // orange
        "2" => Ok(egui::Color32::from_rgba_unmultiplied(246, 200, 68, 127)), // yellow
        "3" => Ok(egui::Color32::from_rgba_unmultiplied(120, 184, 86, 127)), // green
        "4" => Ok(egui::Color32::from_rgba_unmultiplied(52, 120, 246, 127)), // blue
        "5" => Ok(egui::Color32::from_rgba_unmultiplied(154, 85, 163, 127)), // purple
        "6" => Ok(egui::Color32::from_rgba_unmultiplied(228, 92, 156, 127)), // pink
        "-1" => Ok(egui::Color32::from_rgba_unmultiplied(140, 140, 140, 127)), // gray
        _ => Err(anyhow!("Unknown accent color")),
    }
}

// https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdperp/bc6975ee-c630-4414-ba10-04eecbb6fccc
#[cfg(windows)]
pub fn system_accent_color() -> Result<egui::Color32> {
    use std::os::windows::process::CommandExt;

    let output = Command::new("reg")
        .args(&[
            "query",
            "HKCU\\Software\\Microsoft\\Windows\\DWM",
            "/v",
            "ColorizationColor",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()?;

    let out_str = String::from_utf8_lossy(&output.stdout);
    let hex_str = out_str
        .trim()
        .split_whitespace()
        .last()
        .ok_or(anyhow!("Failed to parse accent color"))?;

    if hex_str.len() != 10 {
        bail!("Invalid accent color");
    }

    let r = u8::from_str_radix(&hex_str[4..6], 16)?;
    let g = u8::from_str_radix(&hex_str[6..8], 16)?;
    let b = u8::from_str_radix(&hex_str[8..10], 16)?;

    Ok(egui::Color32::from_rgba_unmultiplied(r, g, b, 127))
}
