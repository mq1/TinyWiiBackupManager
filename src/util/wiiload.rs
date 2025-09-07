// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::anyhow;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::fs::File;
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;
use zip::ZipArchive;

const WIILOAD_MAGIC: &[u8] = b"HAXX";
const WIILOAD_MAJOR: &[u8] = &[0];
const WIILOAD_MINOR: &[u8] = &[5];
const WIILOAD_PORT: u16 = 4299;
const WIILOAD_ARGS: &[u8] = b"app.zip\0";
const WIILOAD_ARGS_LEN: &[u8] = &[0, 8];
const WIILOAD_TIMEOUT: Duration = Duration::from_secs(10);

pub fn push(source_zip: impl AsRef<Path>, wii_ip: &str) -> anyhow::Result<()> {
    // Open the source zip file
    let source_zip = File::open(&source_zip)?;
    let mut source_archive = ZipArchive::new(source_zip)?;

    // Find the app directory containing boot.dol
    let app_dir_name = find_app_directory(&mut source_archive)?;

    // Create a new zip in memory with the app directory
    let zipped_app = create_app_zip(&mut source_archive, &app_dir_name)?;
    let zipped_app_len = zipped_app.len() as u32;

    // Compress the app zip
    let compressed_app = compress_data(&zipped_app)?;
    let compressed_len = compressed_app.len() as u32;

    // Connect to the Wii and send the data
    send_to_wii(wii_ip, &compressed_app, compressed_len, zipped_app_len)?;

    Ok(())
}

fn find_app_directory(archive: &mut ZipArchive<File>) -> anyhow::Result<String> {
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let file_path = file.name();

        // Look for boot.dol and get its parent directory
        if file_path.ends_with("boot.dol") {
            // Extract the app directory name (parent of boot.dol)
            if let Some(parent) = Path::new(file_path).parent() {
                // Get the last component of the path (the app directory name)
                if let Some(dir_name) = parent.file_name() {
                    return Ok(dir_name.to_string_lossy().to_string());
                }
            }
        }
    }

    Err(anyhow!("No boot.dol found in archive"))
}

fn create_app_zip(
    source_archive: &mut ZipArchive<File>,
    app_dir_name: &str,
) -> anyhow::Result<Vec<u8>> {
    let mut zipped_app = Vec::new();

    {
        let mut new_zip = zip::ZipWriter::new(io::Cursor::new(&mut zipped_app));

        // Configure options for storing files without compression
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Copy relevant files to the new zip
        for i in 0..source_archive.len() {
            let mut file = source_archive.by_index(i)?;
            let file_path = file.name();

            // Normalize path separators for comparison
            let normalized_path = file_path.replace('\\', "/");

            // Check if this file belongs to our app directory
            if let Some(relative_path) = extract_relative_path(&normalized_path, app_dir_name) {
                // Start a new file in the zip with normalized path
                new_zip.start_file(relative_path, options)?;

                // Copy the file contents
                io::copy(&mut file, &mut new_zip)?;
            }
        }

        new_zip.finish()?;
    }

    Ok(zipped_app)
}

fn extract_relative_path(file_path: &str, app_dir_name: &str) -> Option<String> {
    // Look for the app directory in the path
    let patterns = [
        format!("{}/", app_dir_name),
        format!("/{}/", app_dir_name),
        app_dir_name.to_string(),
    ];

    for pattern in &patterns {
        if let Some(pos) = file_path.find(pattern) {
            // Get everything from the app directory onwards
            let relative = &file_path[pos..];
            // Ensure we start with the app directory name
            if relative.starts_with(app_dir_name) {
                return Some(relative.to_string());
            }
        }
    }

    None
}

fn compress_data(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut compressed = Vec::new();
    let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
    encoder.write_all(data)?;
    encoder.finish()?;
    Ok(compressed)
}

fn send_to_wii(
    wii_ip: &str,
    compressed_data: &[u8],
    compressed_len: u32,
    uncompressed_len: u32,
) -> anyhow::Result<()> {
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
    stream.write_all(WIILOAD_MAJOR)?;
    stream.write_all(WIILOAD_MINOR)?;
    stream.write_all(WIILOAD_ARGS_LEN)?;
    stream.write_all(&compressed_len.to_be_bytes())?;
    stream.write_all(&uncompressed_len.to_be_bytes())?;

    // Send the compressed data
    stream.write_all(compressed_data)?;

    // Send arguments
    stream.write_all(WIILOAD_ARGS)?;

    stream.flush()?;

    Ok(())
}
