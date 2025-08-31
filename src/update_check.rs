// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::messages::BackgroundMessage;
use anyhow::{Context, Result};
use const_format::concatcp;
use semver::Version;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");
const VERSION_URL: &str = concatcp!(REPO, "/releases/latest/download/version.txt");

/// Holds information about a newer, available version of the application.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Checks for a newer version of the application.
pub fn check_for_new_version() -> Result<Option<UpdateInfo>> {
    let response = minreq::get(VERSION_URL)
        .send()
        .context("Failed to fetch version")?;

    let latest_version_str = response
        .as_str()
        .context("Failed to decode response body as UTF-8")?;

    let latest_version = Version::parse(latest_version_str.trim()).context(format!(
        "Failed to parse latest version from server: '{latest_version_str}'"
    ))?;

    let current_version =
        Version::parse(VERSION).context(format!("Failed to parse current version: '{VERSION}'"))?;

    if latest_version > current_version {
        Ok(Some(UpdateInfo {
            version: format!("v{latest_version}"),
            url: format!("{REPO}/releases/tag/v{latest_version}"),
        }))
    } else {
        Ok(None)
    }
}

pub fn spawn_check_for_new_version_task(app: &App) {
    app.task_processor.spawn_task(move |ui_sender| {
        let update_info = check_for_new_version()?;
        let _ = ui_sender.send(BackgroundMessage::GotUpdate(update_info));
        Ok(())
    });
}
