// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::{INPUT_DIALOG_FILTER, OUTPUT_DIALOG_FILTER, ext_to_format};
use crate::games::game::Game;
use crate::games::game_id::GameID;
use crate::games::util::maybe_path_to_entry;
use crate::hbc::osc::OscAppMeta;
use crate::message::Message;
use crate::util;
use iced::Window;
use nod::common::Format;
use rfd::MessageLevel;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

#[cfg(not(target_vendor = "win7"))]
use rfd::FileDialog;

#[cfg(target_vendor = "win7")]
use crate::ui::xp_dialogs;

pub fn confirm_delete_dir(path: PathBuf) -> Message {
    Message::OpenMessageBox(
        "Delete Directory".to_string(),
        format!("Are you sure you want to delete {}?", path.display()),
        MessageLevel::Warning,
        Some(Box::new(Message::DeleteDirConfirmed(path))),
    )
}

pub fn pick_mount_point(window: &dyn Window) -> Message {
    #[cfg(not(target_vendor = "win7"))]
    let res = FileDialog::new()
        .set_parent(window)
        .set_title("Select Drive/Mount Point")
        .pick_folder();

    #[cfg(target_vendor = "win7")]
    let res = xp_dialogs::pick_dir(window, "Select Drive/Mount Point");

    match res {
        Some(path) => Message::MountPointPicked(path),
        None => Message::None,
    }
}

pub fn pick_games(window: &dyn Window, existing_ids: &[GameID]) -> Message {
    #[cfg(not(target_vendor = "win7"))]
    let paths = FileDialog::new()
        .set_parent(window)
        .set_title("Select Games")
        .add_filter(INPUT_DIALOG_FILTER.0, INPUT_DIALOG_FILTER.1)
        .pick_files()
        .unwrap_or_default();

    #[cfg(target_vendor = "win7")]
    let paths = xp_dialogs::pick_files(window, "Select Games", INPUT_DIALOG_FILTER);

    let mut entries = paths
        .into_iter()
        .filter_map(maybe_path_to_entry)
        .collect::<Vec<_>>();

    // remove already installed games
    entries.retain(|(path, _, id, _)| {
        let is_multidisc = path.file_stem().and_then(OsStr::to_str).is_some_and(|s| {
            let s = s.to_ascii_lowercase();
            s.contains("disc 1") || s.contains("disc 2")
        });

        let is_installed = existing_ids.contains(id);

        is_multidisc || !is_installed
    });

    if entries.is_empty() {
        no_new_games()
    } else {
        confirm_add_games(entries)
    }
}

pub fn pick_games_dir(window: &dyn Window, existing_ids: &[GameID]) -> Message {
    #[cfg(not(target_vendor = "win7"))]
    let res = FileDialog::new()
        .set_parent(window)
        .set_title("Select a folder containing games")
        .pick_folder();

    #[cfg(target_vendor = "win7")]
    let res = xp_dialogs::pick_dir(window, "Select a folder containing games");

    let paths = match res {
        Some(path) => WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .map(DirEntry::into_path)
            .filter(|path| {
                if !path.is_file() {
                    return false;
                }

                let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
                    return false;
                };

                if stem.starts_with('.') {
                    return false;
                }

                let Some(ext) = path.extension() else {
                    return false;
                };

                ext.eq_ignore_ascii_case("zip") || ext_to_format(ext).is_some()
            })
            .collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let mut entries = paths
        .into_iter()
        .filter_map(maybe_path_to_entry)
        .collect::<Vec<_>>();

    // remove already installed games
    entries.retain(|(path, _, id, _)| {
        let is_multidisc = path.file_stem().and_then(OsStr::to_str).is_some_and(|s| {
            let s = s.to_ascii_lowercase();
            s.contains("disc 1") || s.contains("disc 2")
        });

        let is_installed = existing_ids.contains(id);

        is_multidisc || !is_installed
    });

    if entries.is_empty() {
        no_new_games()
    } else {
        confirm_add_games(entries)
    }
}

fn confirm_add_games(entries: Vec<(PathBuf, Format, GameID, String)>) -> Message {
    let mut text = String::new();

    for (_, _, id, game_title) in &entries {
        let _ = writeln!(text, "• {} [{}]", game_title, id.as_str());
    }

    let paths = entries
        .into_iter()
        .map(|(p, _, _, _)| p)
        .collect::<Vec<_>>();

    Message::OpenMessageBox(
        "The following games will be added, are you sure you want to continue?".to_string(),
        text,
        MessageLevel::Info,
        Some(Box::new(Message::AddGamesToTransferStack(paths))),
    )
}

