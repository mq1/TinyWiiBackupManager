// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game, game_id::GameID, util::make_game_dir_name};
use anyhow::Result;
use std::{
    ffi::OsStr,
    fs::{self, File},
    path::Path,
};

fn adopt_orphaned_discs(games_dir: &Path, is_wii: bool) -> Result<()> {
    let all_discs = fs::read_dir(games_dir)?
        .filter_map(|entry| {
            let path = entry.ok()?.path();

            let filename = path.file_name()?.to_str()?;
            if filename.starts_with(".") {
                return None;
            }

            let ext = path.extension()?.to_str()?;
            if !matches!(ext, "iso" | "wbfs" | "ciso") {
                return None;
            }

            let mut f = File::open(&path).ok()?;
            let meta = wii_disc_info::Meta::read(&mut f).ok()?;

            Some((path, meta))
        })
        .collect::<Vec<_>>();

    for (path, meta) in all_discs {
        let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        let Some(game_id) = GameID::new(meta.game_id()) else {
            continue;
        };

        let ext = match meta.format() {
            wii_disc_info::Format::Iso => "iso",
            wii_disc_info::Format::Wbfs => "wbfs",
            wii_disc_info::Format::Ciso => "ciso",
            _ => continue,
        };

        let new_parent_name = make_game_dir_name(game_id, meta.game_title());

        let new_filename = if filename.ends_with(".part0.iso") {
            format!("{game_id}.part0.iso")
        } else if is_wii {
            format!("{game_id}.{ext}")
        } else {
            match meta.disc_number() {
                0 => format!("game.{ext}"),
                n => format!("disc{}.{ext}", n + 1),
            }
        };

        let new_parent = games_dir.join(new_parent_name);
        let new_path = new_parent.join(&new_filename);

        fs::create_dir_all(&new_parent)?;
        fs::rename(&path, &new_path)?;

        // handle split files
        if meta.format() == wii_disc_info::Format::Wbfs {
            let wbf1_path = path.with_extension("wbf1");
            if wbf1_path.exists() {
                let new_wbf1_path = new_path.with_extension("wbf1");
                fs::rename(wbf1_path, new_wbf1_path)?;
            }
            let wbf2_path = path.with_extension("wbf2");
            if wbf2_path.exists() {
                let new_wbf2_path = new_path.with_extension("wbf2");
                fs::rename(wbf2_path, new_wbf2_path)?;
            }
            let wbf3_path = path.with_extension("wbf3");
            if wbf3_path.exists() {
                let new_wbf3_path = new_path.with_extension("wbf3");
                fs::rename(wbf3_path, new_wbf3_path)?;
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

fn readopt_parented_discs(games_dir: &Path) -> Result<()> {
    let all_games = game::scan_dir(games_dir);

    for game in all_games {
        let Some(disc_path) = game.get_disc_path() else {
            continue;
        };

        // fix for an eventual wrong extension
        {
            let Ok(mut f) = File::open(&disc_path) else {
                continue;
            };
            let Ok(meta) = wii_disc_info::Meta::read(&mut f) else {
                continue;
            };
            let Some(ext) = disc_path.extension().and_then(OsStr::to_str) else {
                continue;
            };

            let lowercase = meta.format().to_string().to_ascii_lowercase();
            if ext != lowercase {
                let new_path = disc_path.with_extension(lowercase);
                fs::rename(&disc_path, &new_path)?;
            }
        }

        let new_filename = make_game_dir_name(game.id, &game.title);
        let new_path = games_dir.join(new_filename);

        if new_path.exists() {
            continue;
        }

        fs::rename(&game.path, &new_path)?;
    }

    Ok(())
}

pub fn perform(root_path: &Path) -> Result<()> {
    let wii_dir = root_path.join("wbfs");
    let gc_dir = root_path.join("games");

    fs::create_dir_all(&wii_dir)?;
    fs::create_dir_all(&gc_dir)?;

    adopt_orphaned_discs(&wii_dir, true)?;
    adopt_orphaned_discs(&gc_dir, false)?;

    readopt_parented_discs(&wii_dir)?;
    readopt_parented_discs(&gc_dir)?;

    Ok(())
}
