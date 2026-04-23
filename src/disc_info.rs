// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DiscInfo, scrub::is_worth_scrubbing};
use anyhow::{Result, anyhow, bail};
use slint::{SharedString, ToSharedString};
use std::{
    ffi::OsStr,
    fs::{self, File},
    path::Path,
};

impl DiscInfo {
    pub fn try_from_game_dir(game_dir: &Path) -> Result<Self> {
        if !game_dir.is_dir() {
            bail!("Not a directory");
        }

        let Some(filename) = game_dir.file_name().and_then(OsStr::to_str) else {
            bail!("No file name");
        };

        if filename.starts_with('.') {
            bail!("Hidden directory");
        }

        for entry in fs::read_dir(game_dir)?.filter_map(Result::ok) {
            let disc_path = entry.path();

            if let Ok(disc_info) = Self::try_from_path(&disc_path) {
                return Ok(disc_info);
            }
        }

        Err(anyhow!("No disc file found"))
    }

    pub fn try_from_path(disc_path: &Path) -> Result<Self> {
        if !disc_path.is_file() {
            bail!("Not a file");
        }

        let Some(filename) = disc_path.file_name().and_then(OsStr::to_str) else {
            bail!("No file name");
        };

        if filename.starts_with('.') {
            bail!("Hidden file");
        }

        if filename.ends_with(".part1.iso") {
            bail!("Part 1 file");
        }

        let Some(ext) = disc_path.extension() else {
            bail!("No file extension");
        };

        if !ext.eq_ignore_ascii_case("iso")
            && !ext.eq_ignore_ascii_case("wbfs")
            && !ext.eq_ignore_ascii_case("ciso")
        {
            bail!("Unsupported file extension");
        }

        let mut f = File::open(disc_path)?;
        let meta = wii_disc_info::Meta::read(&mut f)?;

        let is_worth_scrubbing = (meta.format() == wii_disc_info::Format::Wbfs)
            && is_worth_scrubbing(&mut f).unwrap_or(false);

        Ok(Self {
            path: disc_path.to_string_lossy().to_shared_string(),
            format: meta.format().to_shared_string(),
            game_id: meta.game_id().to_shared_string(),
            game_title: meta.game_title().to_shared_string(),
            region: meta.region().to_shared_string(),
            is_wii: meta.is_wii(),
            is_gc: meta.is_gc(),
            disc_number: meta.disc_number().into(),
            disc_version: meta.disc_version().into(),
            is_worth_scrubbing,
        })
    }
}
