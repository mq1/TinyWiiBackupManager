// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use eframe::{
    egui::{Context, FontData, FontDefinitions},
    epaint::FontFamily,
};
use std::sync::Arc;

pub fn load_fonts(ctx: &Context) -> Result<()> {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "Ubuntu".to_string(),
        Arc::from(FontData::from_static(include_bytes!(
            "../assets/Ubuntu-Regular.ttf"
        ))),
    );

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push("Ubuntu".to_string());
    }

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.push("Ubuntu".to_string());
    }

    // Add Phosphor icons
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
    Ok(())
}
