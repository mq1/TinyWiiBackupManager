// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util;
use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;
use smol::{
    Timer,
    fs::{self},
    future::FutureExt,
    net::TcpStream,
};
use std::{
    io::{self, Cursor},
    path::Path,
    time::Duration,
};
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
pub async fn download_file(uri: &str, dest: &Path) -> Result<()> {
    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent).await?;

    let dest = <Box<Path>>::from(dest);
    let uri = <Box<str>>::from(uri);

    smol::unblock(move || {
        let resp = minreq::get(uri.as_ref())
            .with_header("User-Agent", USER_AGENT)
            .with_timeout(timeout::GENERIC_DOWNLOAD)
            .send()?;

        if resp.status_code >= 400 {
            bail!("error downloading {}: {}", uri, resp.status_code);
        }

        std::fs::write(dest, resp.as_bytes())?;
        Ok(())
    })
    .await
}

pub async fn download_file_with_fallback(uri: &str, dest: &Path, fallback: &str) -> Result<()> {
    if download_file(uri, dest).await.is_err() {
        download_file(fallback, dest).await
    } else {
        Ok(())
    }
}

pub async fn download_and_extract_zip(uri: &str, dest: &Path) -> Result<()> {
    let dest = <Box<Path>>::from(dest);
    let uri = <Box<str>>::from(uri);

    smol::unblock(move || {
        let resp = minreq::get(uri.as_ref())
            .with_header("User-Agent", USER_AGENT)
            .with_timeout(timeout::GENERIC_DOWNLOAD)
            .send()?;

        if resp.status_code >= 400 {
            bail!("error downloading {}: {}", uri, resp.status_code);
        }

        let mut cursor = Cursor::new(resp.as_bytes());
        let mut archive = ZipArchive::new(&mut cursor)?;
        std::fs::create_dir_all(&dest)?;
        archive.extract(dest)?;

        Ok(())
    })
    .await
}

pub async fn download_and_send_via_wiiload(uri: &str, wii_ip: &str) -> Result<Vec<String>> {
    let filename = uri
        .split('/')
        .next_back()
        .and_then(|s| s.strip_suffix(".zip"))
        .context("invalid filename")?;

    let uri = <Box<str>>::from(uri);

    let (data, excluded_files) = smol::unblock(move || {
        let resp = minreq::get(uri.as_ref())
            .with_header("User-Agent", USER_AGENT)
            .with_timeout(timeout::GENERIC_DOWNLOAD)
            .send()?;

        if resp.status_code >= 400 {
            bail!("error downloading {}: {}", uri, resp.status_code);
        }

        let mut cursor = Cursor::new(resp.as_bytes());
        util::wiiload::rebuild_zip(&mut cursor)
    })
    .await?;

    let mut conn = TcpStream::connect((wii_ip, WIILOAD_PORT))
        .or(async {
            Timer::after(Duration::from_secs(timeout::CONNECT_TO_WII)).await;
            Err(io::ErrorKind::TimedOut.into())
        })
        .await?;

    wiiload::send_slice_async(&mut conn, filename, &data[..])
        .or(async {
            Timer::after(Duration::from_secs(timeout::SEND_TO_WII)).await;
            Err(io::ErrorKind::TimedOut.into())
        })
        .await?;

    Ok(excluded_files)
}

pub async fn post_then_download_file(uri: &str, data: String, dest: &Path) -> Result<()> {
    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent).await?;

    let dest = <Box<Path>>::from(dest);
    let uri = <Box<str>>::from(uri);

    smol::unblock(move || {
        let resp = minreq::post(uri.as_ref())
            .with_header("User-Agent", USER_AGENT)
            .with_timeout(timeout::GENERIC_DOWNLOAD)
            .with_body(data)
            .send()?;

        if resp.status_code >= 400 {
            bail!("error downloading {}: {}", uri, resp.status_code);
        }

        std::fs::write(dest, resp.as_bytes())?;

        Ok(())
    })
    .await
}

pub async fn get_json<T: DeserializeOwned + Send + 'static>(uri: &'static str) -> Result<T> {
    smol::unblock(move || {
        let resp = minreq::get(uri)
            .with_header("User-Agent", USER_AGENT)
            .with_timeout(timeout::GENERIC_API)
            .send()?
            .json()?;

        Ok(resp)
    })
    .await
}
