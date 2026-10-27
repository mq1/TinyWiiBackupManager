// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::http::get_json;
use anyhow::Result;
use semver::Version;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const URL: &str = "https://api.github.com/repos/petter1/UltimateGameBackupManager/releases/latest";

#[derive(serde::Deserialize)]
struct Response {
    // missing while the repository has no releases yet
    pub tag_name: Option<String>,
}

pub async fn check() -> Result<Option<Version>> {
    if std::env::var("UGBM_DISABLE_UPDATES").is_ok_and(|v| v == "1") {
        return Ok(None);
    }

    let resp = get_json::<Response>(URL).await?;

    let Some(tag_name) = resp.tag_name else {
        return Ok(None);
    };

    let version = match tag_name.strip_prefix('v') {
        Some(v) => v,
        None => &tag_name,
    };

    let current_version = Version::parse(CURRENT_VERSION)?;
    let version = Version::parse(version)?;

    if version > current_version {
        Ok(Some(version))
    } else {
        Ok(None)
    }
}
