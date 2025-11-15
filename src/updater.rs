// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    http,
    tasks::{BackgroundMessage, TaskProcessor},
};
use anyhow::{Context, Result};
use semver::Version;

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

pub fn check() -> Result<Option<Version>> {
    let body = http::get(VERSION_URL).context("Failed to fetch version")?;

    let latest_version_str =
        String::from_utf8(body).context("Failed to decode response body as UTF-8")?;

    let latest_version = Version::parse(latest_version_str.trim()).context(format!(
        "Failed to parse latest version from server: '{}'",
        latest_version_str
    ))?;

    let current_version =
        Version::parse(env!("CARGO_PKG_VERSION")).context("Failed to parse current version")?;

    if latest_version > current_version {
        return Ok(Some(latest_version));
    }

    Ok(None)
}

pub struct UpdateInfo {
    url: String,
    pub ui_text: String,
}

impl UpdateInfo {
    pub fn from_version(version: Version) -> UpdateInfo {
        Self {
            url: format!("{}/releases/tag/{}", env!("CARGO_PKG_REPOSITORY"), &version),
            ui_text: format!("A new version is available: {}", &version),
        }
    }

    pub fn open_url(&self) -> Result<()> {
        open::that(&self.url).map_err(Into::into)
    }
}

pub fn spawn_check_update_task(task_processor: &TaskProcessor) {
    task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "✈ Checking for updates...".to_string(),
        ))?;

        let version = check()?;

        if let Some(version) = version {
            msg_sender.send(BackgroundMessage::GotNewVersion(version))?;
        }

        Ok(())
    });
}
