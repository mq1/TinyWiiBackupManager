// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

#[derive(Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Checks for a new version on GitHub.
/// Returns the URL of the new release if a newer version is found.
pub fn check_for_new_version() -> Result<Option<UpdateInfo>> {
    let repo_path = REPO_URL.trim_start_matches("https://github.com/");
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo_path);

    let mut response = ureq::get(&api_url)
        .header(
            "User-Agent",
            &format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )
        .call()
        .context("Failed to fetch latest release from GitHub")?;

    let release = response
        .body_mut()
        .read_json::<Release>()
        .context("Failed to parse release information")?;

    // Parse versions using semver
    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version = CURRENT_VERSION.trim_start_matches('v');

    let latest_ver = Version::parse(latest_version)
        .context("Failed to parse latest version from GitHub")?;
    let current_ver = Version::parse(current_version)
        .context("Failed to parse current version")?;

    if latest_ver > current_ver {
        Ok(Some(UpdateInfo {
            version: release.tag_name,
            url: release.html_url,
        }))
    } else {
        Ok(None)
    }
}
