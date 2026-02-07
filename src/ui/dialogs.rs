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
use native_dialog::{DialogBuilder, MessageLevel};
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

pub fn choose_mount_point(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select Drive/Mount Point")
        .set_owner(&window)
        .open_single_dir()
        .show()
        .unwrap_or_default()
}

pub fn choose_games(window: &dyn Window) -> Vec<(PathBuf, GameID)> {
    DialogBuilder::file()
        .set_title("Select games")
        .set_owner(&window)
        .add_filter("NINTENDO OPTICAL DISC", SUPPORTED_INPUT_EXTENSIONS)
        .open_multiple_file()
        .show()
        .unwrap_or_default()
        .into_iter()
        .filter_map(maybe_path_to_entry)
        .map(|(path, _, id, _)| (path, id))
        .collect()
}

pub fn choose_src_dir(window: &dyn Window) -> Vec<(PathBuf, GameID)> {
    let dir = DialogBuilder::file()
        .set_title("Select a folder containing games")
        .set_owner(&window)
        .open_single_dir()
        .show()
        .unwrap_or_default();

    match dir {
        Some(dir) => games::util::scan_for_discs(dir),
        None => Vec::new(),
    }
}

pub fn choose_hbc_apps(window: &dyn Window) -> Vec<PathBuf> {
    DialogBuilder::file()
        .set_title("Select Homebrew Channel Apps")
        .set_owner(&window)
        .add_filter("HBC App", ["zip"])
        .open_multiple_file()
        .show()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            p.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        })
        .collect()
}

pub fn choose_file_to_wiiload(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select file to Wiiload")
        .set_owner(&window)
        .add_filter("HBC App", ["zip", "dol", "elf"])
        .open_single_file()
        .show()
        .unwrap_or_default()
}

pub fn delete_dir(window: &dyn Window, path: &Path) -> Result<()> {
    let yes = DialogBuilder::message()
        .set_title("Delete directory")
        .set_owner(&window)
        .set_text(format!(
            "Are you sure you want to delete {}?",
            path.display()
        ))
        .set_level(MessageLevel::Warning)
        .confirm()
        .show()
        .unwrap_or_default();

    if yes {
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

    let yes = DialogBuilder::message()
        .set_title("The following games will be added")
        .set_owner(&window)
        .set_text(desc)
        .set_level(MessageLevel::Info)
        .confirm()
        .show()
        .unwrap_or_default();

    let paths = entries.into_iter().map(|(path, _)| path).collect();

    (paths, yes)
}

pub fn no_new_games(window: &dyn Window) {
    let _ = DialogBuilder::message()
        .set_title("No new games to add")
        .set_owner(&window)
        .set_text(
            "Either you didn't select any valid game, or all the games are already installed.",
        )
        .set_level(MessageLevel::Info)
        .alert()
        .show();
}

pub fn confirm_strip_game(window: &dyn Window, game: Game) -> (Game, bool) {
    let yes = DialogBuilder::message()
        .set_title("Remove update partition?")
        .set_owner(&window)
        .set_text(format!(
            "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!", game.title()
        ))
        .set_level(MessageLevel::Warning)
        .confirm()
        .show()
        .unwrap_or_default();

    (game, yes)
}

pub fn confirm_strip_all_games(window: &dyn Window) -> bool {
    DialogBuilder::message()
        .set_title("Remove update partitions?")
        .set_owner(&window)
        .set_text("Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!")
        .set_level(MessageLevel::Warning)
        .confirm()
        .show()
        .unwrap_or_default()
}

pub fn choose_game_to_archive_manually(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select input disc file")
        .set_owner(&window)
        .add_filter("Nintendo Optical Disc", SUPPORTED_DISC_EXTENSIONS)
        .open_single_file()
        .show()
        .unwrap_or_default()
}

pub fn confirm_install_osc_app(window: &dyn Window, app: OscAppMeta) -> (OscAppMeta, bool) {
    let yes = DialogBuilder::message()
        .set_title("Install OSC app")
        .set_owner(&window)
        .set_text(format!("Are you sure you want to install {}?", app.name()))
        .set_level(MessageLevel::Info)
        .confirm()
        .show()
        .unwrap_or_default();

    (app, yes)
}

pub fn choose_archive_dest(
    window: &dyn Window,
    source: PathBuf,
    title: String,
) -> Option<(PathBuf, String, PathBuf)> {
    let window_title = format!("Archive {title}");

    let default_file_name = format!("{}.rvz", util::sanitize(&title));

    let path = DialogBuilder::file()
        .set_title(&window_title)
        .set_owner(&window)
        .add_filter("RVZ", ["rvz"])
        .add_filter("ISO", ["iso"])
        .add_filter("WBFS", ["wbfs"])
        .add_filter("WIA", ["wia"])
        .add_filter("CISO", ["ciso"])
        .add_filter("GCZ", ["gcz"])
        .add_filter("TGC", ["tgc"])
        .add_filter("NFS", ["nfs"])
        .set_filename(default_file_name)
        .save_single_file()
        .show()
        .unwrap_or_default()?;

    Some((source, title, path))
}
