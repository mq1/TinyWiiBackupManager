// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{fs, io::Cursor, path::Path};
use zip::ZipArchive;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn get(url: &str) -> Result<Box<[u8]>> {
    let body = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send()?
        .into_bytes()
        .into_boxed_slice();

    Ok(body)
}

pub fn download_file(url: &str, dest_path: &Path) -> Result<()> {
    let body = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send()?
        .into_bytes();

    fs::write(dest_path, body)?;

    Ok(())
}

pub fn download_and_extract_zip(uri: &str, dest_dir: &Path) -> Result<()> {
    let body = minreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .send()?
        .into_bytes()
        .into_boxed_slice();

    let cursor = Cursor::new(body);

    let mut zip = ZipArchive::new(cursor)?;
    zip.extract(dest_dir)?;

    Ok(())
}
