// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use eframe::{
    egui::{Context, FontData, FontDefinitions},
    epaint::FontFamily,
};
use fontdb::{Database, Family, Query, Source};
use std::fs;
use std::sync::Arc;

pub fn load_system_font(ctx: &Context) -> Result<()> {
    let mut fonts = FontDefinitions::default();

    // Create a font database and load system fonts into it
    let mut db = Database::new();
    db.load_system_fonts();

    // Query for a sans-serif font
    let query = Query {
        families: &[Family::SansSerif],
        ..Default::default()
    };

    // Find the best match
    if let Some(id) = db.query(&query)
        && let Some((source, _)) = db.face_source(id) {
            let buffer = match source {
                Source::Binary(data) => data.as_ref().as_ref().to_vec(),
                Source::File(path) => fs::read(path)?,
                Source::SharedFile(_, data) => data.as_ref().as_ref().to_vec(),
            };

            fonts.font_data.insert(
                "system".to_string(),
                Arc::from(FontData::from_owned(buffer)),
            );

            if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
                vec.push("system".to_string());
            }
        }

    // Query for a monospace font
    let query = Query {
        families: &[Family::Monospace],
        ..Default::default()
    };

    if let Some(id) = db.query(&query)
        && let Some((source, _)) = db.face_source(id) {
            let buffer = match source {
                Source::Binary(data) => data.as_ref().as_ref().to_vec(),
                Source::File(path) => fs::read(path)?,
                Source::SharedFile(_, data) => data.as_ref().as_ref().to_vec(),
            };

            fonts.font_data.insert(
                "system_monospace".to_string(),
                Arc::from(FontData::from_owned(buffer)),
            );

            if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
                vec.push("system_monospace".to_string());
            }
        }

    // Add Phosphor icons
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
    Ok(())
}
