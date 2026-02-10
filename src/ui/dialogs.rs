// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::{SUPPORTED_DISC_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS};
use crate::games::game::Game;
use crate::games::game_id::GameID;
use crate::games::util::maybe_path_to_entry;
use crate::hbc::osc::OscAppMeta;
use crate::{games, util};
use iced::Window;
use rfd::FileDialog;
use std::fmt::Write;
use std::path::PathBuf;

pub fn choose_mount_point(window: &dyn Window) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select Drive/Mount Point")
        .set_parent(&window)
        .pick_folder()
}

pub fn choose_games(window: &dyn Window) -> Vec<(PathBuf, GameID)> {
    let unfiltered_paths = FileDialog::new()
        .set_title("Select games")
        .set_parent(&window)
        .add_filter("NINTENDO OPTICAL DISC", SUPPORTED_INPUT_EXTENSIONS)
        .pick_files()
        .unwrap_or_default();

    unfiltered_paths
        .into_iter()
        .filter_map(maybe_path_to_entry)
        .map(|(path, _, id, _)| (path, id))
        .collect()
}

pub fn choose_src_dir(window: &dyn Window) -> Vec<(PathBuf, GameID)> {
    let dir = FileDialog::new()
        .set_title("Select a folder containing games")
        .set_parent(&window)
        .pick_folder();

    match dir {
        Some(dir) => games::util::scan_for_discs(dir),
        None => Vec::new(),
    }
}

pub fn choose_hbc_apps(window: &dyn Window) -> Vec<PathBuf> {
    let unfiltered_paths = FileDialog::new()
        .set_title("Select Homebrew Channel Apps")
        .set_parent(&window)
        .add_filter("HBC App", &["zip"])
        .pick_files()
        .unwrap_or_default();

    unfiltered_paths
        .into_iter()
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        })
        .collect()
}

pub fn choose_file_to_wiiload(window: &dyn Window) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select file to Wiiload")
        .set_parent(&window)
        .add_filter("HBC App", &["zip", "dol", "elf"])
        .pick_file()
}

pub fn confirm_strip_game(window: &dyn Window, game: Game) -> (Game, bool) {
    let res = MessageDialog::new()
        .set_level(WARNING_LEVEL)
        .set_title("Remove update partition?")
        .set_description(format!(
            "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!", game.title()
        ))
            .set_buttons(MessageButtons::OkCancel)
        .set_parent(&window)
        .show();

    (game, res == MessageDialogResult::Ok)
}

pub fn confirm_strip_all_games(window: &dyn Window) -> bool {
    let res = MessageDialog::new()
        .set_level(WARNING_LEVEL)
        .set_title("Remove update partitions?")
        .set_description("Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!")
        .set_buttons(MessageButtons::OkCancel)
        .set_parent(&window)
        .show();

    res == MessageDialogResult::Ok
}

pub fn choose_game_to_archive_manually(window: &dyn Window) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select input disc file")
        .set_parent(&window)
        .add_filter("Nintendo Optical Disc", SUPPORTED_DISC_EXTENSIONS)
        .pick_file()
}

pub fn confirm_install_osc_app(window: &dyn Window, app: OscAppMeta) -> (OscAppMeta, bool) {
    let res = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("Install OSC app")
        .set_description(format!("Are you sure you want to install {}?", app.name()))
        .set_buttons(MessageButtons::OkCancel)
        .set_parent(&window)
        .show();

    (app, res == MessageDialogResult::Ok)
}

pub fn choose_archive_dest(
    window: &dyn Window,
    source: PathBuf,
    title: String,
) -> Option<(PathBuf, String, PathBuf)> {
    let window_title = format!(
        "Archiving {title}\n\nSupported extensions: {}",
        SUPPORTED_DISC_EXTENSIONS.join(", ")
    );

    let default_file_name = format!("{}.rvz", util::sanitize(&title));

    let path = FileDialog::new()
        .set_title(&window_title)
        .set_parent(&window)
        .add_filter("NINTENDO OPTICAL DISC", SUPPORTED_DISC_EXTENSIONS)
        .set_file_name(default_file_name)
        .set_can_create_directories(true)
        .save_file()?;

    Some((source, title, path))
}
