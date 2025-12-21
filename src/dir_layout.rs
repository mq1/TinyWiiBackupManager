// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{disc_info::DiscInfo, overflow_writer::get_overflow_path, util::sanitize};
use anyhow::Result;
use nod::common::Format;
use std::{fs, path::Path};

pub fn normalize_games_dir(games_dir: &Path) -> Result<()> {
    fs::create_dir_all(games_dir)?;

    for entry in fs::read_dir(games_dir)?.filter_map(Result::ok) {
        let path = entry.path();

        if let Ok(disc_info) = DiscInfo::from_game_dir(&path) {
            let game_id = disc_info.id.as_str();

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
            let new_disc_path = path.join(new_disc_name);

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

            let new_game_dir =
                games_dir.join(format!("{} [{}]", sanitize(&disc_info.title), game_id));

            if new_game_dir.exists() {
                continue;
            }

            if path != new_game_dir {
                fs::rename(&path, new_game_dir)?;
            }
        } else if let Ok(disc_info) = DiscInfo::from_path(path) {
            let game_id = disc_info.id.as_str();

            let overflow_path = get_overflow_path(&disc_info.disc_path);

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

            let new_game_dir =
                games_dir.join(format!("{} [{}]", sanitize(&disc_info.title), game_id));

            if new_game_dir.exists() {
                continue;
            }

            let new_disc_path = new_game_dir.join(new_disc_name);
            let new_overflow_path = get_overflow_path(&new_disc_path);

            fs::create_dir_all(new_game_dir)?;
            if disc_info.disc_path != new_disc_path {
                fs::rename(&disc_info.disc_path, &new_disc_path)?;
            }
            if let Some(overflow_path) = overflow_path
                && overflow_path.exists()
                && let Some(new_overflow_path) = new_overflow_path
                && overflow_path != new_overflow_path
            {
                fs::rename(overflow_path, new_overflow_path)?;
            }
        }
    }

    Ok(())
}

pub fn normalize_paths(mount_point: &Path) -> Result<()> {
    normalize_games_dir(&mount_point.join("wbfs"))?;
    normalize_games_dir(&mount_point.join("games"))?;

    Ok(())
}
