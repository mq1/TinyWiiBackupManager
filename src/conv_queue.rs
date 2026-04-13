// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConversionKind, QueuedConversion, QueuedStandardConversion, State, convert::Conversion,
};
use slint::{SharedString, ToSharedString, Weak};
use std::{ffi::OsStr, fs::File, path::PathBuf};
use zip::ZipArchive;

impl QueuedConversion {
    pub fn run(&self, weak: Weak<State<'static>>) {
        let (conf, drive_info) = {
            let state = weak.upgrade().unwrap();
            state.set_is_converting(true);
            (state.get_config().contents, state.get_drive_info())
        };

        let mut conv = Conversion::new(self, &conf, &drive_info);

        let _ = std::thread::spawn(move || {
            let res = conv.perform(&weak);

            let _ = weak.upgrade_in_event_loop(move |state| {
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
    pub fn make_queue(paths: Vec<PathBuf>, existing_ids: &[SharedString]) -> Vec<Self> {
        // parse discs
        let mut entries = paths
            .into_iter()
            .filter_map(|p| {
                let mut f = File::open(&p).ok()?;

                let meta = if p
                    .extension()
                    .and_then(OsStr::to_str)
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
                {
                    let mut archive = ZipArchive::new(&mut f).ok()?;
                    let mut disc = archive.by_index(0).ok()?;
                    wii_disc_info::Meta::read(&mut disc).ok()?
                } else {
                    wii_disc_info::Meta::read(&mut f).ok()?
                };

                Some((p, meta))
            })
            .collect::<Vec<_>>();

        // keep only new games
        entries.retain(|(_, meta)| existing_ids.iter().all(|id| id != meta.game_id()));

        let mut queue = Vec::new();
        for (path, meta) in entries {
            let queued = QueuedStandardConversion {
                game_title: meta.game_title().to_shared_string(),
                game_id: meta.game_id().to_shared_string(),
                in_path: path.to_string_lossy().to_shared_string(),
                is_wii: meta.is_wii(),
                disc_number: i32::from(meta.disc_number()),
            };

            let queued = QueuedConversion {
                kind: ConversionKind::Standard,
                standard: queued,
                ..Default::default()
            };

            queue.push(queued);
        }

        queue
    }
}
