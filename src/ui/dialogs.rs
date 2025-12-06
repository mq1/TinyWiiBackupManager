// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::{HBC_APP_EXTENSIONS, SUPPORTED_INPUT_EXTENSIONS, ZIP_EXTENSIONS};
use rfd::FileDialog;
use std::path::PathBuf;

pub fn choose_mount_point(frame: &eframe::Frame) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("Select Drive/Mount Point")
        .set_parent(frame)
        .pick_folder()
}

pub fn choose_games(frame: &eframe::Frame) -> Vec<PathBuf> {
    FileDialog::new()
        .set_title("Select games (or a directory containing games)")
        .set_parent(frame)
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .pick_files()
        .unwrap_or_default()
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
