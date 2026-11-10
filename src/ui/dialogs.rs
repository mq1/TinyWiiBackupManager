// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{game::Game, import::sanitize_title, keep_valid_games},
    message::Message,
    util::fs::recursive_file_scan,
};
use iced::Task;
use rfd::AsyncFileDialog;
use smol::stream;
use std::{ffi::OsStr, path::PathBuf};
use wii_disc_info::game_id::GameID;

#[rustfmt::skip]
const IN_GAME_FILTER_WITH_ZIP: (&[&str], &str) = (
    &[
        "iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc", "zip",
        "ISO", "GCM", "WIA", "RVZ", "WBFS", "CISO", "GZC", "TGC", "ZIP",
    ],
    "Wii/NGC rom",
);

#[rustfmt::skip]
const IN_GAME_FILTER: (&[&str], &str) = (
    &[
        "iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc",
        "ISO", "GCM", "WIA", "RVZ", "WBFS", "CISO", "GZC", "TGC",
    ],
    "Wii/NGC rom"
);

#[rustfmt::skip]
const OUT_GAME_FILTERS: &[(&[&str], &str)] = &[
    (&["rvz"], "RVZ"),
    (&["iso", "gcm"], "ISO"),
    (&["wia"], "WIA"),
    (&["wbfs"], "WBFS"),
    (&["ciso"], "CISO"),
    (&["gcz"], "GCZ"),
    (&["tgc"], "TGC"),
];

#[cfg(feature = "win-xp")]
#[rustfmt::skip]
const OUT_GAME_FILTER: (&[&str], &str) = (
    &[
        "rvz", "iso", "gcm", "wia", "wbfs", "ciso", "gcz", "tgc",
    ],
    "Wii/NGC rom"
);

#[rustfmt::skip]
const IN_APP_FILTER: (&[&str], &str) = (
    &[
        "zip",
        "ZIP"
    ],
    "Homebrew app"
);

#[rustfmt::skip]
const IN_WIILOAD_FILTER: (&[&str], &str) = (
    &[
        "zip", "dol", "elf",
        "ZIP", "DOL", "ELF",
    ],
    "Homebrew app"
);

#[cfg(feature = "win-xp")]
static CAN_USE_RFD: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| winsafe::IsWindowsVistaOrGreater().unwrap_or(false));

pub fn init_file_dialog_task() -> Task<AsyncFileDialog> {
    iced::window::oldest()
        .and_then(|id| iced::window::run(id, |w| rfd::AsyncFileDialog::new().set_parent(w)))
}

async fn pick_folder(base: AsyncFileDialog, title: &'static str) -> Option<PathBuf> {
    #[cfg(feature = "win-xp")]
    if !*CAN_USE_RFD {
        return smol::unblock(|| tinyfiledialogs::select_folder_dialog(title, ""))
            .await
            .map(PathBuf::from);
    }

    base.set_title(title).pick_folder().await.map(PathBuf::from)
}

async fn pick_file(
    base: AsyncFileDialog,
    title: &'static str,
    filter: (&'static [&'static str], &'static str),
) -> Option<PathBuf> {
    #[cfg(feature = "win-xp")]
    if !*CAN_USE_RFD {
        return smol::unblock(move || tinyfiledialogs::open_file_dialog(title, "", Some(filter)))
            .await
            .map(PathBuf::from);
    }

    base.set_title(title)
        .add_filter(filter.1, filter.0)
        .pick_file()
        .await
        .map(PathBuf::from)
}

async fn pick_files(
    base: AsyncFileDialog,
    title: &'static str,
    filter: (&'static [&'static str], &'static str),
) -> Vec<PathBuf> {
    #[cfg(feature = "win-xp")]
    if !*CAN_USE_RFD {
        return smol::unblock(move || {
            tinyfiledialogs::open_file_dialog_multi(title, "", Some(filter))
        })
        .await
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect();
    }

    base.set_title(title)
        .add_filter(filter.1, filter.0)
        .pick_files()
        .await
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect()
}

