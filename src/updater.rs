// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{UpdateInfo, http::AGENT, tasks};
use anyhow::{Context, Result};
use slint::ToSharedString;

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

pub fn check() -> Result<()> {
    tasks::spawn(|weak| {
        let latest_version = AGENT
            .get(VERSION_URL)
            .call()
            .context("Failed to fetch version")?
            .body_mut()
            .read_to_string()
            .context("Failed to decode response body as UTF-8")?;

        let latest_version = latest_version.trim();

        if latest_version != env!("CARGO_PKG_VERSION") {
            let info = UpdateInfo {
                version: latest_version.to_shared_string(),
                url: format!(
                    "{}/releases/tag/{}",
                    env!("CARGO_PKG_REPOSITORY"),
                    latest_version
                )
                .to_shared_string(),
            };

            weak.upgrade_in_event_loop(|handle| {
                handle.set_update_info(info);
                handle.invoke_show_update();
            })?;
        }

        Ok(())
    })
}
