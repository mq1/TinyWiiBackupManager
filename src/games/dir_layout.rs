// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{disc_info::DiscInfo, extensions::format_to_ext},
    util::sanitize,
};
use anyhow::Result;
use nod::common::Format;
use std::{ffi::OsStr, fs, path::Path};

fn adopt_orphaned_discs(mount_point: &Path) -> Result<()> {
    let all_files = fs::read_dir(mount_point.join("wbfs"))?
        .chain(fs::read_dir(mount_point.join("games"))?)
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()));

    let all_discs = all_files
        .map(|e| e.path())
        .map(DiscInfo::try_from_path)
        .filter_map(Result::ok);

    for disc in all_discs {
        let title = if disc.is_wii() {
            sanitize(disc.title())
        } else {
            sanitize(disc.title())
                .replace(" game disc 1", "")
                .replace(" game disc 2", "")
        };

        let new_parent_name = format!("{} [{}]", title, disc.id().as_str());

        let new_parent = if disc.is_wii() {
            mount_point.join("wbfs").join(new_parent_name)
        } else {
            mount_point.join("games").join(new_parent_name)
        };

        if new_parent.exists() {
            continue;
        }

        let Some(orig_filename) = disc.disc_path().file_name().and_then(OsStr::to_str) else {
            continue;
        };

        let new_filename = if orig_filename.ends_with(".part0.iso") {
            format!("{}.part0.iso", disc.id().as_str())
        } else if disc.is_wii() {
            format!("{}.{}", disc.id().as_str(), format_to_ext(disc.format()))
        } else {
            match disc.disc_num() {
                0 => format!("game.{}", format_to_ext(disc.format())),
                n => format!("disc{}.{}", n + 1, format_to_ext(disc.format())),
            }
        };

        let new_path = new_parent.join(&new_filename);

        fs::create_dir_all(&new_parent)?;
        fs::rename(disc.disc_path(), &new_path)?;

        // handle split files
        if disc.format() == Format::Wbfs {
            let wbf1_path = disc.disc_path().with_extension("wbf1");
            if wbf1_path.exists() {
                let new_wbf1_path = new_path.with_extension("wbf1");
                fs::rename(wbf1_path, new_wbf1_path)?;
            }
        }
        if orig_filename.ends_with(".part0.iso") {
            let part1_orig = disc
                .disc_path()
                .with_file_name(orig_filename.replace(".part0.iso", ".part1.iso"));
            if part1_orig.exists() {
                let part1_new =
                    new_path.with_file_name(new_filename.replace(".part0.iso", ".part1.iso"));
                fs::rename(part1_orig, part1_new)?;
            }
        }
    }

    Ok(())
}

fn readopt_parented_discs(mount_point: &Path) -> Result<()> {
    let all_dirs = fs::read_dir(mount_point.join("wbfs"))?
        .chain(fs::read_dir(mount_point.join("games"))?)
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()));

    let all_discs = all_dirs.filter_map(|e| {
        let path = e.path();
        let disc_info = DiscInfo::try_from_game_dir(&path).ok()?;
        Some((disc_info, path))
    });

    for (disc, parent) in all_discs {
        let title = if disc.is_wii() {
            sanitize(disc.title())
        } else {
            sanitize(disc.title())
                .replace(" game disc 1", "")
                .replace(" game disc 2", "")
        };

        let new_parent_name = format!("{} [{}]", title, disc.id().as_str());

        let new_parent = if disc.is_wii() {
            mount_point.join("wbfs").join(new_parent_name)
        } else {
            mount_point.join("games").join(new_parent_name)
        };

        if new_parent.exists() {
            continue;
        }

        fs::rename(parent, &new_parent)?;
    }

    Ok(())
}

pub fn normalize_paths(mount_point: &Path) -> Result<String> {
    fs::create_dir_all(mount_point.join("wbfs"))?;
    fs::create_dir_all(mount_point.join("games"))?;
    adopt_orphaned_discs(mount_point)?;
    readopt_parented_discs(mount_point)?;

    Ok("Paths successfully normalized".to_string())
}
