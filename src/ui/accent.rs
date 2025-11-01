// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui::Color32;

pub fn get_accent_color() -> Color32 {
    system_accent_color().unwrap_or(
        // Fallback accent color (orange)
        Color32::from_rgba_unmultiplied(232, 136, 58, 127),
    )
}

#[cfg(target_os = "macos")]
fn system_accent_color() -> anyhow::Result<Color32> {
    use anyhow::anyhow;
    use std::process::Command;

    let output = Command::new("defaults")
        .args(&["read", "-g", "AppleAccentColor"])
        .output()?;

    match String::from_utf8_lossy(&output.stdout).trim() {
        "0" => Ok(Color32::from_rgba_unmultiplied(236, 95, 93, 127)), // red
        "1" => Ok(Color32::from_rgba_unmultiplied(232, 136, 58, 127)), // orange
        "2" => Ok(Color32::from_rgba_unmultiplied(246, 200, 68, 127)), // yellow
        "3" => Ok(Color32::from_rgba_unmultiplied(120, 184, 86, 127)), // green
        "4" => Ok(Color32::from_rgba_unmultiplied(52, 120, 246, 127)), // blue
        "5" => Ok(Color32::from_rgba_unmultiplied(154, 85, 163, 127)), // purple
        "6" => Ok(Color32::from_rgba_unmultiplied(228, 92, 156, 127)), // pink
        "-1" => Ok(Color32::from_rgba_unmultiplied(140, 140, 140, 127)), // gray
        _ => Err(anyhow!("Unknown accent color")),
    }
}

// https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdperp/bc6975ee-c630-4414-ba10-04eecbb6fccc
#[cfg(windows)]
fn system_accent_color() -> anyhow::Result<Color32> {
    use anyhow::{anyhow, bail};
    use std::os::windows::process::CommandExt;
    use std::process::Command;

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

    Ok(Color32::from_rgba_unmultiplied(r, g, b, 127))
}

#[cfg(target_os = "linux")]
fn system_accent_color() -> anyhow::Result<Color32> {
    use anyhow::anyhow;

    Err(anyhow!("System accent color not supported on Linux"))
}
