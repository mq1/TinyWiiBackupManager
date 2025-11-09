// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_info::DiscInfo,
    overflow_reader::{get_overflow_file, get_overflow_file_name},
};
use anyhow::{Result, bail};
use sanitize_filename::sanitize;
use std::{fs, path::Path};
use walkdir::{DirEntry, WalkDir};

fn filter(entry: &DirEntry) -> bool {
    // Ignore directories
    if entry.file_type().is_dir() {
        return false;
    }

    // Ignore invalid files
    let file_name = if let Some(file_name) = entry.file_name().to_str() {
        file_name
    } else {
        return false;
    };

    // Ignore hidden files
    if file_name.starts_with(".") {
        return false;
    }

    // Ignore non-ISO/WBFS files
    if !file_name.ends_with(".iso") || file_name.ends_with(".wbfs") {
        return false;
    }

    // Ignore partial ISO files
    if file_name.ends_with(".part1.iso") {
        return false;
    }

    true
}

fn scan_for_games(mount_point: &Path) -> Box<[DiscInfo]> {
    let wii_dir = mount_point.join("wbfs");
    let games_dir = mount_point.join("games");

    let mut games: Vec<DiscInfo> = Vec::new();
    for dir in [wii_dir, games_dir] {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(filter)
        {
            if let Ok(game) = DiscInfo::from_main_file(entry.path().to_path_buf()) {
                games.push(game);
            }
        }
    }

    games.into_boxed_slice()
}

fn remove_empty_dirs(mount_point: &Path) {
    for dir_name in ["wbfs", "games"] {
        if let Ok(entries) = fs::read_dir(mount_point.join(dir_name)) {
            entries.filter_map(Result::ok).for_each(|entry| {
                let _ = fs::remove_dir(entry.path());
            });
        }
    }
}

pub fn bulk_rename(mount_point: &Path) -> Result<()> {
    for disc_info in scan_for_games(mount_point) {
        log::info!("Found game: {}", disc_info.main_disc_path.display());

        let dir = mount_point
            .join(if disc_info.header.is_wii() {
                "wbfs"
            } else {
                "games"
            })
            .join(format!(
                "{} [{}]",
                sanitize(disc_info.header.game_title_str()),
                disc_info.header.game_id_str()
            ));

        let original_file_name = if let Some(name) = disc_info
            .main_disc_path
            .file_name()
            .and_then(|s| s.to_str())
        {
            name
        } else {
            bail!("Failed to get file name");
        };

        let file_name1 = if original_file_name.ends_with(".wbfs") {
            disc_info.header.game_id_str().to_string() + ".wbfs"
        } else if original_file_name.ends_with(".part0.iso") {
            disc_info.header.game_id_str().to_string() + ".part0.iso"
        } else if disc_info.header.is_gamecube() && disc_info.header.disc_num == 0 {
            "game.iso".to_string()
        } else if disc_info.header.is_gamecube() {
            format!("disc{}.iso", disc_info.header.disc_num)
        } else {
            disc_info.header.game_id_str().to_string() + ".iso"
        };

        let file_name2 = get_overflow_file_name(&file_name1);

        let file1 = dir.join(file_name1);

        fs::create_dir_all(&dir)?;
        fs::rename(&disc_info.main_disc_path, &file1)?;

        if let Some(overflow_file) = get_overflow_file(&disc_info.main_disc_path)
            && let Some(file_name2) = file_name2
        {
            let file2 = dir.join(file_name2);
            fs::rename(&overflow_file, &file2)?;
        }
    }

    remove_empty_dirs(mount_point);

    Ok(())
}
