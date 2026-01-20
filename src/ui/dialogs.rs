// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::{HBC_APP_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS, ZIP_EXTENSIONS};
use crate::util;
use iced::Window;
use iced::futures::future::join_all;
use native_dialog::{DialogBuilder, MessageLevel};
use std::ffi::OsStr;
use std::fmt::Write;
use std::path::PathBuf;

pub fn choose_mount_point(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select Drive/Mount Point")
        .set_owner(&window)
        .open_single_dir()
        .show()
        .unwrap_or_default()
}

pub fn choose_games(window: &dyn Window) -> Vec<PathBuf> {
    let paths = DialogBuilder::file()
        .set_title("Select games")
        .set_owner(&window)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .open_multiple_file()
        .show()
        .unwrap_or_default();

    smol::block_on(async move {
        join_all(paths.into_iter().map(util::keep_disc_file))
            .await
            .into_iter()
            .flatten()
            .collect()
    })
}

pub fn choose_src_dir(window: &dyn Window) -> Vec<PathBuf> {
    let dir = DialogBuilder::file()
        .set_title("Select a folder containing games")
        .set_owner(&window)
        .open_single_dir()
        .show()
        .unwrap_or_default();

    match dir {
        Some(dir) => smol::block_on(util::scan_for_discs(dir)).unwrap_or_default(),
        None => Vec::new(),
    }
}

pub fn choose_hbc_apps(window: &dyn Window) -> Box<[PathBuf]> {
    DialogBuilder::file()
        .set_title("Select Homebrew Channel Apps")
        .set_owner(&window)
        .add_filter("HBC App", ZIP_EXTENSIONS)
        .open_multiple_file()
        .show()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            p.extension()
                .and_then(OsStr::to_str)
                .is_some_and(|ext| ext == "zip")
        })
        .collect()
}

pub fn choose_dest_dir(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Save game to:")
        .set_owner(&window)
        .open_single_dir()
        .show()
        .unwrap_or_default()
}

pub fn choose_file_to_push(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select file to Wiiload")
        .set_owner(&window)
        .add_filter("HBC App", HBC_APP_EXTENSIONS)
        .open_single_file()
        .show()
        .unwrap_or_default()
}

pub fn delete_dir(path: PathBuf) -> impl FnOnce(&dyn Window) -> Result<(), String> {
    move |window: &dyn Window| {
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
            std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

pub fn confirm_add_games(window: &dyn Window, paths: &[PathBuf]) -> bool {
    const MAX: usize = 20;

    let mut desc = String::new();
    for path in paths.iter().take(MAX) {
        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("Invalid file name");

        desc.push_str("• ");
        desc.push_str(file_name);
        desc.push('\n');
    }

    let not_shown = paths.len().saturating_sub(MAX);
    if not_shown > 0 {
        desc.push_str("\n... and ");
        let _ = write!(desc, "{}", not_shown);
        desc.push_str(" more");
    }

    desc.push_str("\n\nAlready present games will be skipped\nAre you sure you want to continue?");

    DialogBuilder::message()
        .set_title("The following games will be added")
        .set_owner(&window)
        .set_text(desc)
        .set_level(MessageLevel::Info)
        .confirm()
        .show()
        .unwrap_or_default()
}

pub fn confirm_strip_game(window: &dyn Window, game_title: &str) -> bool {
    DialogBuilder::message()
        .set_title("Remove update partition?")
        .set_owner(&window)
        .set_text(format!(
            "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!",
            game_title
        ))
        .set_level(MessageLevel::Warning)
        .confirm()
        .show()
        .unwrap_or_default()
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

pub fn choose_input_disc_path(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select input disc file")
        .set_owner(&window)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .open_single_file()
        .show()
        .unwrap_or_default()
}

pub fn choose_output_disc_path(window: &dyn Window) -> Option<PathBuf> {
    DialogBuilder::file()
        .set_title("Select output disc file")
        .set_owner(&window)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .save_single_file()
        .show()
        .unwrap_or_default()
}

pub fn confirm_single_conversion(window: &dyn Window, in_path: &str, out_path: &str) -> bool {
    DialogBuilder::message()
        .set_title("Convert disc")
        .set_owner(&window)
        .set_text(format!("Convert {} to {}?", in_path, out_path))
        .set_level(MessageLevel::Info)
        .confirm()
        .show()
        .unwrap_or_default()
}

pub fn confirm_cancel_tasks(window: &dyn Window) -> bool {
    DialogBuilder::message()
        .set_title("Cancel pending tasks")
        .set_owner(&window)
        .set_text("Are you sure you want to cancel all pending tasks?")
        .set_level(MessageLevel::Warning)
        .confirm()
        .show()
        .unwrap_or_default()
}

pub fn confirm_install_osc_app(window: &dyn Window, app_name: String) -> bool {
    DialogBuilder::message()
        .set_title("Install OSC app")
        .set_owner(&window)
        .set_text(format!("Are you sure you want to install {}?", app_name))
        .set_level(MessageLevel::Info)
        .confirm()
        .show()
        .unwrap_or_default()
}
