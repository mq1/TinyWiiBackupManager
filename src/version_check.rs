// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;

// Get version and repository information from Cargo environment variables
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

// Struct to deserialize the GitHub release API response
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
}

/// Information about an available update.
#[derive(Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

/// Checks for a new version on GitHub.
pub fn check_for_new_version() -> Result<Option<UpdateInfo>> {
    // Construct the GitHub API URL for the latest release
    let repo_path = REPO_URL.trim_start_matches("https://github.com/");
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo_path);

    // Send a GET request to the GitHub API
    let mut response = ureq::get(&api_url)
        .header(
            "User-Agent",
            &format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )
        .call()
        .context("Failed to fetch latest release from GitHub")?;

    // Deserialize the JSON response into a Release struct
    let release = response
        .body_mut()
        .read_json::<Release>()
        .context("Failed to parse release information")?;

    // Extract and parse version strings (removing 'v' prefix)
    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version = CURRENT_VERSION.trim_start_matches('v');

    // Parse versions using the semver crate for accurate comparison
    let latest_ver = Version::parse(latest_version)
        .context("Failed to parse latest version from GitHub")?;
    let current_ver = Version::parse(current_version)
        .context("Failed to parse current version")?;

    // Compare versions and return UpdateInfo if a newer version is available
    Ok((latest_ver > current_ver).then(|| UpdateInfo {
        version: release.tag_name,
        url: release.html_url,
    }))
}