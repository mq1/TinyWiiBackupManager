// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::DisplayedConfig;
use slint::{ModelRc, ToSharedString, VecModel};
use std::{path::PathBuf, rc::Rc};
use twbm_core::config::{Config, ConfigContents};

impl From<&Config> for DisplayedConfig {
    fn from(config: &Config) -> Self {
        let known_drives = config
            .contents
            .known_drives
            .iter()
            .map(|d| d.to_string_lossy().to_shared_string())
            .collect::<VecModel<_>>();

        DisplayedConfig {
            path: config.path.to_string_lossy().to_shared_string(),
            mount_point: config
                .contents
                .mount_point
                .to_string_lossy()
                .to_shared_string(),
            sort_by: config.contents.sort_by.into(),
            always_split: config.contents.always_split,
            gc_output_format: config.contents.gc_output_format.into(),
            wii_output_format: config.contents.wii_output_format.into(),
            remove_sources_apps: config.contents.remove_sources_apps,
            remove_sources_games: config.contents.remove_sources_games,
            scrub_update_partition: config.contents.scrub_update_partition,
            show_gc: config.contents.show_gc,
            show_wii: config.contents.show_wii,
            txt_codes_source: config.contents.txt_codes_source.into(),
            view_as: config.contents.view_as.into(),
            wii_ip: config.contents.wii_ip.to_shared_string(),
            theme_preference: config.contents.theme_preference.into(),
            known_drives: ModelRc::from(Rc::new(known_drives)),
        }
    }
}

impl From<&DisplayedConfig> for Config {
    fn from(config: &DisplayedConfig) -> Self {
        let path = PathBuf::from(&config.path);

        let contents = ConfigContents {
            always_split: config.always_split,
            mount_point: PathBuf::from(&config.mount_point),
            remove_sources_apps: config.remove_sources_apps,
            remove_sources_games: config.remove_sources_games,
            scrub_update_partition: config.scrub_update_partition,
            wii_ip: config.wii_ip.to_string(),
            show_wii: config.show_wii,
            show_gc: config.show_gc,
            known_drives: Vec::new(),
            wii_output_format: config.wii_output_format.try_into().unwrap_or_default(),
            gc_output_format: config.gc_output_format.try_into().unwrap_or_default(),
            sort_by: config.sort_by.try_into().unwrap_or_default(),
            view_as: config.view_as.try_into().unwrap_or_default(),
            txt_codes_source: config.txt_codes_source.try_into().unwrap_or_default(),
            theme_preference: config.theme_preference.try_into().unwrap_or_default(),
        };

        Self { path, contents }
    }
}
