// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use rfd::FileDialog;
use slint::WindowHandle;
use std::path::PathBuf;
use walkdir::WalkDir;

const INPUT_DIALOG_FILTER: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs", "zip",
];

const OUTPUT_DIALOG_FILTER: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
];

pub fn pick_mount_point(window_handle: &WindowHandle) -> Option<PathBuf> {
    FileDialog::new()
        .set_parent(window_handle)
        .set_title("Select Drive/Mount Point")
        .pick_folder()
}

pub fn pick_game(window_handle: &WindowHandle) -> Option<PathBuf> {
    FileDialog::new()
        .set_parent(window_handle)
        .set_title("Select Game")
        .add_filter("Nintendo Optical Disc", INPUT_DIALOG_FILTER)
        .pick_file()
}

pub fn pick_games(window_handle: &WindowHandle) -> Vec<PathBuf> {
    FileDialog::new()
        .set_parent(window_handle)
        .set_title("Select Games")
        .add_filter("Nintendo Optical Disc", INPUT_DIALOG_FILTER)
        .pick_files()
        .unwrap_or_default()
}

pub fn pick_games_r(window_handle: &WindowHandle) -> Vec<PathBuf> {
    let res = FileDialog::new()
        .set_parent(window_handle)
        .set_title("Select folder (games will be searched recursively)")
        .pick_folder();

    let mut paths = Vec::new();

    let Some(res) = res else {
        return paths;
    };

    for entry in WalkDir::new(res).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file()
            && let Some(ext) = entry.path().extension()
            && INPUT_DIALOG_FILTER
                .iter()
                .any(|e| ext.eq_ignore_ascii_case(e))
        {
            paths.push(entry.into_path());
        }
    }

    paths
}

pub fn save_game(window_handle: &WindowHandle, game_title: &str) -> Option<PathBuf> {
    let title = format!(
        "Select Destination for {game_title} - Supported extensions: iso, wbfs, wia, rvz, ciso, gcz, tgc, nfs",
    );

    let filename = format!("{}.rvz", twbm_core::util::sanitize(game_title));

    FileDialog::new()
        .set_parent(window_handle)
        .set_title(title)
        .set_file_name(filename)
        .add_filter("Nintendo Optical Disc", OUTPUT_DIALOG_FILTER)
        .save_file()
}

pub fn pick_homebrew_apps(window_handle: &WindowHandle) -> Vec<PathBuf> {
    FileDialog::new()
        .set_parent(window_handle)
        .set_title("Select Homebrew apps")
        .add_filter("ZIP", &["zip"])
        .pick_files()
        .unwrap_or_default()
}

pub fn pick_wiiload(window_handle: &WindowHandle) -> Option<PathBuf> {
    FileDialog::new()
        .set_parent(window_handle)
        .set_title("Select Homebrew apps")
        .add_filter("ZIP/DOL/ELF", &["zip", "dol", "elf"])
        .pick_file()
}
