// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use rfd::FileDialog;
use std::path::PathBuf;

pub fn pick_mount_point(w: &dyn iced::Window) -> Option<PathBuf> {
    FileDialog::new()
        .set_parent(w)
        .set_title("Select Drive/Mount Point")
        .pick_folder()
}

pub fn pick_homebrew_apps(w: &dyn iced::Window) -> Vec<PathBuf> {
    FileDialog::new()
        .set_parent(w)
        .set_title("Select Homebrew App(s) to import")
        .pick_files()
        .unwrap_or_default()
}

pub fn pick_games(w: &dyn iced::Window) -> Vec<PathBuf> {
    FileDialog::new()
        .set_parent(w)
        .set_title("Select Game(s) to import")
        .add_filter(
            "Wii/NGC rom",
            &["iso", "gcm", "wia", "rvz", "wbfs", "iso", "gcz", "tgc"],
        )
        .pick_files()
        .unwrap_or_default()
}
