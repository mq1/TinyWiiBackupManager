// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util;
use anyhow::Result;
use smol::fs;
use std::path::Path;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub async fn get(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::new();

    let body = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?
        .bytes()
        .await?;

    Ok(body.to_vec())
}

pub async fn download_file(url: &str, dest_path: &Path) -> Result<()> {
    let body = get(url).await?;
    fs::write(dest_path, body).await?;

    Ok(())
}

pub async fn download_and_extract_zip(uri: &str, dest_dir: &Path) -> Result<()> {
    println!(
        "Downloading and extracting \"{}\" into \"{}\"",
        uri,
        dest_dir.display()
    );

    let body = get(uri).await?;
    util::extract_zip_bytes(body, dest_dir).await
}
