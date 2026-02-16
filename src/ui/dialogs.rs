// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::extensions::{
    SUPPORTED_DISC_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS, ext_to_format,
};
use crate::games::game::Game;
use crate::games::game_id::GameID;
use crate::hbc::osc::OscAppMeta;
use crate::message::Message;
use crate::util;
use iced::Window;
use native_dialog::{DialogBuilder, MessageLevel};
use nod::common::Format;
use std::fmt::Write;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

fn confirm(
    window: &dyn Window,
    title: String,
    text: String,
    level: MessageLevel,
    on_confirm: Message,
) -> Message {
    let dialog = DialogBuilder::message()
        .set_owner(&window)
        .set_title(title)
        .set_text(text)
        .set_level(level)
        .confirm();

    match dialog.show() {
        Ok(true) => on_confirm,
        Ok(false) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

fn alert(window: &dyn Window, title: String, text: Option<String>, level: MessageLevel) -> Message {
    let mut dialog = DialogBuilder::message()
        .set_owner(&window)
        .set_title(title)
        .set_level(level);

    if let Some(text) = text {
        dialog = dialog.set_text(text);
    }

    let dialog = dialog.alert();

    match dialog.show() {
        Ok(()) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

fn pick_dir(
    window: &dyn Window,
    title: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .open_single_dir();

    match dialog.show() {
        Ok(Some(path)) => on_picked(path),
        Ok(None) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

#[cfg(not(feature = "windows-legacy"))]
fn pick_file(
    window: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .add_filters(filters)
        .open_single_file();

    match dialog.show() {
        Ok(Some(path)) => on_picked(path),
        Ok(None) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

#[cfg(feature = "windows-legacy")]
fn pick_file(
    _window: &dyn Window,
    _title: String,
    _filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    use windows::{
        Win32::UI::Controls::Dialogs::{
            GetOpenFileNameW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_PATHMUSTEXIST, OPENFILENAMEW,
        },
        core::PWSTR,
    };

    let filter = "All Files\0*.*\0\0";
    let filter_u16: Vec<u16> = filter.encode_utf16().collect();
    let mut file_buffer = [0u16; 260];

    let yes = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            lpstrFilter: PCWSTR(filter_u16.as_ptr()),
            lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
            nMaxFile: file_buffer.len() as u32,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST,
            ..Default::default()
        };

        GetOpenFileNameW(&mut ofn).as_bool()
    };

    if yes {
        let path = String::from_utf16_lossy(&file_buffer).trim_matches(char::from(0));
        on_picked(PathBuf::from(path))
    } else {
        Message::None
    }
}

fn pick_files(
    window: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(Vec<PathBuf>) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .add_filters(filters)
        .open_multiple_file();

    match dialog.show() {
        Ok(paths) if !paths.is_empty() => on_picked(paths),
        Ok(_) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

fn save_file(
    window: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    filename: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .add_filters(filters)
        .set_filename(filename)
        .save_single_file();

    match dialog.show() {
        Ok(Some(path)) => on_picked(path),
        Ok(None) => Message::None,
        Err(e) => Message::GenericError(e.to_string()),
    }
}

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
    let text = Some("All selected games are already installed.".to_string());
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
    let text = None;
    let level = MessageLevel::Warning;

    alert(window, title, text, level)
}
