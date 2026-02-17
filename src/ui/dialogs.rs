// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::{INPUT_DIALOG_FILTER, OUTPUT_DIALOG_FILTER, ext_to_format};
use crate::games::game::Game;
use crate::games::game_id::GameID;
use crate::hbc::osc::OscAppMeta;
use crate::message::Message;
use crate::util;
use blocking_dialog::{
    BlockingAlertDialog, BlockingConfirmDialog, BlockingDialogLevel, BlockingPickDirectoryDialog,
    BlockingPickFilesDialog, BlockingPickFilesDialogFilter, BlockingSaveFileDialog,
};
use iced::Window;
use nod::common::Format;
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub fn confirm_delete_dir(window: &dyn Window, path: PathBuf) -> Message {
    let dialog = BlockingConfirmDialog {
        window,
        title: "Delete Directory",
        message: &format!("Are you sure you want to delete {}?", path.display()),
        level: BlockingDialogLevel::Warning,
    };

    match dialog.show() {
        Ok(true) => Message::DeleteDirConfirmed(path),
        Ok(false) => Message::None,
        Err(e) => return Message::GenericError(e.to_string()),
    }
}

pub fn pick_mount_point(window: &dyn Window) -> Message {
    let dialog = BlockingPickDirectoryDialog {
        window,
        title: "Select Drive/Mount Point",
    };

    match dialog.show() {
        Ok(Some(path)) => Message::MountPointPicked(path),
        Ok(None) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn pick_games(window: &dyn Window) -> Message {
    let dialog = BlockingPickFilesDialog {
        window,
        title: "Select Games",
        multiple: true,
        filter: INPUT_DIALOG_FILTER,
    };

    match dialog.show() {
        Ok(games) if !games.is_empty() => Message::ConfirmAddGamesToTransferStack(games),
        Ok(_) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn pick_games_dir(window: &dyn Window) -> Message {
    let dialog = BlockingPickDirectoryDialog {
        window,
        title: "Select a folder containing games",
    };

    match dialog.show() {
        Ok(Some(path)) => {
            let paths = WalkDir::new(path)
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

                    ext.eq_ignore_ascii_case("zip") || ext_to_format(&ext).is_some()
                })
                .collect::<Vec<_>>();

            if paths.is_empty() {
                Message::None
            } else {
                Message::ConfirmAddGamesToTransferStack(paths)
            }
        }
        Ok(None) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn confirm_add_games(
    window: &dyn Window,
    entries: Vec<(PathBuf, Format, GameID, String)>,
) -> Message {
    let text = {
        const MAX: usize = 20;

        let mut text = String::new();
        for (_, _, id, game_title) in entries.iter().take(MAX) {
            let _ = writeln!(text, "• {} [{}]", game_title, id.as_str());
        }

        let not_shown = entries.len().saturating_sub(MAX);
        if not_shown > 0 {
            let _ = writeln!(text, "\n... and {not_shown} more");
        }

        let _ = write!(text, "\nAre you sure you want to continue?");

        text
    };

    let dialog = BlockingConfirmDialog {
        window,
        title: "The following games will be added",
        message: text.as_str(),
        level: BlockingDialogLevel::Info,
    };

    let paths = entries
        .into_iter()
        .map(|(p, _, _, _)| p)
        .collect::<Vec<_>>();

    match dialog.show() {
        Ok(true) => Message::AddGamesToTransferStack(paths),
        Ok(false) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn pick_hbc_apps(window: &dyn Window) -> Message {
    let dialog = BlockingPickFilesDialog {
        window,
        title: "Select Homebrew Channel Apps",
        multiple: true,
        filter: &[BlockingPickFilesDialogFilter {
            name: "HBC App",
            extensions: &["zip"],
        }],
    };

    match dialog.show() {
        Ok(hbc_apps) if !hbc_apps.is_empty() => Message::AddHbcApps(hbc_apps),
        Ok(_) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn pick_hbc_app_to_wiiload(window: &dyn Window) -> Message {
    let dialog = BlockingPickFilesDialog {
        window,
        title: "Select HBC App to Wiiload",
        multiple: false,
        filter: &[BlockingPickFilesDialogFilter {
            name: "HBC App",
            extensions: &["zip", "dol", "elf"],
        }],
    };

    match dialog.show() {
        Ok(mut hbc_apps) => match hbc_apps.pop() {
            Some(hbc_app) => Message::Wiiload(hbc_app),
            None => Message::None,
        },
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn pick_game_to_convert(window: &dyn Window) -> Message {
    let dialog = BlockingPickFilesDialog {
        window,
        title: "Select Game to Convert",
        multiple: false,
        filter: OUTPUT_DIALOG_FILTER,
    };

    match dialog.show() {
        Ok(mut paths) => match paths.pop() {
            Some(path) => Message::SetManualArchivingGame(path),
            None => Message::None,
        },
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn pick_archive_dest(window: &dyn Window, source: PathBuf, game_title: String) -> Message {
    let dialog = BlockingSaveFileDialog {
        window,
        title: &format!(
            "Select Destination for {game_title}\n\nSupported extensions: iso, wbfs, wia, rvz, ciso, gcz, tgc, nfs"
        ),
        default_filename: Some(&format!("{}.rvz", util::sanitize(&game_title))),
        filter: OUTPUT_DIALOG_FILTER,
    };

    match dialog.show() {
        Ok(Some(path)) => Message::ArchiveGame(source, game_title, path),
        Ok(None) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn no_new_games(window: &dyn Window) -> Message {
    let dialog = BlockingAlertDialog {
        window,
        title: "No new games to add",
        message: "All selected games are already installed.",
        level: BlockingDialogLevel::Info,
    };

    match dialog.show() {
        Ok(()) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn confirm_strip_game(window: &dyn Window, game: Game) -> Message {
    let dialog = BlockingConfirmDialog {
        window,
        title: "Remove update partition?",
        message: &format!(
            "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!",
            game.title()
        ),
        level: BlockingDialogLevel::Warning,
    };

    match dialog.show() {
        Ok(true) => Message::StripGame(game),
        Ok(false) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn confirm_strip_all_games(window: &dyn Window) -> Message {
    let dialog = BlockingConfirmDialog {
        window,
        title: "Remove update partitions?",
        message: "Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!",
        level: BlockingDialogLevel::Warning,
    };

    match dialog.show() {
        Ok(true) => Message::StripAllGames,
        Ok(false) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn confirm_install_osc_app(window: &dyn Window, app: OscAppMeta) -> Message {
    let dialog = BlockingConfirmDialog {
        window,
        title: "Install OSC App",
        message: &format!("Are you sure you want to install {}?", app.name()),
        level: BlockingDialogLevel::Info,
    };

    match dialog.show() {
        Ok(true) => Message::InstallOscApp(app),
        Ok(false) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

pub fn no_archive_source(window: &dyn Window) -> Message {
    let dialog = BlockingAlertDialog {
        window,
        title: "No archive source found",
        message: "No archive source was found for the selected game.",
        level: BlockingDialogLevel::Error,
    };

    match dialog.show() {
        Ok(()) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}
