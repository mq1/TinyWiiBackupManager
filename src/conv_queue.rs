// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConfigContents, DriveInfo, GameList, QueuedConversion, State,
    convert::{Conversion, ConversionFlags},
    disc_util,
};
use slint::{Model, SharedString, ToSharedString, Weak};
use std::{fs::File, path::PathBuf};

impl QueuedConversion {
    pub fn run(&self, weak: Weak<State<'static>>) {
        let mut conv = match <Conversion>::try_from(self) {
            Ok(conv) => conv,
            Err(e) => {
                weak.upgrade()
                    .unwrap()
                    .invoke_notify_err(e.to_shared_string());

                return;
            }
        };

        weak.upgrade().unwrap().set_is_converting(true);

        let _ = std::thread::spawn(move || {
            let res = conv.perform(&weak);

            let _ = weak.upgrade_in_event_loop(|state| {
                state.set_is_converting(false);
                state.set_status(SharedString::new());

                if let Err(e) = res {
                    state.invoke_notify_err(e.to_shared_string());
                } else {
                    state.invoke_start_conversion();
                }
            });
        });
    }

    #[must_use]
    pub fn display_string(&self) -> String {
        format!("• {}", &self.in_path)
    }

    #[must_use]
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

        let mut queue = Vec::new();
        for (path, _, _, _) in entries {
            let mut flags = ConversionFlags::IS_FOR_DRIVE;
            flags.set(ConversionFlags::IS_FAT32, drive_info.fs_kind == "FAT32");
            flags.set(ConversionFlags::REMOVE_SOURCES, conf.remove_sources_games);
            flags.set(ConversionFlags::SCRUB_UPDATE, conf.scrub_update_partition);
            flags.set(ConversionFlags::ALWAYS_SPLIT, conf.always_split);

            queue.push(QueuedConversion {
                in_path: path.to_string_lossy().to_shared_string(),
                out_path: conf.mount_point.clone(),
                wii_output_format: conf.wii_output_format.clone(),
                gc_output_format: conf.gc_output_format.clone(),
                flags: flags.bits(),
            });
        }

        queue
    }
}
