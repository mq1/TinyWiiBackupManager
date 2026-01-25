// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};
use std::{fs, io::Cursor, path::Path};
use zip::ZipArchive;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn get(url: &str) -> Result<Vec<u8>> {
    let resp = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send()?;

    if resp.status_code != 200 {
        bail!("HTTP error: {}", resp.status_code);
    }

    let bytes = resp.into_bytes();
    Ok(bytes)
}

pub fn get_string(url: &str) -> Result<String> {
    let resp = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send()?;

    if resp.status_code != 200 {
        bail!("HTTP error: {}", resp.status_code);
    }

    let string = resp.as_str()?.to_string();
    Ok(string)
}

/// Works well for small files
pub fn download_file(url: &str, dest_path: &Path) -> Result<()> {
    let body = get(url)?;
    fs::write(dest_path, body)?;
    Ok(())
}

/// Works well for small files
pub fn download_and_extract_zip(url: &str, dest_dir: &Path) -> Result<()> {
    let body = get(url)?;
    let mut archive = ZipArchive::new(Cursor::new(body))?;
    archive.extract(dest_dir)?;
    Ok(())
}

pub fn send_form(url: &str, form: &str) -> Result<Vec<u8>> {
    let resp = minreq::post(url)
        .with_header("User-Agent", USER_AGENT)
        .with_header("Content-Type", "application/x-www-form-urlencoded")
        .with_body(form)
        .send()?;

    if resp.status_code != 200 {
        bail!("HTTP error: {}", resp.status_code);
    }

    let bytes = resp.into_bytes();
    Ok(bytes)
}
