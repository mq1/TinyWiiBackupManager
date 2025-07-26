// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Context;
use semver::Version;
use serde::Deserialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

/// Information about an available update
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Check for a newer version on GitHub
pub fn check_for_new_version() -> anyhow::Result<Option<UpdateInfo>> {
    let repo = REPO.trim_start_matches("https://github.com/");
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), VERSION);

    let release = ureq::get(&url)
        .header("User-Agent", &user_agent)
        .call()
        .context("Failed to fetch latest release")?
        .body_mut()
        .read_json::<Release>()
        .context("Failed to parse release info")?;

    let latest = Version::parse(release.tag_name.trim_start_matches('v'))
        .context("Failed to parse latest version")?;
    
    let current = Version::parse(VERSION.trim_start_matches('v'))
        .context("Failed to parse current version")?;

    Ok((latest > current).then_some(UpdateInfo {
        version: release.tag_name,
        url: release.html_url,
    }))
}