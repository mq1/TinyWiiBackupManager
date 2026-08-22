// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::keep_valid_games, messages::Message, util::fs::recursive_file_scan};
use iced::Task;
use rfd::AsyncFileDialog;
use smol::stream;
use std::path::PathBuf;

#[rustfmt::skip]
const GAME_EXTS: &[&str] = &[
    "iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc", "zip",
    "ISO", "GCM", "WIA", "RVZ", "WBFS", "CISO", "GZC", "TGC", "ZIP",
];

pub fn make_pick_mount_point_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        async move {
            base.set_title("Select Drive/Mount Point")
                .pick_folder()
                .await
                .map(Into::into)
        },
        Message::MountPointPicked,
    )
}

pub fn make_pick_homebrew_apps_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        async move {
            base.set_title("Select Homebrew App(s) to import")
                .pick_files()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect()
        },
        Message::ImportHomebrewApps,
    )
}

pub fn make_pick_games_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        async move {
            let res = base
                .set_title("Select Game(s) to import")
                .add_filter("Wii/NGC rom", GAME_EXTS)
                .pick_files()
                .await;

            if let Some(paths) = res
                && !paths.is_empty()
            {
                let it = paths.into_iter().map(PathBuf::from);
                keep_valid_games(stream::iter(it)).await
            } else {
                vec![]
            }
        },
        Message::ImportGames,
    )
}

pub fn make_pick_games_recursively_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        async move {
            let res = base
                .set_title("Select a directory containing game(s) to import")
                .pick_folder()
                .await;

            if let Some(path) = res {
                keep_valid_games(recursive_file_scan(path, GAME_EXTS)).await
            } else {
                vec![]
            }
        },
        Message::ImportGames,
    )
}
