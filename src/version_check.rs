// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Context;
use semver::Version;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Check for a newer version on GitHub
pub fn check_for_new_version() -> anyhow::Result<Option<UpdateInfo>> {
    let url = format!("{REPO}/releases/latest/download/version.txt");

    let latest_version = ureq::get(&url)
        .call()
        .context("Failed to fetch latest release")?
        .body_mut()
        .read_to_string()
        .context("Failed to read response body")?;

    let latest = Version::parse(&latest_version).context("Failed to parse latest version")?;
    let current = Version::parse(VERSION).context("Failed to parse current version")?;

    Ok((latest > current).then_some(UpdateInfo {
        version: format!("v{latest}"),
        url: format!("{REPO}/releases/tag/v{latest}"),
    }))
}
