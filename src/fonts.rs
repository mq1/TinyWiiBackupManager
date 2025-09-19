// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::{
    egui::{Context, FontData, FontDefinitions},
    epaint::FontFamily,
};
use std::fs;
use std::sync::Arc;

const FONT_PATHS: &[&str] = &[
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "C:\\Windows\\Fonts\\segoeui.ttf",
];

pub fn load_system_font(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    for (i, font_path) in FONT_PATHS.iter().enumerate() {
        if let Ok(bytes) = fs::read(font_path) {
            fonts
                .font_data
                .insert(i.to_string(), Arc::from(FontData::from_owned(bytes)));

            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .push(i.to_string());

            fonts
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .push(i.to_string());
        }
    }

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
}
