// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::http;
use crate::messages::Message;
use anyhow::{Context, Result};
use semver::Version;

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

pub fn check() -> Result<Option<Version>> {
    let body = http::get_string(VERSION_URL).context("Failed to fetch version")?;

    let latest_version = Version::parse(&body)?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    if latest_version > current_version {
        return Ok(Some(latest_version));
    }

    Ok(None)
}

pub struct UpdateInfo {
    pub url: String,
    pub ui_text: String,
}

impl UpdateInfo {
    pub fn from_version(version: Version) -> UpdateInfo {
        Self {
            url: format!("{}/releases/tag/{}", env!("CARGO_PKG_REPOSITORY"), &version),
            ui_text: format!("A new version is available: {}", &version),
        }
    }
}

pub fn spawn_check_update_task(app: &App) {
    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(
            "✈ Checking for updates...".to_string(),
        ))?;

        let version = check()?;

        if let Some(version) = version {
            msg_sender.send(Message::GotNewVersion(version))?;
        }

        Ok(())
    });
}
