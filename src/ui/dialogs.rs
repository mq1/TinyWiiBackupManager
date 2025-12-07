// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::disc_info::DiscInfo;
use crate::extensions::{HBC_APP_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS, ZIP_EXTENSIONS};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::path::PathBuf;

pub fn choose_mount_point(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select Drive/Mount Point")
        .set_parent(frame)
        .pick_folder()
}

pub fn choose_games(frame: &eframe::Frame) -> Vec<PathBuf> {
    FileDialog::new()
        .set_title("Select games")
        .set_parent(frame)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .pick_files()
        .unwrap_or_default()
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

pub fn choose_game_manual_conv(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select disc image to convert")
        .set_parent(frame)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
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

pub fn confirm_conversion(frame: &eframe::Frame, discs: &[DiscInfo]) -> bool {
    let mut desc = String::new();
    for disc in discs {
        desc.push_str(&format!("• {}\n", &disc.title));
    }
    desc.push_str("\n\nAre you sure you want to continue?");

    MessageDialog::new()
        .set_title("The following games will be added")
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
