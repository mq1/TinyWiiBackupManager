// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use semver::Version;
use twbm_http::get_string;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const URL: &str = "https://api.github.com/repos/mq1/TinyWiiBackupManager/releases/latest";

#[derive(serde::Deserialize)]
struct Response {
    pub tag_name: String,
}

pub async fn check() -> Result<Option<Version>> {
    if std::env::var("TWBM_DISABLE_UPDATES").is_ok_and(|v| v == "1") {
        return Ok(None);
    }

    let resp = get_string(URL)?;
    let resp = serde_json::from_str::<Response>(&resp)?;

    let version = match resp.tag_name.strip_prefix('v') {
        Some(v) => v,
        None => &resp.tag_name,
    };

    let current_version = Version::parse(CURRENT_VERSION)?;
    let version = Version::parse(version)?;

    if version > current_version {
        Ok(Some(version))
    } else {
        Ok(None)
    }
}
