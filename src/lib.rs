// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

pub const PRODUCT_NAME: &str = "TinyWiiBackupManager";

mod app;
mod components;
mod game;
mod titles;

use std::path::PathBuf;

pub use app::App;

/// Correct base_dir if the user has picked either "wbfs" or "games" dir.
pub fn correct_base_dir(base_dir: &mut PathBuf) {
    if let Some(file_name) = base_dir.file_name().and_then(|name| name.to_str()) {
        if file_name == "wbfs" || file_name == "games" {
            if let Some(parent) = base_dir.parent() {
                *base_dir = parent.to_path_buf();
            }
        }
    }
}
