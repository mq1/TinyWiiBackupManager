// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util;
use anyhow::Result;
use minreq::Response;
use smol::fs;
use std::path::Path;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub async fn get(url: String) -> Result<Vec<u8>> {
    smol::unblock(move || -> Result<Vec<u8>> {
        minreq::get(&url)
            .with_header("User-Agent", USER_AGENT)
            .send()
            .map(Response::into_bytes)
            .map_err(Into::into)
    })
    .await
}

pub async fn download_file(url: String, dest_path: &Path) -> Result<()> {
    let body = get(url).await?;
    fs::write(dest_path, body).await?;

    Ok(())
}

pub async fn download_and_extract_zip(url: String, dest_dir: &Path) -> Result<()> {
    println!(
        "Downloading and extracting \"{}\" into \"{}\"",
        &url,
        dest_dir.display()
    );

    let body = get(url).await?;
    util::extract_zip_bytes(body, dest_dir).await
}
