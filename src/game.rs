// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::titles::GAME_TITLES;
use anyhow::{Context, Result};
use phf::phf_map;
use std::path::PathBuf;

// for gametdb images
static REGION_TO_LANG: phf::Map<char, &'static str> = phf_map! {
    'A' => "EN", // System Wii Channels (i.e. Mii Channel)
    'B' => "EN", // Ufouria: The Saga (NA)
    'D' => "DE", // Germany
    'E' => "US", // USA
    'F' => "FR", // France
    'H' => "NL", // Netherlands
    'I' => "IT", // Italy
    'J' => "JA", // Japan
    'K' => "KO", // Korea
    'L' => "EN", // Japanese import to Europe, Australia and other PAL regions
    'M' => "EN", // American import to Europe, Australia and other PAL regions
    'N' => "US", // Japanese import to USA and other NTSC regions
    'P' => "EN", // Europe and other PAL regions such as Australia
    'Q' => "KO", // Japanese Virtual Console import to Korea
    'R' => "RU", // Russia
    'S' => "ES", // Spain
    'T' => "KO", // American Virtual Console import to Korea
    'U' => "EN", // Australia / Europe alternate languages
    'V' => "EN", // Scandinavia
    'W' => "ZH", // Republic of China (Taiwan) / Hong Kong / Macau
    'X' => "EN", // Europe alternate languages / US special releases
    'Y' => "EN", // Europe alternate languages / US special releases
    'Z' => "EN", // Europe alternate languages / US special releases
};

#[derive(Clone)]
pub struct Game {
    pub id: String,
    pub is_gc: bool,
    pub display_title: String,
    pub path: PathBuf,
    pub language: String,
    pub image_url: String,
}

impl Game {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid file name")?;

        let (id_start, id_end) = (
            file_name.rfind('[').context("No '[' in file name")? + 1,
            file_name.rfind(']').context("No ']' in file name")?,
        );

        let id = file_name[id_start..id_end].to_string();

        let is_gc = id.chars().next() == Some('G');

        let title = path
            .file_stem()
            .and_then(|n| n.to_str())
            .map(|n| n.trim_end_matches(&format!(" [{id}]")))
            .context("Failed to get title")?;

        let display_title = GAME_TITLES
            .get(&*id)
            .map_or_else(|| format!("{title} [{id}]"), |&s| s.into());

        // the 4th character in the ID is the region code
        let region_code = id.chars().nth(3).context("No region code in ID")?;
        let language = REGION_TO_LANG
            .get(&region_code)
            .copied()
            .unwrap_or("EN")
            .to_string();

        let image_url = format!(
            "https://art.gametdb.com/wii/cover3D/{}/{}.png",
            language, id
        );

        Ok(Self {
            id,
            is_gc,
            display_title,
            path,
            language,
            image_url,
        })
    }

    pub fn remove(&self) -> Result<()> {
        let confirm = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!(
                "Are you sure you want to remove {}?",
                self.display_title
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if confirm == rfd::MessageDialogResult::No {
            return Ok(());
        }

        std::fs::remove_dir_all(&self.path)
            .with_context(|| format!("Failed to remove game: {}", self.path.display()))
    }
}
