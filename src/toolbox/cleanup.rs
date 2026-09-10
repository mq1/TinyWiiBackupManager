// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    games::{game_list, import::make_game_dir_name},
    toolbox::{ToolboxGroup, ToolboxItem},
};
use iced::Task;
use lucide_icons::Icon;
use smol::{
    fs::{self, DirEntry, File},
    stream::{self, Stream, StreamExt},
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

async fn scan_dir_for_discs(
    path: &Path,
) -> Result<impl Stream<Item = (PathBuf, wii_disc_info::Meta)>, Error> {
    async fn try_open_entry(
        entry: Result<DirEntry, std::io::Error>,
    ) -> Result<(PathBuf, wii_disc_info::Meta), Error> {
        let path = entry?.path();
        let mut f = File::open(&path).await?;
        let meta = wii_disc_info::Meta::read_async(&mut f).await?;
        Ok((path, meta))
    }

    fs::read_dir(path)
        .await
        .map_err(Error::from)
        .map(|entries| entries.then(try_open_entry).filter_map(Result::ok))
}

async fn adopt_orphaned_discs(games_dir: &Path) -> Result<(), Error> {
    let entries = scan_dir_for_discs(games_dir).await?;

    let _ = entries
        .then(|(path, meta)| async move {
            let filename = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or(Error::InvalidFilename)?;

            let ext = match meta.format() {
                wii_disc_info::Format::Iso => "iso",
                wii_disc_info::Format::Wbfs => "wbfs",
                wii_disc_info::Format::Ciso => "ciso",
                _ => {
                    return Err(Error::InvalidDiscFormat);
                }
            };

            let game_id = meta.game_id();

            let new_parent_name = make_game_dir_name(game_id, meta.game_title());

            let new_filename = if path.ends_with(".part0.iso") {
                format!("{game_id}.part0.iso")
            } else if meta.is_wii() {
                format!("{game_id}.{ext}")
            } else {
                match meta.disc_number() {
                    0 => format!("game.{ext}"),
                    n => format!("disc{}.{ext}", n + 1),
                }
            };

            let new_parent = games_dir.join(new_parent_name);
            let new_path = new_parent.join(&new_filename);

            fs::create_dir_all(&new_parent).await?;
            fs::rename(&path, &new_path).await?;

            // handle split files
            if meta.format() == wii_disc_info::Format::Wbfs {
                let wbf1_path = path.with_extension("wbf1");
                if wbf1_path.exists() {
                    let new_wbf1_path = new_path.with_extension("wbf1");
                    fs::rename(wbf1_path, new_wbf1_path).await?;
                }
                let wbf2_path = path.with_extension("wbf2");
                if wbf2_path.exists() {
                    let new_wbf2_path = new_path.with_extension("wbf2");
                    fs::rename(wbf2_path, new_wbf2_path).await?;
                }
                let wbf3_path = path.with_extension("wbf3");
                if wbf3_path.exists() {
                    let new_wbf3_path = new_path.with_extension("wbf3");
                    fs::rename(wbf3_path, new_wbf3_path).await?;
                }
            } else if filename.ends_with(".part0.iso") {
                let part1_orig = games_dir.join(filename.replace(".part0.iso", ".part1.iso"));
                if part1_orig.exists() {
                    let part1_new =
                        new_parent.join(new_filename.replace(".part0.iso", ".part1.iso"));
                    fs::rename(part1_orig, part1_new).await?;
                }
            }

            Ok(())
        })
        .collect::<Vec<_>>()
        .await;

    Ok(())
}

async fn readopt_parented_discs(games_dir: &Path) -> Result<(), Error> {
    // is_wii is irrelevant here
    let all_games = game_list::scan_dir(games_dir, true).await;

    let _ = all_games
        .then(|game| async move {
            let disc_path = game.get_disc_path().await.ok_or(Error::DiscNotFound)?;

            // fix for an eventual wrong extension
            {
                let mut f = File::open(&disc_path).await?;
                let meta = wii_disc_info::Meta::read_async(&mut f).await?;
                let ext = disc_path
                    .extension()
                    .and_then(OsStr::to_str)
                    .ok_or(Error::InvalidFilename)?;

                let lowercase = meta.format().to_string().to_ascii_lowercase();
                if ext != lowercase {
                    let new_path = disc_path.with_extension(lowercase);
                    fs::rename(&disc_path, &new_path).await?;
                }
            }

            let new_filename = make_game_dir_name(game.id, &game.title);
            let new_path = games_dir.join(new_filename);

            if !new_path.exists() {
                fs::rename(&game.path, &new_path).await?;
            }

            Ok::<_, Error>(())
        })
        .collect::<Vec<_>>()
        .await;

    Ok(())
}

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Cleanup",
        icon: Icon::BrushCleaning,
        items: &[ToolboxItem {
            label: "Normalize paths (makes the game directories' layouts consistent)",
            run: |ctx| {
                Task::future(async move {
                    stream::iter([ctx.mount_point.join("wbfs"), ctx.mount_point.join("games")])
                        .then(|path| async move {
                            if fs::metadata(&path).await.is_ok_and(|meta| meta.is_dir()) {
                                adopt_orphaned_discs(&path).await?;
                                readopt_parented_discs(&path).await?;
                            }

                            Ok(())
                        })
                        .collect::<Vec<_>>()
                        .await
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()
                        .map(|_| "Paths successfully normalized".to_string())
                })
            },
        }],
    }]
};
