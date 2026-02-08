// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::{SUPPORTED_DISC_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS};
use crate::games::game::Game;
use crate::games::game_id::GameID;
use crate::games::util::maybe_path_to_entry;
use crate::hbc::osc::OscAppMeta;
use crate::{games, util};
use anyhow::Result;
use iced::Window;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
const WARNING_LEVEL: MessageLevel = MessageLevel::Error;

#[cfg(not(target_os = "macos"))]
const WARNING_LEVEL: MessageLevel = MessageLevel::Warning;

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

pub fn delete_dir(window: &dyn Window, path: &Path) -> Result<()> {
    let res = MessageDialog::new()
        .set_level(WARNING_LEVEL)
        .set_title("Delete directory")
        .set_description(format!(
            "Are you sure you want to delete {}?",
            path.display()
        ))
        .set_buttons(MessageButtons::OkCancel)
        .set_parent(&window)
        .show();

    if res == MessageDialogResult::Ok {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

pub fn confirm_add_games(
    window: &dyn Window,
    entries: Vec<(PathBuf, GameID)>,
) -> (Vec<PathBuf>, bool) {
    const MAX: usize = 20;

    let mut desc = String::new();
    for (path, id) in entries.iter().take(MAX) {
        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("Invalid file name");

        let _ = writeln!(desc, "• {} [{}]", file_name, id.as_str());
    }

    let not_shown = entries.len().saturating_sub(MAX);
    if not_shown > 0 {
        let _ = writeln!(desc, "\n... and {not_shown} more");
    }

    let _ = write!(desc, "\nAre you sure you want to continue?");

    let res = MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("The following games will be added")
        .set_description(desc)
        .set_buttons(MessageButtons::OkCancel)
        .set_parent(&window)
        .show();

    let paths = entries.into_iter().map(|(path, _)| path).collect();

    (paths, res == MessageDialogResult::Ok)
}

pub fn no_new_games(window: &dyn Window) {
    let _ = MessageDialog::new()
        .set_level(WARNING_LEVEL)
        .set_title("No new games to add")
        .set_description(
            "Either you didn't select any valid game, or all the games are already installed.",
        )
        .set_buttons(MessageButtons::Ok)
        .set_parent(&window)
        .show();
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
    let window_title = format!("Archive {title}");

    let default_file_name = format!("{}.rvz", util::sanitize(&title));

    let path = FileDialog::new()
        .set_title(&window_title)
        .set_parent(&window)
        .add_filter("RVZ", &["rvz"])
        .add_filter("ISO", &["iso"])
        .add_filter("WBFS", &["wbfs"])
        .add_filter("WIA", &["wia"])
        .add_filter("CISO", &["ciso"])
        .add_filter("GCZ", &["gcz"])
        .add_filter("TGC", &["tgc"])
        .add_filter("NFS", &["nfs"])
        .set_file_name(default_file_name)
        .save_file()?;

    Some((source, title, path))
}
