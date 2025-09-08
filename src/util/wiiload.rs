// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use anyhow::anyhow;
use std::fs::File;
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const WIILOAD_PORT: u16 = 4299;
const WIILOAD_MAGIC: &[u8] = b"HAXX";
const WIILOAD_ARGS: &[u8] = b"app.zip\0";
const WIILOAD_VER_LEN: &[u8] = &[0, 5, 0, 8]; // 8 is app.zip\0 len
const WIILOAD_TIMEOUT: Duration = Duration::from_secs(10);
const WIILOAD_CHUNK_SIZE: usize = 4 * 1024;

pub fn push(source_zip: impl AsRef<Path>, wii_ip: &str) -> Result<()> {
    let source_zip_name = source_zip
        .as_ref()
        .with_extension("")
        .file_name()
        .ok_or(anyhow!("Failed to get file name"))?
        .to_string_lossy()
        .to_string();

    // Extract to temporary directory
    let temp_dir = TempDir::new()?;
    let source_zip_file = File::open(&source_zip)?;
    let mut archive = ZipArchive::new(source_zip_file)?;
    archive.extract(&temp_dir)?;

    // Find the app directory containing boot.dol
    let (app_dir, app_name) = find_app_directory(&temp_dir, &source_zip_name)?;

    // Create new zip from the app directory
    let zipped_app = create_app_zip_from_dir(&app_dir, &app_name)?;

    // Connect to the Wii and send the data
    send_to_wii(wii_ip, &zipped_app)?;

    Ok(())
}

fn find_app_directory(temp_dir: &TempDir, fallback_name: &str) -> Result<(PathBuf, String)> {
    let boot_dol = WalkDir::new(temp_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name() == "boot.dol")
        .ok_or(anyhow!("No boot.dol found in archive"))?;

    let parent = boot_dol
        .path()
        .parent()
        .ok_or(anyhow!("Unable to find boot.dol parent dir"))?;

    if parent == temp_dir.path() {
        return Ok((parent.to_path_buf(), fallback_name.to_string()));
    }

    let app_name = parent
        .file_name()
        .ok_or(anyhow!("Unable to find app name"))?
        .to_string_lossy();

    Ok((parent.to_path_buf(), app_name.to_string()))
}

fn create_app_zip_from_dir(app_dir: &Path, app_name: &str) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut cursor = io::Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(&mut cursor);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9));

    // Walk through the app directory and add all files
    for entry in WalkDir::new(app_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let rel_path = path.strip_prefix(app_dir)?;
        let rel_path = format!("{}/{}", app_name, rel_path.display());

        let mut file = File::open(path)?;
        zip.start_file(rel_path, options)?;
        io::copy(&mut file, &mut zip)?;
    }

    zip.finish()?;
    Ok(buffer)
}

fn send_to_wii(wii_ip: &str, compressed_data: &[u8]) -> Result<()> {
    // Connect to the Wii
    let addr = (wii_ip, WIILOAD_PORT)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Failed to resolve Wii IP: {}", wii_ip))?;

    let mut stream = TcpStream::connect_timeout(&addr, WIILOAD_TIMEOUT)?;
    stream.set_read_timeout(Some(WIILOAD_TIMEOUT))?;
    stream.set_write_timeout(Some(WIILOAD_TIMEOUT))?;

    // Send Wiiload header
    stream.write_all(WIILOAD_MAGIC)?;
    stream.write_all(WIILOAD_VER_LEN)?;
    stream.write_all(&(compressed_data.len() as u32).to_be_bytes())?;
    stream.write_all(&[0u8; 4])?; // signal that the data is already compressed

    // Send the compressed data in chunks
    for chunk in compressed_data.chunks(WIILOAD_CHUNK_SIZE) {
        stream.write_all(chunk)?;
    }

    // Send arguments
    stream.write_all(WIILOAD_ARGS)?;

    stream.flush()?;

    Ok(())
}
