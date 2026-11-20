// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::message::ConfigMessage;
use anyhow::Result;
use iced::{Task, futures::TryFutureExt};
use smol::fs;
use std::path::{Path, PathBuf};
use twbm_common::config_contents::{
    ConfigContents, PreferredLanguage, SortBy, ThemePreference, TxtCodesSource, ViewAs,
};

pub mod message;
pub mod ui;
pub mod update;

#[derive(Debug, Clone, Default)]
pub struct Config {
    path: PathBuf,
    pub contents: ConfigContents,
}

impl Config {
    pub async fn load(data_dir: &'static Path) -> Self {
        let path = data_dir.join("config.json");

        let contents = fs::read_to_string(&path)
            .await
            .ok()
            .and_then(|s| serde_json::from_str::<ConfigContents>(&s).ok())
            .map(|mut contents| async move {
                // invalidate mount point if it doesn't exist anymore
                if let Some(mount_point) = &contents.mount_point
                    && (mount_point.as_os_str().is_empty()
                        || fs::read_dir(mount_point).await.is_err())
                {
                    contents.mount_point = None;
                }

                contents
            });

        let contents = match contents {
            Some(contents) => contents.await,
            None => ConfigContents::default(),
        };

        Self { path, contents }
    }

    async fn write_inner(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.contents)?;
        fs::write(&self.path, contents).await?;
        Ok(())
    }

    fn write(&self) -> Task<ConfigMessage> {
        let config = self.clone();

        Task::perform(
            async move { config.write_inner().await }.map_err(|e| e.to_string()),
            ConfigMessage::Written,
        )
    }

    pub fn set_mount_point(&mut self, path: PathBuf) -> Task<ConfigMessage> {
        let is_new = self.contents.known_drives.insert(path.clone());
        self.contents.mount_point = Some(path);

        if is_new {
            Task::batch([self.write(), Task::done(ConfigMessage::NewMountPoint)])
        } else {
            self.write()
        }
    }
}
