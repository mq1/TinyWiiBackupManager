// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::state::AppState;
use std::fmt::Write;

pub mod components;
pub mod developers;
pub mod dialogs;
pub mod modals;
pub mod pages;
pub mod root;
pub mod theme;

pub fn title(state: &AppState) -> String {
    let mut s = "TinyWiiBackupManager  ›  ".to_string();

    let mount_point = &state.config.mount_point;
    if mount_point.as_os_str().is_empty() {
        s.push_str("No drive selected");
        return s;
    }

    let label = mount_point.file_name().unwrap_or(mount_point.as_os_str());
    write!(&mut s, "{}", label.display()).unwrap();

    if let Some(drive_info) = &state.drive_info {
        write!(
            &mut s,
            "  ({}/{})",
            drive_info.used_size, drive_info.total_size
        )
        .unwrap();
    }

    s
}
