// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use anyhow::anyhow;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const WIILOAD_MAGIC: &[u8] = b"HAXX";
const WIILOAD_MAJOR: &[u8] = &[0];
const WIILOAD_MINOR: &[u8] = &[5];
const WIILOAD_PORT: u16 = 4299;
const WIILOAD_ARGS: &[u8] = b"app.zip\0";
const WIILOAD_ARGS_LEN: &[u8] = &[0, 8];
const WIILOAD_TIMEOUT: Duration = Duration::from_secs(10);
const WIILOAD_BUF_SIZE: usize = 128 * 1024;

pub fn push(source_zip: impl AsRef<Path>, wii_ip: &str) -> Result<()> {
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

fn find_app_directory(archive: &mut ZipArchive<File>) -> Result<String> {
    let boot_dol_path = archive
        .file_names()
        .find(|name| name.ends_with("boot.dol"))
        .ok_or(anyhow!("No boot.dol found in archive"))?;

    let parent = boot_dol_path
        .rsplit_once('/')
        .ok_or(anyhow!("Failed to get parent directory"))?
        .0
        .to_string();

    Ok(parent)
}

fn create_app_zip(source_archive: &mut ZipArchive<File>, app_dir_name: &str) -> Result<Vec<u8>> {
    let mut zipped_app = Vec::new();
    let mut new_zip = ZipWriter::new(io::Cursor::new(&mut zipped_app));

    // Configure options for storing files without compression
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Collect all relevant file names first
    let relevant_files: Vec<String> = source_archive
        .file_names()
        .filter(|name| name.starts_with(app_dir_name))
        .map(|s| s.to_string())
        .collect();

    let mut app_dir_prefix = app_dir_name
        .rsplit_once('/')
        .ok_or(anyhow!("Failed to get app directory prefix"))?
        .0
        .to_string();

    app_dir_prefix.push('/');

    for file_name in &relevant_files {
        let mut file = source_archive.by_name(file_name)?;

        let new_name = file_name
            .strip_prefix(&app_dir_prefix)
            .ok_or(anyhow!("Failed to strip prefix"))?;

        new_zip.start_file(new_name, options)?;
        io::copy(&mut file, &mut new_zip)?;
    }

    new_zip.finish()?;

    Ok(zipped_app)
}

fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
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
) -> Result<()> {
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
    let mut reader = BufReader::with_capacity(WIILOAD_BUF_SIZE, compressed_data);
    io::copy(&mut reader, &mut stream)?;

    // Send arguments
    stream.write_all(WIILOAD_ARGS)?;

    stream.flush()?;

    Ok(())
}
