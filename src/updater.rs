// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{TaskType, UpdateInfo, http::AGENT, tasks::TaskProcessor};
use anyhow::{Context, Result};
use semver::Version;
use slint::ToSharedString;
use std::sync::{Arc, LazyLock};

const VERSION_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/latest/download/version.txt"
);

static CURRENT_VERSION: LazyLock<Version> = LazyLock::new(|| {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Failed to parse current version")
});

pub fn check(task_processor: &Arc<TaskProcessor>) -> Result<()> {
    task_processor.spawn(Box::new(|weak| {
        weak.upgrade_in_event_loop(|handle| {
            handle.set_status("Checking for updates...".to_shared_string());
            handle.set_task_type(TaskType::CheckingForUpdates);
        })?;

        let latest_version_str = AGENT
            .get(VERSION_URL)
            .call()
            .context("Failed to fetch version")?
            .body_mut()
            .read_to_string()
            .context("Failed to decode response body as UTF-8")?;

        let latest_version = Version::parse(latest_version_str.trim()).context(format!(
            "Failed to parse latest version from server: '{latest_version_str}'"
        ))?;

        if latest_version > *CURRENT_VERSION {
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
            })?;
        }

        Ok(())
    }))
}
