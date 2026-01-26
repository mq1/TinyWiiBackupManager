// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use anyhow::{Result, bail};
use iced::{Task, futures::TryFutureExt};
use std::{
    ffi::OsStr,
    fs,
    io::{self, Cursor, Read, Seek},
    net::Ipv4Addr,
    path::{Path, PathBuf},
    sync::Arc,
};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

fn get_app_files(
    archive: &mut ZipArchive<impl Read + Seek>,
) -> Result<(String, Vec<String>, Vec<String>)> {
    let Some(app_filename) = archive
        .file_names()
        .find(|f| f.ends_with("boot.dol") || f.ends_with("boot.elf"))
    else {
        bail!("Failed to find app binary")
    };

    let parent_filename = app_filename[0..app_filename.len() - 8].to_string();

    let mut app_files = Vec::new();
    let mut excluded_files = Vec::new();
    for filename in archive.file_names() {
        if filename.starts_with(&parent_filename) {
            app_files.push(filename.to_string());
        } else {
            excluded_files.push(filename.to_string());
        }
    }

    Ok((parent_filename, app_files, excluded_files))
}

fn rebuild_zip(body: Vec<u8>) -> Result<(Vec<u8>, Vec<String>)> {
    let mut archive = ZipArchive::new(Cursor::new(body))?;
    let (parent_filename, app_files, excluded_files) = get_app_files(&mut archive)?;
    let Some(app_name) = parent_filename.split('/').next_back() else {
        bail!("Failed to get app name")
    };

    let mut buf = Vec::new();
    let mut writer = ZipWriter::new(Cursor::new(&mut buf));

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    for filename in &app_files {
        let new_filename = filename.replace(&parent_filename, app_name);
        writer.start_file(new_filename, options)?;
        let mut file = archive.by_name(filename.as_str())?;
        io::copy(&mut file, &mut writer)?;
    }

    writer.finish()?;
    Ok((buf, excluded_files))
}

fn send_too_wiiload(wii_ip: &str, path: &Path) -> Result<String> {
    let wii_ip: Ipv4Addr = wii_ip.parse()?;

    let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
        bail!("Failed to get filename")
    };

    let Some(ext) = path.extension().and_then(OsStr::to_str) else {
        bail!("Failed to get extension")
    };

    let body = fs::read(path)?;

    if ext == "zip" {
        let (body, excluded_files) = rebuild_zip(body)?;
        wiiload::send(filename, body, wii_ip)?;
        Ok(format!(
            "File sent successfully. Excluded files: {}",
            excluded_files.join(", ")
        ))
    } else {
        wiiload::compress_then_send(filename, body, wii_ip)?;
        Ok("File sent successfully".to_string())
    }
}

pub fn get_send_to_wiiload_task(state: &State, path: PathBuf) -> Task<Message> {
    let wii_ip = state.config.wii_ip().clone();

    Task::perform(
        async move { send_too_wiiload(&wii_ip, &path) }.map_err(Arc::new),
        Message::GenericResult,
    )
}
