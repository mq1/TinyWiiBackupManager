// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_util::read_disc_header_from_game_dir,
    games::{extensions::format_to_ext, util::maybe_path_to_entry},
    util::sanitize,
};
use anyhow::Result;
use nod::common::Format;
use std::{
    ffi::OsStr,
    fs::{self},
    path::Path,
};

fn adopt_orphaned_discs(games_dir: &Path, is_wii: bool) -> Result<()> {
    let all_discs = fs::read_dir(games_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(maybe_path_to_entry);

    for (path, format, game_id, title) in all_discs {
        let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        let title = if is_wii {
            sanitize(&title)
        } else {
            sanitize(&title)
                .replace(" game disc 1", "")
                .replace(" game disc 2", "")
        };

        let new_parent_name = format!("{} [{}]", title, game_id.as_str());

        let new_filename = if filename.ends_with(".part0.iso") {
            format!("{}.part0.iso", game_id.as_str())
        } else if is_wii {
            format!("{}.{}", game_id.as_str(), format_to_ext(format))
        } else {
            format!("game.{}", format_to_ext(format))
        };

        let new_parent = games_dir.join(new_parent_name);
        let new_path = new_parent.join(&new_filename);

        fs::create_dir_all(&new_parent)?;
        fs::rename(&path, &new_path)?;

        // handle split files
        if format == Format::Wbfs {
            let wbf1_path = path.with_extension("wbf1");
            if wbf1_path.exists() {
                let new_wbf1_path = new_path.with_extension("wbf1");
                fs::rename(wbf1_path, new_wbf1_path)?;
            }
        } else if filename.ends_with(".part0.iso") {
            let part1_orig = games_dir.join(filename.replace(".part0.iso", ".part1.iso"));
            if part1_orig.exists() {
                let part1_new = new_parent.join(new_filename.replace(".part0.iso", ".part1.iso"));
                fs::rename(part1_orig, part1_new)?;
            }
        }
    }

    Ok(())
}

fn readopt_parented_discs(games_dir: &Path, is_wii: bool) -> Result<()> {
    let all_game_dirs = fs::read_dir(games_dir)?.filter_map(|entry| {
        let path = entry.ok()?.path();
        let (format, game_id, title) = read_disc_header_from_game_dir(&path)?;
        Some((path, format, game_id, title))
    });

    for (path, _, game_id, title) in all_game_dirs {
        let title = if is_wii {
            sanitize(&title)
        } else {
            sanitize(&title)
                .replace(" game disc 1", "")
                .replace(" game disc 2", "")
        };

        let new_filename = format!("{} [{}]", title, game_id.as_str());
        let new_path = games_dir.join(new_filename);

        if new_path.exists() {
            continue;
        }

        fs::rename(path, &new_path)?;
    }

    Ok(())
}

pub fn normalize_paths(mount_point: &Path) -> Result<String> {
    let wii_dir = mount_point.join("wbfs");
    let gc_dir = mount_point.join("games");

    fs::create_dir_all(&wii_dir)?;
    fs::create_dir_all(&gc_dir)?;

    adopt_orphaned_discs(&wii_dir, true)?;
    adopt_orphaned_discs(&gc_dir, false)?;

    readopt_parented_discs(&wii_dir, true)?;
    readopt_parented_discs(&gc_dir, false)?;

    Ok("Paths successfully normalized".to_string())
}
