// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, bail};
use smol::{
    fs::{self, File},
    io::AsyncSeekExt,
    net::TcpStream,
};
use std::{
    ffi::OsStr,
    io::{Seek, SeekFrom},
    path::Path,
};
use tempfile::tempfile;
use wiiload::WIILOAD_PORT;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use windows::{download, post_then_download};

#[cfg(unix)]
mod unix;

#[cfg(unix)]
use unix::{download, post_then_download};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Downloads a file, creating the parent directory if needed
/// Skips if the file already exists
pub async fn download_file(uri: &str, dest: &Path) -> Result<()> {
    let dest_filename = dest
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent).await?;

    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();
        let dest_filename = dest_filename.to_string();
        let dest_parent = dest_parent.to_path_buf();

        move || {
            let mut out = tempfile::Builder::new()
                .prefix(&dest_filename)
                .suffix(".part")
                .rand_bytes(0)
                .tempfile_in(dest_parent)?;

            download(&uri, &mut out)?;
            out.persist(&dest)?;

            Ok(())
        }
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
    if !fs::metadata(dest).await.is_ok_and(|m| m.is_dir()) {
        bail!("{} is not a directory", dest.display());
    }

    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();

        move || {
            let mut tmp = tempfile()?;
            download(&uri, &mut tmp)?;

            tmp.rewind()?;
            let mut archive = ZipArchive::new(&mut tmp)?;
            archive.extract(dest)?;

            Ok(())
        }
    })
    .await
}

// recompresses the archive with -9 before sending
pub async fn download_and_send_via_wiiload(uri: &str, wii_ip: &str) -> Result<()> {
    let filename = uri
        .split('/')
        .next_back()
        .and_then(|s| s.strip_suffix(".zip"))
        .context("invalid filename")?;

    let file = smol::unblock({
        let uri = uri.to_string();

        move || {
            let mut og_app = tempfile()?;
            download(&uri, &mut og_app)?;
            og_app.rewind()?;
            let mut og_app = ZipArchive::new(&mut og_app)?;

            let mut recompressed_app = tempfile()?;
            let mut writer = ZipWriter::new(&mut recompressed_app);

            let opts = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(9));

            for i in 0..og_app.len() {
                let mut file = og_app.by_index(i)?;
                if file.is_dir() {
                    writer.add_directory(file.name(), opts)?;
                } else {
                    writer.start_file(file.name(), opts)?;
                    std::io::copy(&mut file, &mut writer)?;
                }
            }
            writer.finish()?;

            Ok::<_, anyhow::Error>(recompressed_app)
        }
    })
    .await?;

    // make it async
    let mut file = File::from(file);
    file.seek(SeekFrom::Start(0)).await?;

    let size = file.metadata().await?.len() as usize;
    let mut conn = TcpStream::connect((wii_ip, WIILOAD_PORT)).await?;
    wiiload::send_async(&mut conn, filename, &mut file, size).await?;

    Ok(())
}

pub async fn post_then_download_file(uri: &str, data: String, dest: &Path) -> Result<()> {
    let dest_filename = dest
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().context("invalid path")?;
    fs::create_dir_all(dest_parent).await?;

    smol::unblock({
        let uri = uri.to_string();
        let dest = dest.to_path_buf();
        let dest_filename = dest_filename.to_string();
        let dest_parent = dest_parent.to_path_buf();

        move || {
            let mut out = tempfile::Builder::new()
                .prefix(&dest_filename)
                .suffix(".part")
                .rand_bytes(0)
                .tempfile_in(dest_parent)?;

            post_then_download(&uri, data.as_bytes(), &mut out)?;
            out.persist(&dest)?;

            Ok(())
        }
    })
    .await
}
