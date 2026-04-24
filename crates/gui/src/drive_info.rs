// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    DisplayedDriveInfo,
    util::{GIB, MIB},
};
use slint::ToSharedString;
use twbm_core::drive_info::DriveInfo;

impl DisplayedDriveInfo {
    pub fn new(drive_info: &DriveInfo) -> Self {
        Self {
            label: drive_info.label.to_shared_string(),
            fs_kind: drive_info.fs_kind.to_shared_string(),
            used_gib: drive_info.used_bytes as f32 / GIB,
            total_gib: drive_info.total_bytes as f32 / GIB,
            games_gib: drive_info.games_bytes as f32 / GIB,
            apps_mib: drive_info.apps_bytes as f32 / MIB,
        }
    }
}
