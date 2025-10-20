// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, http::AGENT, tasks::BackgroundMessage};
use anyhow::{Context, Result};
use semver::Version;
use std::fmt;

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

pub fn check() -> Result<Option<UpdateInfo>> {
    let latest_version_str = AGENT
        .get(VERSION_URL)
        .call()
        .context("Failed to fetch version")?
        .body_mut()
        .read_to_string()
        .context("Failed to decode response body as UTF-8")?;

    let latest_version = Version::parse(latest_version_str.trim()).context(format!(
        "Failed to parse latest version from server: '{}'",
        latest_version_str
    ))?;

    let current_version =
        Version::parse(env!("CARGO_PKG_VERSION")).context("Failed to parse current version")?;

    if latest_version > current_version {
        return Ok(Some(UpdateInfo(latest_version)));
    }

    Ok(None)
}

pub struct UpdateInfo(Version);

impl fmt::Display for UpdateInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A new version is available: {}", self.0)
    }
}

impl UpdateInfo {
    pub fn open_url(&self) -> Result<()> {
        let url = format!("{}/releases/tag/{}", env!("CARGO_PKG_REPOSITORY"), &self.0);
        open::that(&url)?;
        Ok(())
    }
}

pub fn spawn_check_update_task(app: &App) {
    app.task_processor.spawn(move |status, msg_sender| {
        *status.lock() = "✈ Checking for updates...".to_string();

        let update_info = check()?;

        if let Some(update_info) = update_info {
            msg_sender.send(BackgroundMessage::NotifyInfo(update_info.to_string()));
            msg_sender.send(BackgroundMessage::GotUpdateInfo(update_info))?;
        }

        Ok(())
    });
}
