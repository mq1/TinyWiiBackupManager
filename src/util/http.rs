// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use async_compat::CompatExt;
use std::{fs, path::Path};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub async fn download_file(uri: &str, dest: impl AsRef<Path>) -> Result<()> {
    let resp = bitreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .send_async()
        .compat()
        .await?;

    fs::write(dest, resp.as_bytes())?;

    Ok(())
}
