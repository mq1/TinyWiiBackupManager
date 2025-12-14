// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::http;
use crate::messages::Message;
use anyhow::{Context, Result};
use egui_phosphor::regular as ph;
use semver::Version;

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

pub fn check() -> Result<Option<Version>> {
    let body = http::get_string(VERSION_URL)
        .context(format!("{} Failed to fetch version", ph::CLOUD_WARNING))?;

    let latest_version = Version::parse(&body)?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    if latest_version > current_version {
        return Ok(Some(latest_version));
    }

    Ok(None)
}

pub fn spawn_check_update_task(app: &App) {
    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "{} Checking for updates...",
            ph::CLOUD_CHECK
        )))?;

        let version = check()?;

        if let Some(version) = version {
            msg_sender.send(Message::GotNewVersion(version))?;
        }

        Ok(())
    });
}
