// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::{
    SUPPORTED_DISC_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS, ext_to_format,
};
use crate::games::game::Game;
use crate::games::game_id::GameID;
use crate::hbc::osc::OscAppMeta;
use crate::message::Message;
use crate::ui::os_dialogs::{
    MessageLevel, alert, confirm, pick_dir, pick_file, pick_files, save_file,
};
use crate::util;
use iced::Window;
use nod::common::Format;
use std::fmt::Write;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

pub fn confirm_delete_dir(window: &dyn Window, path: PathBuf) -> Message {
    let title = "Delete Directory".to_string();
    let text = format!("Are you sure you want to delete {}?", path.display());
    let level = MessageLevel::Warning;
    let on_confirm = Message::DeleteDirConfirmed(path);

    confirm(window, title, text, level, on_confirm)
}

pub fn pick_mount_point(window: &dyn Window) -> Message {
    let title = "Select Drive/Mount Point".to_string();
    let on_picked = |path| Message::MountPointPicked(path);

    pick_dir(window, title, on_picked)
}

pub fn pick_games(window: &dyn Window) -> Message {
    let title = "Select Games".to_string();
    let on_picked = |path| Message::ConfirmAddGamesToTransferStack(path);
    let filters = [(
        "Nintendo Optical Disc".to_string(),
        SUPPORTED_INPUT_EXTENSIONS
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    )];

    pick_files(window, title, filters, on_picked)
}

pub fn pick_games_dir(window: &dyn Window) -> Message {
    let title = "Select a folder containing games".to_string();
    let on_picked = |path| {
        let paths = WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| !name.starts_with('.'))
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext_to_format(ext).is_some())
            })
            .map(DirEntry::into_path)
            .collect::<Vec<_>>();

        if paths.is_empty() {
            Message::None
        } else {
            Message::ConfirmAddGamesToTransferStack(paths)
        }
    };

    pick_dir(window, title, on_picked)
}

pub fn confirm_add_games(
    window: &dyn Window,
    entries: Vec<(PathBuf, Format, GameID, String)>,
) -> Message {
    let title = "The following games will be added".to_string();

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

    let level = MessageLevel::Info;

    let paths = entries
        .into_iter()
        .map(|(p, _, _, _)| p)
        .collect::<Vec<_>>();

    let on_confirm = Message::AddGamesToTransferStack(paths);

    confirm(window, title, text, level, on_confirm)
}

pub fn pick_hbc_apps(window: &dyn Window) -> Message {
    let title = "Select Homebrew Channel Apps".to_string();
    let on_picked = |paths| Message::AddHbcApps(paths);
    let filters = [("HBC App".to_string(), vec!["zip".to_string()])];

    pick_files(window, title, filters, on_picked)
}

pub fn pick_hbc_app_to_wiiload(window: &dyn Window) -> Message {
    let title = "Select HBC App to Wiiload".to_string();
    let on_picked = |path| Message::Wiiload(path);
    let filters = [(
        "HBC App".to_string(),
        vec!["zip".to_string(), "dol".to_string(), "elf".to_string()],
    )];

    pick_file(window, title, filters, on_picked)
}

pub fn pick_game_to_convert(window: &dyn Window) -> Message {
    let title = "Select Game to Convert".to_string();
    let on_picked = |path| Message::SetManualArchivingGame(path);
    let filters = [(
        "Nintendo Optical Disc".to_string(),
        SUPPORTED_DISC_EXTENSIONS
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    )];

    pick_file(window, title, filters, on_picked)
}

pub fn pick_archive_dest(window: &dyn Window, source: PathBuf, game_title: String) -> Message {
    let title = format!(
        "Archiving {game_title}\n\nSupported extensions: {}",
        SUPPORTED_DISC_EXTENSIONS.join(", ")
    );

    let filename = format!("{}.rvz", util::sanitize(&game_title));

    let on_picked = move |path| Message::ArchiveGame(source, game_title, path);

    let filters = [(
        "Nintendo Optical Disc".to_string(),
        SUPPORTED_DISC_EXTENSIONS
            .iter()
            .map(std::string::ToString::to_string)
            .collect(),
    )];

    save_file(window, title, filters, filename, on_picked)
}

pub fn no_new_games(window: &dyn Window) -> Message {
    let title = "No new games to add".to_string();
    let text = "All selected games are already installed.".to_string();
    let level = MessageLevel::Info;

    alert(window, title, text, level)
}

pub fn confirm_strip_game(window: &dyn Window, game: Game) -> Message {
    let title = "Remove update partition?".to_string();
    let text = format!(
        "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!",
        game.title()
    );
    let level = MessageLevel::Warning;
    let on_confirm = Message::StripGame(game);

    confirm(window, title, text, level, on_confirm)
}

pub fn confirm_strip_all_games(window: &dyn Window) -> Message {
    let title = "Remove update partitions?".to_string();
    let text = "Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!".to_string();
    let level = MessageLevel::Warning;
    let on_confirm = Message::StripAllGames;

    confirm(window, title, text, level, on_confirm)
}

pub fn confirm_install_osc_app(window: &dyn Window, app: OscAppMeta) -> Message {
    let title = "Install OSC App".to_string();
    let text = format!("Are you sure you want to install {}?", app.name());
    let level = MessageLevel::Info;
    let on_confirm = Message::InstallOscApp(app);

    confirm(window, title, text, level, on_confirm)
}

pub fn no_archive_source(window: &dyn Window) -> Message {
    let title = "No archive source found".to_string();
    let text = String::new();
    let level = MessageLevel::Warning;

    alert(window, title, text, level)
}
