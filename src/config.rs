// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Config, ConfigContents, SortBy};
use anyhow::Result;
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::{fs, path::Path};

impl Config {
    #[must_use]
    pub fn load(data_dir: &Path) -> Self {
        let path = data_dir.join("config.json");
        let bytes = fs::read(&path).unwrap_or_default();
        let mut contents = serde_json::from_slice::<ConfigContents>(&bytes)
            .unwrap_or(ConfigContents::my_default());

        // Invalidate invalid mount_point
        if !contents.is_mount_point_valid() {
            contents.mount_point = SharedString::new();
        }

        Self {
            path: path.to_string_lossy().to_shared_string(),
            contents,
        }
    }

    /// Writes the config into config.json
    pub fn write(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.contents)?;
        fs::write(&self.path, &bytes)?;

        Ok(())
    }

    /// Returns true if the notification should be shown
    pub fn check_mount_point(&mut self) -> bool {
        if !self.contents.is_mount_point_valid() {
            return false;
        }

        let is_new = self
            .contents
            .known_drives
            .iter()
            .all(|p| p != self.contents.mount_point.as_str());

        if is_new {
            self.contents
                .known_drives
                .as_any()
                .downcast_ref::<VecModel<SharedString>>()
                .unwrap()
                .push(self.contents.mount_point.clone());

            let _ = self.write();
        }

        is_new
    }
}

impl ConfigContents {
    #[must_use]
    pub fn is_mount_point_valid(&self) -> bool {
        if self.mount_point.is_empty() {
            return false;
        }

        Path::new(&self.mount_point).exists()
    }

    #[must_use]
    pub fn my_default() -> Self {
        Self {
            always_split: false,
            mount_point: SharedString::new(),
            remove_sources_apps: false,
            remove_sources_games: false,
            scrub_update_partition: false,
            sort_by: SortBy::NameAscending,
            view_as_table: false,
            wii_ip: "192.168.1.100".to_shared_string(),
            txt_codes_source: "web_archive".to_shared_string(),
            theme_preference: "system".to_shared_string(),
            wii_output_format: "wbfs".to_shared_string(),
            gc_output_format: "iso".to_shared_string(),
            show_wii: true,
            show_gc: true,
            known_drives: ModelRc::new(VecModel::default()),
        }
    }
}
