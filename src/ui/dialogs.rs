// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{game::Game, import::sanitize_title, keep_valid_games},
    message::Message,
    util::fs::recursive_file_scan,
};
use iced::Task;
use smol::stream;
use std::{ffi::OsStr, path::PathBuf};
use wii_disc_info::game_id::GameID;

#[rustfmt::skip]
const GAME_EXTS_WITH_ZIP: &[&str] = &[
    "iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc", "zip",
    "ISO", "GCM", "WIA", "RVZ", "WBFS", "CISO", "GZC", "TGC", "ZIP",
];

#[rustfmt::skip]
const GAME_EXTS: &[&str] = &[
    "iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc",
    "ISO", "GCM", "WIA", "RVZ", "WBFS", "CISO", "GZC", "TGC",
];

#[rustfmt::skip]
const LOWERCASE_GAME_EXTS: &[&str] = &[
    "iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc",
];

#[rustfmt::skip]
const WIILOAD_EXTS: &[&str] = &[
    "zip", "dol", "elf",
    "ZIP", "DOL", "ELF",
];

#[cfg(feature = "dialogs-rfd")]
type Base = rfd::AsyncFileDialog;

#[cfg(feature = "dialogs-tfd")]
type Base = ();

#[cfg(feature = "dialogs-rfd")]
pub fn init_file_dialog_task() -> Task<Base> {
    iced::window::oldest()
        .and_then(|id| iced::window::run(id, |w| rfd::AsyncFileDialog::new().set_parent(w)))
}

#[cfg(feature = "dialogs-tfd")]
pub fn init_file_dialog_task() -> Task<Base> {
    Task::done(())
}

#[cfg(feature = "dialogs-rfd")]
async fn pick_folder(base: Base, title: &'static str) -> Option<PathBuf> {
    base.set_title(title).pick_folder().await.map(PathBuf::from)
}

#[cfg(feature = "dialogs-tfd")]
async fn pick_folder(_base: Base, title: &'static str) -> Option<PathBuf> {
    smol::unblock(|| tinyfiledialogs::select_folder_dialog(title, ""))
        .await
        .map(PathBuf::from)
}

#[cfg(feature = "dialogs-rfd")]
async fn pick_file(
    base: Base,
    title: &'static str,
    filter: (&'static [&str], &'static str),
) -> Option<PathBuf> {
    base.set_title(title)
        .add_filter(filter.1, filter.0)
        .pick_file()
        .await
        .map(PathBuf::from)
}

#[cfg(feature = "dialogs-tfd")]
async fn pick_file(
    _base: (),
    title: &'static str,
    filter: (&'static [&str], &'static str),
) -> Option<PathBuf> {
    smol::unblock(move || tinyfiledialogs::open_file_dialog(title, "", Some(filter)))
        .await
        .map(PathBuf::from)
}

#[cfg(feature = "dialogs-rfd")]
async fn pick_files(
    base: Base,
    title: &'static str,
    filter: (&'static [&str], &'static str),
) -> Vec<PathBuf> {
    base.set_title(title)
        .add_filter(filter.1, filter.0)
        .pick_files()
        .await
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect()
}

#[cfg(feature = "dialogs-tfd")]
async fn pick_files(
    _base: Base,
    title: &'static str,
    filter: (&'static [&str], &'static str),
) -> Vec<PathBuf> {
    smol::unblock(move || tinyfiledialogs::open_file_dialog_multi(title, "", Some(filter)))
        .await
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect()
}

#[cfg(feature = "dialogs-rfd")]
async fn save_file(
    base: Base,
    title: &'static str,
    filename: String,
    filter: (&'static [&str], &'static str),
) -> Option<PathBuf> {
    base.set_title(title)
        .set_file_name(filename)
        .add_filter(filter.1, filter.0)
        .save_file()
        .await
        .map(PathBuf::from)
}

#[cfg(feature = "dialogs-tfd")]
async fn save_file(
    _base: Base,
    title: &'static str,
    filename: String,
    filter: (&'static [&str], &'static str),
) -> Option<PathBuf> {
    smol::unblock(move || {
        tinyfiledialogs::save_file_dialog_with_filter(title, &filename, filter.0, filter.1)
    })
    .await
    .map(PathBuf::from)
}

pub fn make_pick_mount_point_dialog_task(base: Base) -> Task<Message> {
    Task::perform(
        pick_folder(base, "Select Drive/Mount Point"),
        |res| match res {
            Some(path) => Message::MountPointPicked(path),
            None => Message::NoOp,
        },
    )
}

pub fn make_pick_homebrew_apps_dialog_task(base: Base) -> Task<Message> {
    Task::perform(
        pick_files(
            base,
            "Select Homebrew App(s) to import",
            (&["zip", "ZIP"], "Homebrew app"),
        ),
        |res| match res {
            paths if !paths.is_empty() => Message::ImportHomebrewApps(paths),
            _ => Message::NoOp,
        },
    )
}

pub fn make_pick_in_out_dialogs_task(base: Base) -> Task<Message> {
    Task::perform(
        async move {
            let in_path = pick_file(
                base.clone(),
                "Select Game to convert",
                (GAME_EXTS, "Wii/NGC rom"),
            )
            .await?;

            let file_stem = in_path.file_stem().and_then(OsStr::to_str)?;
            let rvz_filename = format!("{file_stem}.rvz");

            let out_path = save_file(
                base,
                "Save converted game to",
                rvz_filename,
                (LOWERCASE_GAME_EXTS, "Wii/NGC rom"),
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

pub fn make_pick_games_dialog_task(base: Base, existing_ids: Box<[GameID]>) -> Task<Message> {
    Task::perform(
        async move {
            let paths = pick_files(
                base,
                "Select Game(s) to import",
                (GAME_EXTS_WITH_ZIP, "Wii/NGC rom"),
            )
            .await;

            keep_valid_games(stream::iter(paths), &existing_ids).await
        },
        Message::PickedGames,
    )
}

pub fn make_pick_games_recursively_dialog_task(
    base: Base,
    existing_ids: Box<[GameID]>,
) -> Task<Message> {
    Task::perform(
        async move {
            let res = pick_folder(base, "Select a directory containing game(s) to import").await;

            if let Some(path) = res {
                let games = recursive_file_scan(path, GAME_EXTS_WITH_ZIP);

                keep_valid_games(games, &existing_ids).await
            } else {
                Vec::new()
            }
        },
        Message::PickedGames,
    )
}

pub fn make_pick_export_game_dest_dialog_task(base: Base, game: Game) -> Task<Message> {
    let ascii_title = twbm_idmap::get_ascii_title(game.id()).unwrap_or(game.title());
    let filename = sanitize_title(ascii_title) + ".rvz";

    Task::perform(
        save_file(
            base,
            "Select where you want to export the game",
            filename,
            (LOWERCASE_GAME_EXTS, "Wii/NGC rom"),
        ),
        move |opt| match opt {
            Some(path) => Message::ExportGame(game, path),
            None => Message::NoOp,
        },
    )
}

pub fn make_pick_file_to_wiiload_dialog_task(base: Base) -> Task<Message> {
    Task::perform(
        pick_file(
            base,
            "Select a file to send via wiiload",
            (WIILOAD_EXTS, "Homebrew app"),
        ),
        |opt| match opt {
            Some(path) => Message::SendViaWiiload(path),
            None => Message::NoOp,
        },
    )
}
