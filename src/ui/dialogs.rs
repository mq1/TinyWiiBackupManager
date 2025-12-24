// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::{HBC_APP_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS, ZIP_EXTENSIONS};
use crate::util;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::ffi::OsStr;
use std::path::PathBuf;

pub fn choose_mount_point(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select Drive/Mount Point")
        .set_parent(frame)
        .pick_folder()
}

pub fn choose_games(frame: &eframe::Frame) -> Box<[PathBuf]> {
    let mut paths = FileDialog::new()
        .set_title("Select games")
        .set_parent(frame)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .pick_files()
        .unwrap_or_default();

    paths.retain(|path| {
        path.extension()
            .and_then(OsStr::to_str)
            .is_some_and(|ext| SUPPORTED_INPUT_EXTENSIONS.contains(&ext))
    });

    // nod already fetches these automatically
    paths.retain(|path| {
        path.file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| !name.ends_with(".part1.iso"))
    });

    // filter out invalid zip files
    paths.retain(|path| {
        path.extension() != Some(OsStr::new("zip")) || util::does_this_zip_contain_a_disc(path)
    });

    paths.into_boxed_slice()
}

pub fn choose_src_dir(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select a folder containing games")
        .set_parent(frame)
        .pick_folder()
}

pub fn choose_hbc_apps(frame: &eframe::Frame) -> Vec<PathBuf> {
    FileDialog::new()
        .set_title("Select Homebrew Channel Apps")
        .set_parent(frame)
        .add_filter("HBC App", ZIP_EXTENSIONS)
        .pick_files()
        .unwrap_or_default()
}

pub fn choose_dest_dir(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Save game to:")
        .set_parent(frame)
        .pick_folder()
}

pub fn choose_file_to_push(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select file to Wiiload")
        .set_parent(frame)
        .add_filter("HBC App", HBC_APP_EXTENSIONS)
        .pick_file()
}

pub fn delete_game(frame: &eframe::Frame, game_title: &str) -> bool {
    MessageDialog::new()
        .set_title("Delete game")
        .set_parent(frame)
        .set_description(format!("Are you sure you want to delete {}?", game_title))
        .set_level(MessageLevel::Warning)
        .set_buttons(MessageButtons::OkCancel)
        .show()
        == MessageDialogResult::Ok
}

pub fn delete_hbc_app(frame: &eframe::Frame, app_name: &str) -> bool {
    MessageDialog::new()
        .set_title("Delete Homebrew Channel app")
        .set_parent(frame)
        .set_description(format!("Are you sure you want to delete {}?", app_name))
        .set_level(MessageLevel::Warning)
        .set_buttons(MessageButtons::OkCancel)
        .show()
        == MessageDialogResult::Ok
}

pub fn confirm_add_games(frame: &eframe::Frame, paths: &[PathBuf]) -> bool {
    let file_names = paths
        .iter()
        .filter_map(|path| path.file_name())
        .filter_map(OsStr::to_str);

    let mut desc = String::new();
    for file_name in file_names {
        desc.push_str(&format!("• {}\n", file_name));
    }
    desc.push_str("\n\nAre you sure you want to continue?");

    MessageDialog::new()
        .set_title("Games to convert (already present games will be skipped):")
        .set_parent(frame)
        .set_description(desc)
        .set_level(MessageLevel::Info)
        .set_buttons(MessageButtons::OkCancel)
        .show()
        == MessageDialogResult::Ok
}

pub fn confirm_strip_game(frame: &eframe::Frame, game_title: &str) -> bool {
    MessageDialog::new()
        .set_title("Remove update partition?")
        .set_parent(frame)
        .set_description(format!(
            "Are you sure you want to remove the update partition from {}?\n\nThis is irreversible!",
            game_title
        ))
        .set_level(MessageLevel::Warning)
        .set_buttons(MessageButtons::OkCancel)
        .show()
        == MessageDialogResult::Ok
}

pub fn confirm_strip_all_games(frame: &eframe::Frame) -> bool {
    MessageDialog::new()
        .set_title("Remove update partitions?")
        .set_parent(frame)
        .set_description("Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!")
        .set_level(MessageLevel::Warning)
        .set_buttons(MessageButtons::OkCancel)
        .show() == MessageDialogResult::Ok
}

pub fn choose_input_disc_path(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select input disc file")
        .set_parent(frame)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .pick_file()
}

pub fn choose_output_disc_path(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select output disc file")
        .set_parent(frame)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .save_file()
}

pub fn confirm_single_conversion(frame: &eframe::Frame, in_path: &str, out_path: &str) -> bool {
    MessageDialog::new()
        .set_title("Convert disc")
        .set_parent(frame)
        .set_description(format!("Convert {} to {}?", in_path, out_path))
        .set_level(MessageLevel::Info)
        .set_buttons(MessageButtons::OkCancel)
        .show()
        == MessageDialogResult::Ok
}

pub fn confirm_cancel_tasks(frame: &eframe::Frame) -> bool {
    MessageDialog::new()
        .set_title("Cancel pending tasks")
        .set_parent(frame)
        .set_description("Are you sure you want to cancel all pending tasks?")
        .set_level(MessageLevel::Warning)
        .set_buttons(MessageButtons::OkCancel)
        .show()
        == MessageDialogResult::Ok
}
