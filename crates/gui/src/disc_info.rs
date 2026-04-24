// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::DisplayedDiscInfo;
use slint::ToSharedString;
use twbm_core::disc_info::DiscInfo;

impl From<&DiscInfo> for DisplayedDiscInfo {
    fn from(disc_info: &DiscInfo) -> Self {
        DisplayedDiscInfo {
            path: disc_info.path.to_string_lossy().to_shared_string(),
            game_title: disc_info.meta.game_title().to_shared_string(),
            game_id: disc_info.meta.game_id().to_shared_string(),
            disc_number: disc_info.meta.disc_number() as i32,
            disc_version: disc_info.meta.disc_version() as i32,
            region: disc_info.meta.region().to_shared_string(),
            format: disc_info.meta.format().to_shared_string(),
            is_wii: disc_info.meta.is_wii(),
            is_gc: disc_info.meta.is_gc(),
            is_worth_scrubbing: disc_info.is_worth_scrubbing,
            crc32: format!("{:08x}", disc_info.crc32).to_shared_string(),
        }
    }
}