pub fn pick_hbc_apps(window: &dyn Window) -> Message {
    #[cfg(not(target_vendor = "win7"))]
    let paths = FileDialog::new()
        .set_parent(window)
        .set_title("Select Homebrew Channel Apps")
        .add_filter("HBC App", &["zip"])
        .pick_files()
        .unwrap_or_default();

    #[cfg(target_vendor = "win7")]
    let paths = xp_dialogs::pick_files(
        window,
        "Select Homebrew Channel Apps",
        ("HBC App", &["zip"]),
    );

    if paths.is_empty() {
        Message::None
    } else {
        Message::AddHbcApps(paths)
    }
}

pub fn pick_hbc_app_to_wiiload(window: &dyn Window) -> Message {
    #[cfg(not(target_vendor = "win7"))]
    let res = FileDialog::new()
        .set_parent(window)
        .set_title("Select HBC App to Wiiload")
        .add_filter("HBC App", &["zip", "dol", "elf"])
        .pick_file();

    #[cfg(target_vendor = "win7")]
    let res = xp_dialogs::pick_file(
        window,
        "Select HBC App to Wiiload",
        ("HBC App", &["zip", "dol", "elf"]),
    );

    match res {
        Some(path) => Message::Wiiload(path),
        None => Message::None,
    }
}

pub fn pick_game_to_convert(window: &dyn Window) -> Message {
    #[cfg(not(target_vendor = "win7"))]
    let res = FileDialog::new()
        .set_parent(window)
        .set_title("Select Game to Convert")
        .add_filter(OUTPUT_DIALOG_FILTER.0, OUTPUT_DIALOG_FILTER.1)
        .pick_file();

    #[cfg(target_vendor = "win7")]
    let res = xp_dialogs::pick_file(window, "Select Game to Convert", OUTPUT_DIALOG_FILTER);

    match res {
        Some(path) => Message::SetManualArchivingGame(path),
        None => Message::None,
    }
}

pub fn pick_archive_dest(window: &dyn Window, source: PathBuf, game_title: String) -> Message {
    let title = format!(
        "Select Destination for {game_title} | Supported extensions: iso, wbfs, wia, rvz, ciso, gcz, tgc, nfs"
    );
    let filename = format!("{}.rvz", util::sanitize(&game_title));

    #[cfg(not(target_vendor = "win7"))]
    let res = FileDialog::new()
        .set_parent(window)
        .set_title(&title)
        .add_filter(OUTPUT_DIALOG_FILTER.0, OUTPUT_DIALOG_FILTER.1)
        .set_file_name(&filename)
        .pick_file();

    #[cfg(target_vendor = "win7")]
    let res = xp_dialogs::save_file(window, &title, OUTPUT_DIALOG_FILTER, &filename);

    match res {
        Some(path) => Message::ArchiveGame(source, game_title, path),
        None => Message::None,
    }
}

fn no_new_games() -> Message {
    Message::OpenMessageBox(
        "No new games to add".to_string(),
        "All selected games are already installed.".to_string(),
        MessageLevel::Info,
        None,
    )
}

pub fn confirm_strip_game(game: Game) -> Message {
    Message::OpenMessageBox(
        "Remove update partition?".to_string(),
        format!(
            "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!",
            game.title()
        ),
        MessageLevel::Warning,
        Some(Box::new(Message::StripGame(game))),
    )
}

pub fn confirm_strip_all_games() -> Message {
    Message::OpenMessageBox(
        "Remove update partitions?".to_string(),
        "Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!".to_string(),
        MessageLevel::Warning,
        Some(Box::new(Message::StripAllGames)),
    )
}

pub fn confirm_install_osc_app(app: OscAppMeta) -> Message {
    Message::OpenMessageBox(
        "Install OSC App".to_string(),
        format!("Are you sure you want to install {}?", app.name()),
        MessageLevel::Info,
        Some(Box::new(Message::InstallOscApp(app))),
    )
}

pub fn no_archive_source() -> Message {
    Message::OpenMessageBox(
        "No archive source found".to_string(),
        "No archive source was found for the selected game.".to_string(),
        MessageLevel::Error,
        None,
    )
}
