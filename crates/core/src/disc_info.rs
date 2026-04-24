// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use wii_disc_info::Meta;

#[derive(Debug, Clone)]
pub struct DiscInfo {
    pub path: PathBuf,
    pub meta: Meta,
    pub is_worth_scrubbing: bool,
    pub crc32: u32,
}

impl DiscInfo {
    pub fn from_game_dir(game_dir: &Path) -> Option<Self> {
        if !game_dir.is_dir() {
            return None;
        }

        let filename = game_dir.file_name()?.to_str()?;

        if filename.starts_with('.') {
            return None;
        }

        for entry in fs::read_dir(game_dir).ok()?.filter_map(Result::ok) {
            let disc_path = entry.path();

            if let Some(disc_info) = Self::from_path(disc_path) {
                return Some(disc_info);
            }
        }

        None
    }

    pub fn from_path(path: PathBuf) -> Option<Self> {
        if !path.is_file() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;

        if filename.starts_with('.') {
            return None;
        }

        if filename.ends_with(".part1.iso") {
            return None;
        }

        let ext = path.extension()?;
        if !ext.eq_ignore_ascii_case("iso")
            && !ext.eq_ignore_ascii_case("wbfs")
            && !ext.eq_ignore_ascii_case("ciso")
        {
            return None;
        }

        let mut f = File::open(&path).ok()?;
        let meta = wii_disc_info::Meta::read(&mut f).ok()?;

        let is_worth_scrubbing = (meta.format() == wii_disc_info::Format::Wbfs)
            && is_worth_scrubbing(&mut f).unwrap_or(false);

        let crc32_path = path.with_file_name(format!("{}.crc32", meta.game_id()));
        let crc32 = fs::read_to_string(&crc32_path).unwrap_or_default();
        let crc32 = u32::from_str_radix(&crc32, 16).unwrap_or_default();

        Some(Self {
            path,
            meta,
            is_worth_scrubbing,
            crc32,
        })
    }
}

// use this only on wbfs files
pub fn is_worth_scrubbing<R: Read + Seek>(disc_reader: &mut R) -> Result<bool> {
    let mut buf = [0u8; 4];

    // check if the first partition is an update one
    disc_reader.seek(SeekFrom::Start(0x0024_0024))?;
    disc_reader.read_exact(&mut buf)?;
    if buf != [0, 0, 0, 1] {
        return Ok(false);
    }

    // check if the update data is unmapped
    disc_reader.seek(SeekFrom::Start(0x302))?;
    disc_reader.read_exact(&mut buf)?;
    let worth_it = buf != [0, 0, 0, 0];

    Ok(worth_it)
}
