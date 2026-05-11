// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::DisplayedConfig;
use slint::ToSharedString;
use twbm_core::config::Config;

impl From<&Config> for DisplayedConfig {
    fn from(config: &Config) -> Self {
        Self {
            path: config.path.to_string_lossy().to_shared_string(),
            mount_point: config
                .contents
                .mount_point
                .to_string_lossy()
                .to_shared_string(),
            sort_by: config.contents.sort_by.to_shared_string(),
            always_split: config.contents.always_split.to_shared_string(),
            gc_output_format: config.contents.gc_output_format.to_shared_string(),
            wii_output_format: config.contents.wii_output_format.to_shared_string(),
            remove_sources_apps: config.contents.remove_sources_apps.to_shared_string(),
            remove_sources_games: config.contents.remove_sources_games.to_shared_string(),
            scrub_update_partition: config.contents.scrub_update_partition.to_shared_string(),
            show_gc: config.contents.show_gc,
            show_wii: config.contents.show_wii,
            txt_codes_source: config.contents.txt_codes_source.to_shared_string(),
            view_as: config.contents.view_as.to_shared_string(),
            wii_ip: config.contents.wii_ip.to_shared_string(),
            theme_preference: config.contents.theme_preference.to_shared_string(),
            preferred_language: config.contents.preferred_language.to_shared_string(),
        }
    }
}
