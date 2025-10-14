// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::http::AGENT;
use crate::{MainWindow, TaskType};
use anyhow::anyhow;
use anyhow::{Result, bail};
use path_slash::PathBufExt;
use rfd::FileDialog;
use slint::{ToSharedString, Weak};
use std::fs::{self, File};
use std::io::{self, Cursor, Read, Seek, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const WIILOAD_PORT: u16 = 4299;
const WIILOAD_MAGIC: &[u8] = b"HAXX";
const WIILOAD_TIMEOUT: Duration = Duration::from_secs(10);
const WIILOAD_CHUNK_SIZE: usize = 4 * 1024;

pub fn push_file(wii_ip: &str, weak: &Weak<MainWindow>) -> Result<Vec<String>> {
    let addr = (wii_ip, WIILOAD_PORT)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Failed to resolve Wii IP: {wii_ip}"))?;

    if let Some(source) = FileDialog::new()
        .set_title("Select Wii HBC App")
        .add_filter("Wii App", &["zip", "dol", "elf"])
        .pick_file()
    {
        let file_name = source
            .file_name()
            .ok_or(anyhow!("No file name found"))?
            .to_str()
            .ok_or(anyhow!("Invalid file name"))?
            .to_string();

        let status = format!("Uploading {}...", file_name);
        weak.upgrade_in_event_loop(move |handle| {
            handle.set_status(status.to_shared_string());
            handle.set_task_type(TaskType::Wiiload);
        })?;

        if file_name.ends_with(".zip") {
            // Open the source file
            let source_zip_file = File::open(source)?;
            let mut archive = ZipArchive::new(source_zip_file)?;

            // Find the dir containing boot.dol or boot.elf
            let app_dir = find_app_dir(&mut archive)?;

            // Create new zip in memory (with deflate -9 compression)
            let (zipped_app, excluded_files) = recreate_zip(&mut archive, &app_dir)?;

            // Connect to the Wii and send the data
            send_to_wii(&addr, &zipped_app, file_name)?;

            Ok(excluded_files)
        } else {
            let bytes = fs::read(source)?;
            send_to_wii(&addr, &bytes, file_name)?;
            Ok(vec![])
        }
    } else {
        Err(anyhow!("No file selected"))
    }
}

pub fn push_oscwii(zip_url: &str, wii_ip: &str, weak: &Weak<MainWindow>) -> Result<Vec<String>> {
    let addr = (wii_ip, WIILOAD_PORT)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Failed to resolve Wii IP: {}", &wii_ip))?;

    let zip_url = zip_url.to_string();

    let status = format!("Downloading {}...", zip_url);
    weak.upgrade_in_event_loop(move |handle| {
        handle.set_status(status.to_shared_string());
        handle.set_task_type(TaskType::DownloadingFolder);
    })?;

    let buffer = AGENT
        .get(zip_url)
        .call()?
        .body_mut()
        .with_config()
        .limit(50 * 1024 * 1024) // 50MB
        .read_to_vec()?;

    let cursor = Cursor::new(buffer);
    let mut archive = ZipArchive::new(cursor)?;

    // Find the dir containing boot.dol or boot.elf
    let app_dir = find_app_dir(&mut archive)?;

    // Create new zip in memory (with deflate -9 compression)
    let (zipped_app, excluded_files) = recreate_zip(&mut archive, &app_dir)?;

    // Connect to the Wii and send the data
    send_to_wii(&addr, &zipped_app, "app.zip".to_string())?;

    Ok(excluded_files)
}

fn find_app_dir(archive: &mut ZipArchive<impl Read + Seek>) -> Result<PathBuf> {
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let path = file.mangled_name();

        if let Some(file_name) = path.file_name()
            && (file_name == "boot.dol" || file_name == "boot.elf")
            && let Some(parent) = path.parent()
        {
            return Ok(parent.to_path_buf());
        }
    }

    bail!("No app directory found in zip");
}

fn recreate_zip(
    archive: &mut ZipArchive<impl Read + Seek>,
    app_dir: &Path,
) -> Result<(Vec<u8>, Vec<String>)> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    let mut writer = ZipWriter::new(&mut cursor);
    let mut excluded_files = Vec::new();

    let app_name = app_dir
        .file_name()
        .ok_or(anyhow!("No app name found"))?
        .to_string_lossy()
        .to_string();

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9));

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let path = file.mangled_name();

        // only add files that are in the app directory
        if path.starts_with(app_dir) {
            let rel_path = path.strip_prefix(app_dir)?;
            let final_path = Path::new(&app_name)
                .join(rel_path)
                .to_slash()
                .ok_or(anyhow!("Invalid path"))?
                .to_string();

            writer.start_file(final_path, options)?;
            io::copy(&mut file, &mut writer)?;
        } else {
            excluded_files.push(path.to_string_lossy().to_string());
        }
    }

    writer.finish()?;
    Ok((buffer, excluded_files))
}

fn send_to_wii(addr: &SocketAddr, compressed_data: &[u8], mut file_name: String) -> Result<()> {
    file_name += "\0";

    // Connect to the Wii
    let mut stream = TcpStream::connect_timeout(addr, WIILOAD_TIMEOUT)?;
    stream.set_read_timeout(Some(WIILOAD_TIMEOUT))?;
    stream.set_write_timeout(Some(WIILOAD_TIMEOUT))?;

    // Send Wiiload header
    stream.write_all(WIILOAD_MAGIC)?;
    stream.write_all(&[0, 5, 0, file_name.len() as u8])?;
    stream.write_all(&(compressed_data.len() as u32).to_be_bytes())?;
    stream.write_all(&[0u8; 4])?; // signal that the data is not compressed

    // Send the data in chunks
    for chunk in compressed_data.chunks(WIILOAD_CHUNK_SIZE) {
        stream.write_all(chunk)?;
    }

    // Send arguments
    stream.write_all(file_name.as_bytes())?;

    stream.flush()?;
    Ok(())
}
