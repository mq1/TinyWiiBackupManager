// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::import::make_game_dir_name,
    toolbox::{ToolboxGroup, ToolboxItem},
};
use anyhow::{Result, bail};
use lucide_icons::Icon;
use smol::{
    fs::{self, DirEntry, File},
    stream::{self, Stream, StreamExt},
};
use std::{
    convert::identity,
    ffi::OsStr,
    path::{Path, PathBuf},
};

async fn scan_dir_for_discs(
    path: &Path,
) -> Result<impl Stream<Item = (PathBuf, wii_disc_info::Meta)>> {
    async fn try_open_entry(
        entry: Result<DirEntry, std::io::Error>,
    ) -> Result<(PathBuf, wii_disc_info::Meta)> {
        let path = entry?.path();
        let mut f = File::open(&path).await?;
        let meta = wii_disc_info::Meta::read_async(&mut f).await?;
        Ok((path, meta))
    }

    fs::read_dir(path)
        .await
        .map_err(anyhow::Error::from)
        .map(|entries| entries.then(try_open_entry).filter_map(Result::ok))
}

fn make_disc_filename(path: &Path, meta: &wii_disc_info::Meta) -> String {
    let ext = match meta.format() {
        wii_disc_info::Format::Iso => "iso",
        wii_disc_info::Format::Wbfs => "wbfs",
        wii_disc_info::Format::Ciso => "ciso",
        _ => unreachable!(),
    };

    if path.ends_with(".part0.iso") || path.ends_with(".PART0.ISO") {
        format!("{}.part0.iso", meta.game_id())
    } else if meta.is_wii() {
        format!("{}.{ext}", meta.game_id())
    } else {
        match meta.disc_number() {
            0 => format!("game.{ext}"),
            n => format!("disc{}.{ext}", n + 1),
        }
    }
}

async fn rename_split_files(
    disc_path: &Path,
    meta: &wii_disc_info::Meta,
    new_disc_path: &Path,
    games_dir: &Path,
) -> Result<()> {
    if meta.format() == wii_disc_info::Format::Wbfs {
        let wbf1_path = disc_path.with_extension("wbf1");
        if wbf1_path.exists() {
            let new_wbf1_path = new_disc_path.with_extension("wbf1");
            fs::rename(wbf1_path, new_wbf1_path).await?;
        }
        let wbf2_path = disc_path.with_extension("wbf2");
        if wbf2_path.exists() {
            let new_wbf2_path = new_disc_path.with_extension("wbf2");
            fs::rename(wbf2_path, new_wbf2_path).await?;
        }
        let wbf3_path = disc_path.with_extension("wbf3");
        if wbf3_path.exists() {
            let new_wbf3_path = new_disc_path.with_extension("wbf3");
            fs::rename(wbf3_path, new_wbf3_path).await?;
        }
    } else if let Some(filename) = disc_path.file_name().and_then(OsStr::to_str).and_then(|s| {
        s.strip_suffix(".part0.iso")
            .or_else(|| s.strip_suffix(".PART0.ISO"))
    }) {
        let part1_orig = games_dir.join(format!("{filename}.part1.iso"));
        let part1_orig_uppercase = games_dir.join(format!("{filename}.PART1.ISO"));
        let part1_new = new_disc_path
            .with_extension("")
            .with_extension("part1")
            .with_added_extension("iso");

        if part1_orig.exists() {
            fs::rename(part1_orig, part1_new).await?;
        } else if part1_orig_uppercase.exists() {
            fs::rename(part1_orig_uppercase, part1_new).await?;
        }
    }

    Ok(())
}

async fn adopt_orphaned_discs(games_dir: &Path) -> Result<()> {
    let entries = scan_dir_for_discs(games_dir).await?;

    let results = entries
        .then(|(path, meta)| async move {
            let new_parent_name = make_game_dir_name(meta.game_id(), meta.game_title());
            let new_disc_filename = make_disc_filename(&path, &meta);

            let new_disc_parent = games_dir.join(new_parent_name);
            let new_path = new_disc_parent.join(&new_disc_filename);

            fs::create_dir_all(&new_disc_parent).await?;
            fs::rename(&path, &new_path).await?;

            // handle split files
            rename_split_files(&path, &meta, &new_path, games_dir).await
        })
        .collect::<Vec<_>>()
        .await;

    let errors = results
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();

    match errors.is_empty() {
        true => Ok(()),
        false => bail!("{errors:?}"),
    }
}

async fn readopt_parented_discs(games_dir: &Path) -> Result<()> {
    let all_dirs = fs::read_dir(games_dir)
        .await?
        .filter_map(Result::ok)
        .then(|entry| async move {
            entry
                .file_type()
                .await
                .is_ok_and(|ft| ft.is_dir())
                .then_some(entry.path())
        })
        .filter_map(identity);

    let results = all_dirs
        .then(|path| async move {
            let discs = scan_dir_for_discs(&path).await?.collect::<Vec<_>>().await;

            match &discs[..] {
                [(disc_path, meta)] => {
                    // rename disc
                    let new_disc_filename = make_disc_filename(disc_path, meta);
                    let new_disc_path = path.join(&new_disc_filename);
                    fs::rename(&disc_path, &new_disc_path).await?;

                    // rename eventual split files
                    rename_split_files(disc_path, meta, &new_disc_path, games_dir).await?;
                }
                [(disc0_path, _), (disc1_path, _)] => {
                    let stem0 = disc0_path.file_stem().unwrap_or_default().to_string_lossy();
                    let stem1 = disc1_path.file_stem().unwrap_or_default().to_string_lossy();

                    if !((stem0 == "game" && stem1.starts_with("disc"))
                        || (stem1 == "game" && stem0.starts_with("disc")))
                    {
                        // ignore unknown layouts
                        return Ok(());
                    }

                    // rename discs
                    for (disc_path, meta) in &discs {
                        let new_disc_filename = make_disc_filename(disc_path, meta);
                        let new_disc_path = path.join(&new_disc_filename);
                        fs::rename(&disc_path, &new_disc_path).await?;
                    }
                }
                _ => {
                    // ignore unknown layouts
                    return Ok(());
                }
            }

            let (_, meta) = &discs[0];
            let new_dir_name = make_game_dir_name(meta.game_id(), meta.game_title());
            let new_path = path.with_file_name(new_dir_name);

            if !new_path.exists() {
                fs::rename(path, &new_path).await?;
            }

            Ok::<_, anyhow::Error>(())
        })
        .collect::<Vec<_>>()
        .await;

    let errors = results
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();

    match errors.is_empty() {
        true => Ok(()),
        false => bail!("{errors:?}"),
    }
}

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Cleanup",
        icon: Icon::BrushCleaning,
        items: &[ToolboxItem {
            label: "Normalize paths (makes the game directories' layouts consistent)",
            run_fn: |ctx| {
                Box::pin(async move {
                    stream::iter([
                        ctx.config.mount_point().join("wbfs"),
                        ctx.config.mount_point().join("games"),
                    ])
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
