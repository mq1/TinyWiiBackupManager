// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::state::AppState;

mod components;
pub mod pages;
pub mod root;

pub fn title(state: &AppState) -> String {
    if let Some(drive_info) = &state.drive_info {
        format!(
            "TinyWiiBackupManager  ›  {}  ({}/{})",
            drive_info.label, drive_info.used_size, drive_info.total_size
        )
    } else {
        String::from("TinyWiiBackupManager  ›  No drive selected")
    }
}
