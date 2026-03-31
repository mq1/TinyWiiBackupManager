// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ConfigContents, DriveInfo, GameList, QueuedConversion, disc_util};
use slint::{Model, ToSharedString};
use std::{fs::File, path::PathBuf};

impl QueuedConversion {
    #[must_use] 
    pub fn display_string(&self) -> String {
        format!("• {}", &self.in_path)
    }
}

pub fn make_queue(
    paths: Vec<PathBuf>,
    game_list: &GameList,
    conf: &ConfigContents,
    drive_info: &DriveInfo,
) -> Vec<QueuedConversion> {
    // parse discs
    let mut entries = paths
        .into_iter()
        .filter_map(|p| {
            let mut f = File::open(&p).ok()?;
            let (format, id, title) = disc_util::read_disc_header(&mut f)?;
            Some((p, format, id, title))
        })
        .collect::<Vec<_>>();

    // keep only new games
    entries.retain(|(_, _, id, _)| game_list.games.iter().all(|g| g.id != *id));

    entries
        .into_iter()
        .map(|(p, _, _, _)| QueuedConversion {
            in_path: p.to_string_lossy().to_shared_string(),
            out_path: conf.mount_point.clone(),
            is_fat32: drive_info.fs_kind == "FAT32",
            always_split: conf.always_split,
            remove_sources: conf.remove_sources_games,
            scrub_update: conf.scrub_update_partition,
            wii_output_format: conf.wii_output_format.clone(),
            gc_output_format: conf.gc_output_format.clone(),
        })
        .collect()
}
