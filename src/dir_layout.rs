// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    disc_info::DiscInfo,
    overflow_writer::get_overflow_path,
    util::{get_files_and_dirs, sanitize},
};
use anyhow::Result;
use nod::common::Format;
use std::{fs, path::Path};

fn normalize_games_dir(base_dir: &Path) -> Result<()> {
    if !base_dir.exists() {
        return Ok(());
    }

    let (files, dirs) = get_files_and_dirs(base_dir);

    let orphaned_discs = files
        .into_iter()
        .map(DiscInfo::from_path)
        .filter_map(Result::ok);

    let non_orphaned_discs = dirs
        .into_iter()
        .map(|p| DiscInfo::from_game_dir(&p))
        .filter_map(Result::ok);

    for disc_info in orphaned_discs {
        let game_id = disc_info.id.as_str();
        let game_title = sanitize(&disc_info.title);

        let new_disc_name = match disc_info.format {
            Format::Wbfs => format!("{}.wbfs", game_id),
            Format::Ciso => match disc_info.disc_num {
                0 => "game.ciso".to_string(),
                n => format!("disc{}.ciso", n + 1),
            },
            Format::Iso => match disc_info.is_wii {
                true => format!("{}.iso", game_id),
                false => match disc_info.disc_num {
                    0 => "game.iso".to_string(),
                    n => format!("disc{}.iso", n + 1),
                },
            },
            _ => continue,
        };

        let new_game_dir = base_dir.join(format!("{} [{}]", game_title, game_id));
        if new_game_dir.exists() {
            continue;
        }
        fs::create_dir_all(&new_game_dir)?;

        let new_disc_path = new_game_dir.join(new_disc_name);
        fs::rename(&disc_info.disc_path, &new_disc_path)?;

        if let Some(overflow_path) = get_overflow_path(&disc_info.disc_path)
            && overflow_path.exists()
            && let Some(new_overflow_path) = get_overflow_path(&new_disc_path)
        {
            fs::rename(overflow_path, new_overflow_path)?;
        }
    }

    for disc_info in non_orphaned_discs {
        let game_id = disc_info.id.as_str();
        let game_title = sanitize(&disc_info.title);

        let new_disc_name = match disc_info.format {
            Format::Wbfs => format!("{}.wbfs", game_id),
            Format::Ciso => match disc_info.disc_num {
                0 => "game.ciso".to_string(),
                n => format!("disc{}.ciso", n + 1),
            },
            Format::Iso => match disc_info.is_wii {
                true => format!("{}.iso", game_id),
                false => match disc_info.disc_num {
                    0 => "game.iso".to_string(),
                    n => format!("disc{}.iso", n + 1),
                },
            },
            _ => continue,
        };
        let new_disc_path = disc_info.game_dir.join(new_disc_name);

        if disc_info.disc_path != new_disc_path {
            fs::rename(&disc_info.disc_path, &new_disc_path)?;
        }

        if let Some(overflow_path) = get_overflow_path(&disc_info.disc_path)
            && overflow_path.exists()
            && let Some(new_overflow_path) = get_overflow_path(&new_disc_path)
            && overflow_path != new_overflow_path
        {
            fs::rename(overflow_path, new_overflow_path)?;
        }

        let new_game_dir = base_dir.join(format!("{} [{}]", game_title, game_id));
        if new_game_dir.exists() {
            continue;
        }

        fs::rename(&disc_info.game_dir, new_game_dir)?;
    }

    Ok(())
}

pub fn normalize_paths(mount_point: &Path) -> Result<()> {
    normalize_games_dir(&mount_point.join("wbfs"))?;
    normalize_games_dir(&mount_point.join("games"))?;

    Ok(())
}
