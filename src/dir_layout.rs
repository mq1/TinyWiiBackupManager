// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_info::DiscInfo,
    overflow_reader::{get_overflow_file, get_overflow_file_name},
};
use anyhow::{Result, bail};
use nod::common::Format;
use sanitize_filename::sanitize;
use std::{fs, path::Path};
use walkdir::{DirEntry, WalkDir};

fn filter(path: &Path) -> bool {
    // Ignore directories
    if path.is_dir() {
        return false;
    }

    // Ignore invalid files
    let file_name = if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        file_name
    } else {
        return false;
    };
    let file_extension = if let Some(file_extension) = path.extension().and_then(|s| s.to_str()) {
        file_extension
    } else {
        return false;
    };

    // Ignore hidden files
    if file_name.starts_with(".") {
        return false;
    }

    // Ignore unsupported extensions
    if !matches!(file_extension, "iso" | "wbfs" | "gcm" | "ciso") {
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
        for path in WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .map(DirEntry::into_path)
            .filter(|path| filter(path))
        {
            if let Ok(game) = DiscInfo::from_main_file(path) {
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

pub fn normalize_paths(mount_point: &Path) -> Result<()> {
    for disc_info in scan_for_games(mount_point) {
        log::info!("Found game: {}", disc_info.main_disc_path.display());

        let game_id = disc_info.header.game_id_str();

        let dir = mount_point
            .join(if disc_info.header.is_wii() {
                "wbfs"
            } else {
                "games"
            })
            .join(format!(
                "{} [{}]",
                sanitize(disc_info.header.game_title_str()),
                game_id
            ));

        let original_file_parent = if let Some(parent) = disc_info.main_disc_path.parent() {
            parent
        } else {
            bail!("Failed to get file parent");
        };

        let file_name1 = match (
            disc_info.meta.format,
            disc_info.header.is_wii(),
            disc_info.header.disc_num,
        ) {
            (Format::Wbfs, _, _) => &format!("{}.wbfs", game_id),
            (Format::Iso, true, _) => &format!("{}.iso", game_id),
            (Format::Iso, false, 0) => "game.iso",
            (Format::Iso, false, n) => &format!("disc{}.iso", n),
            (Format::Ciso, _, 0) => "game.ciso",
            (Format::Ciso, _, n) => &format!("disc{}.ciso", n),
            _ => bail!("Unsupported format"),
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

        let memcard_file_name = format!("memcard_{}.bin", game_id);
        let memcard_orig_path = original_file_parent.join(&memcard_file_name);
        if memcard_orig_path.exists() {
            let memcard_dest_path = dir.join(memcard_file_name);
            fs::rename(&memcard_orig_path, &memcard_dest_path)?;
        }
    }

    remove_empty_dirs(mount_point);

    Ok(())
}