async fn save_file(
    base: AsyncFileDialog,
    title: &'static str,
    filename: String,
    filters: &'static [(&'static [&'static str], &'static str)],
    #[cfg(feature = "win-xp")] tfd_filter: (&'static [&'static str], &'static str),
) -> Option<PathBuf> {
    #[cfg(feature = "win-xp")]
    if !*CAN_USE_RFD {
        return smol::unblock(move || {
            tinyfiledialogs::save_file_dialog_with_filter(
                title,
                &filename,
                tfd_filter.0,
                tfd_filter.1,
            )
        })
        .await
        .map(PathBuf::from);
    }

    filters
        .iter()
        .fold(base, |d, &(exts, name)| d.add_filter(name, exts))
        .set_title(title)
        .set_file_name(filename)
        .save_file()
        .await
        .map(PathBuf::from)
}

pub fn make_pick_mount_point_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        pick_folder(base, "Select Drive/Mount Point"),
        |res| match res {
            Some(path) => Message::MountPointPicked(path),
            None => Message::NoOp,
        },
    )
}

pub fn make_pick_homebrew_apps_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        pick_files(base, "Select Homebrew App(s) to import", IN_APP_FILTER),
        |res| match res {
            paths if !paths.is_empty() => Message::ImportHomebrewApps(paths),
            _ => Message::NoOp,
        },
    )
}

pub fn make_pick_in_out_dialogs_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        async move {
            let in_path = pick_file(base.clone(), "Select Game to convert", IN_GAME_FILTER).await?;

            let file_stem = in_path.file_stem().and_then(OsStr::to_str)?;
            let rvz_filename = format!("{file_stem}.rvz");

            let out_path = save_file(
                base,
                "Save converted game to",
                rvz_filename,
                OUT_GAME_FILTERS,
                #[cfg(feature = "win-xp")]
                OUT_GAME_FILTER,
            )
            .await?;

            Some((in_path, out_path))
        },
        |opt| match opt {
            Some((in_path, out_path)) => Message::ConvertGame(in_path, out_path),
            None => Message::NoOp,
        },
    )
}

pub fn make_pick_games_dialog_task(
    base: AsyncFileDialog,
    existing_ids: Box<[GameID]>,
) -> Task<Message> {
    Task::perform(
        async move {
            let paths = pick_files(base, "Select Game(s) to import", IN_GAME_FILTER_WITH_ZIP).await;

            keep_valid_games(stream::iter(paths), &existing_ids).await
        },
        Message::PickedGames,
    )
}

pub fn make_pick_games_recursively_dialog_task(
    base: AsyncFileDialog,
    existing_ids: Box<[GameID]>,
) -> Task<Message> {
    Task::perform(
        async move {
            let res = pick_folder(base, "Select a directory containing game(s) to import").await;

            if let Some(path) = res {
                let games = recursive_file_scan(path, IN_GAME_FILTER_WITH_ZIP.0);

                keep_valid_games(games, &existing_ids).await
            } else {
                Vec::new()
            }
        },
        Message::PickedGames,
    )
}

pub fn make_pick_export_game_dest_dialog_task(base: AsyncFileDialog, game: Game) -> Task<Message> {
    let ascii_title = twbm_idmap::get_ascii_title(game.id()).unwrap_or(game.title());
    let filename = sanitize_title(ascii_title) + ".rvz";

    Task::perform(
        save_file(
            base,
            "Select where you want to export the game",
            filename,
            OUT_GAME_FILTERS,
            #[cfg(feature = "win-xp")]
            OUT_GAME_FILTER,
        ),
        move |opt| match opt {
            Some(path) => Message::ExportGame(game, path),
            None => Message::NoOp,
        },
    )
}

pub fn make_pick_file_to_wiiload_dialog_task(base: AsyncFileDialog) -> Task<Message> {
    Task::perform(
        pick_file(base, "Select a file to send via wiiload", IN_WIILOAD_FILTER),
        |opt| match opt {
            Some(path) => Message::SendViaWiiload(path),
            None => Message::NoOp,
        },
    )
}
