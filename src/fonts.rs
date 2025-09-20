// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::{Result, bail};
use eframe::{
    egui::{Context, FontData, FontDefinitions},
    epaint::FontFamily,
};
use fontdb::{Database, Family, Query, Source};
use std::fs;
use std::sync::Arc;

include!(concat!(env!("OUT_DIR"), "/phosphor_meta.rs"));

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
    if let Some(id) = db.query(&query) {
        if let Some((source, _)) = db.face_source(id) {
            let buffer = match source {
                Source::Binary(data) => data.as_ref().as_ref().to_vec(),
                Source::File(path) => fs::read(path)?,
                _ => bail!("Unsupported font source"),
            };

            fonts.font_data.insert(
                "system".to_string(),
                Arc::from(FontData::from_owned(buffer)),
            );

            if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
                vec.push("system".to_string());
            }
        }
    }

    let phosphor = include_bytes!(concat!(env!("OUT_DIR"), "/Phosphor.ttf.zst"));
    let phosphor = zstd::bulk::decompress(phosphor, PHOSPHOR_SIZE)?;
    fonts.font_data.insert(
        "phosphor".to_string(),
        Arc::from(FontData::from_owned(phosphor)),
    );
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.push("phosphor".to_string());
    }

    ctx.set_fonts(fonts);
    Ok(())
}
