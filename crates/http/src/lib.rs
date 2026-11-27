// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::Cursor,
    net::{IpAddr, SocketAddr, TcpStream},
    path::Path,
    time::Duration,
};
use twbm_common::wiiload_util;
use wiiload::WIILOAD_PORT;
use zip::ZipArchive;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

// in seconds
mod timeout {
    pub const GENERIC_DOWNLOAD: u64 = 300;
    pub const GENERIC_API: u64 = 30;
    pub const CONNECT_TO_WII: u64 = 10;
    pub const SEND_TO_WII: u64 = 600;
}

/// Downloads a file, creating the parent directory if needed
/// Skips if the file already exists
pub fn download_file(uri: &str, dest: &Path) -> Result<bool> {
    if fs::symlink_metadata(dest).is_ok() {
        // already exists, skipping
        return Ok(false);
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent)?;

    let resp = minreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .with_timeout(timeout::GENERIC_DOWNLOAD)
        .send()?;

    if resp.status_code >= 400 {
        bail!("error downloading {}: {}", uri, resp.status_code);
    }

    fs::write(dest, resp.as_bytes())?;
    Ok(true)
}

pub fn download_file_with_fallback(uri: &str, dest: &Path, fallback: &str) -> Result<bool> {
    if download_file(uri, dest).is_err() {
        download_file(fallback, dest)
    } else {
        Ok(true)
    }
}

pub fn download_and_extract_zip(uri: &str, dest: &Path) -> Result<()> {
    let resp = minreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .with_timeout(timeout::GENERIC_DOWNLOAD)
        .send()?;

    if resp.status_code >= 400 {
        bail!("error downloading {}: {}", uri, resp.status_code);
    }

    let mut cursor = Cursor::new(resp.as_bytes());
    let mut archive = ZipArchive::new(&mut cursor)?;
    fs::create_dir_all(dest)?;
    archive.extract(dest)?;

    Ok(())
}

pub fn download_and_send_via_wiiload(uri: &str, wii_ip: &str) -> Result<Vec<String>> {
    let filename = uri
        .split('/')
        .next_back()
        .and_then(|s| s.strip_suffix(".zip"))
        .context("invalid filename")?;

    let resp = minreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .with_timeout(timeout::GENERIC_DOWNLOAD)
        .send()?;

    if resp.status_code >= 400 {
        bail!("error downloading {}: {}", uri, resp.status_code);
    }

    let mut cursor = Cursor::new(resp.as_bytes());
    let (data, excluded_files) = wiiload_util::rebuild_zip(&mut cursor)?;

    let mut conn = {
        let wii_ip = wii_ip.parse::<IpAddr>()?;
        let addr = SocketAddr::new(wii_ip, WIILOAD_PORT);
        let timeout = Duration::from_secs(timeout::CONNECT_TO_WII);

        TcpStream::connect_timeout(&addr, timeout)?
    };

    wiiload::send_slice(&mut conn, filename, &data[..])?;

    Ok(excluded_files)
}

pub fn post_then_download_file(uri: &str, data: String, dest: &Path) -> Result<bool> {
    if fs::symlink_metadata(dest).is_ok() {
        // already exists, skipping
        return Ok(false);
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent)?;

    let resp = minreq::post(uri)
        .with_header("User-Agent", USER_AGENT)
        .with_timeout(timeout::GENERIC_DOWNLOAD)
        .with_body(data)
        .send()?;

    if resp.status_code >= 400 {
        bail!("error downloading {}: {}", uri, resp.status_code);
    }

    fs::write(dest, resp.as_bytes())?;

    Ok(true)
}

pub fn get_string(uri: &str) -> Result<String> {
    let bytes = minreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .with_timeout(timeout::GENERIC_API)
        .send()?
        .into_bytes();

    let s = String::from_utf8(bytes)?;

    Ok(s)
}
