// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http;
use anyhow::Result;
use semver::Version;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const URL: &str = "https://github.com/mq1/TinyWiiBackupManager/releases/latest";

#[derive(serde::Deserialize)]
struct Response {
    pub tag_name: String,
}

pub fn check() -> Result<Option<Version>> {
    if cfg!(debug_assertions) {
        return Ok(Some(Version::parse("999.0.0").unwrap()));
    }

    let resp = http::get_json::<Response>(URL)?;

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
