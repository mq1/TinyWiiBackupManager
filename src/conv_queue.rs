// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConfigContents, DriveInfo, Game, QueuedConversion, State,
    convert::{Conversion, ConversionFlags},
};
use anyhow::Result;
use slint::{SharedString, ToSharedString, Weak};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

impl QueuedConversion {
    pub fn run(&self, weak: Weak<State<'static>>) {
        let mut conv = match <Conversion>::try_from(self) {
            Ok(conv) => conv,
            Err(e) => {
                weak.upgrade().unwrap().push_notification(e.into());
                return;
            }
        };

        weak.upgrade().unwrap().set_is_converting(true);

        let _ = std::thread::spawn(move || {
            let res = conv.perform(&weak);

            let _ = weak.upgrade_in_event_loop(|state| {
                state.set_is_converting(false);
                state.set_status(SharedString::new());
                state.invoke_refresh_all();

                if let Err(e) = res {
                    state.push_notification(e.into());
                } else {
                    state.start_conversion();
                }
            });
        });
    }

    #[must_use]
    pub fn make_queue(
        paths: Vec<PathBuf>,
        existing_ids: &[SharedString],
        conf: &ConfigContents,
        drive_info: &DriveInfo,
    ) -> Vec<QueuedConversion> {
        // parse discs
        let mut entries = paths
            .into_iter()
            .filter_map(|p| {
                let mut f = File::open(&p).ok()?;
                let meta = wii_disc_info::Meta::read(&mut f).ok()?;
                Some((p, meta))
            })
            .collect::<Vec<_>>();

        // keep only new games
        entries.retain(|(_, meta)| existing_ids.iter().all(|id| id != meta.game_id()));

        let mut queue = Vec::new();
        for (path, meta) in entries {
            let mut flags = ConversionFlags::IS_FOR_DRIVE;
            flags.set(ConversionFlags::IS_FAT32, drive_info.fs_kind == "FAT32");
            flags.set(ConversionFlags::REMOVE_SOURCES, conf.remove_sources_games);
            flags.set(ConversionFlags::SCRUB_UPDATE, conf.scrub_update_partition);
            flags.set(ConversionFlags::ALWAYS_SPLIT, conf.always_split);

            queue.push(QueuedConversion {
                game_title: meta.game_title().to_shared_string(),
                game_id: meta.game_id().to_shared_string(),
                in_path: path.to_string_lossy().to_shared_string(),
                out_path: conf.mount_point.clone(),
                wii_output_format: conf.wii_output_format.clone(),
                gc_output_format: conf.gc_output_format.clone(),
                flags: flags.bits(),
            });
        }

        queue
    }

    pub fn new_archive(game: &Game, out_path: &Path) -> Result<Self> {
        let conv = QueuedConversion {
            game_title: game.title.clone(),
            game_id: game.id.clone(),
            in_path: game.get_disc_path()?.to_string_lossy().to_shared_string(),
            out_path: out_path.to_string_lossy().to_shared_string(),
            wii_output_format: "iso".to_shared_string(),
            gc_output_format: "iso".to_shared_string(),
            flags: 0,
        };

        Ok(conv)
    }
}
