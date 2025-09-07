// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::anyhow;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Seek, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;
use zip::ZipArchive;

const WIILOAD_MAGIC: &[u8] = b"HAXX";
const WIILOAD_MAJOR: &[u8] = &[0];
const WIILOAD_MINOR: &[u8] = &[5];
const WIILOAD_CHUNK_SIZE: usize = 128 * 1024;
const WIILOAD_TIMEOUT: Duration = Duration::from_secs(10);
const WIILOAD_PORT: u16 = 4299;
const WIILOAD_ARGS: &[u8] = b"app.zip\0";
const WIILOAD_ARGS_LEN: &[u8] = &[0, 8];

pub fn push(source_zip: impl AsRef<Path>, wii_ip: &str) -> anyhow::Result<()> {
    // Open the source zip file
    let source_zip = File::open(&source_zip)?;
    let mut source_archive = ZipArchive::new(source_zip)?;

    // Find the app directory containing boot.dol
    let mut app_dir_name = None;
    for i in 0..source_archive.len() {
        let file = source_archive.by_index(i)?;
        let file_path = file.name();

        // Look for boot.dol and get its parent directory
        if file_path.ends_with("boot.dol") {
            // Extract the app directory name (parent of boot.dol)
            if let Some(parent) = Path::new(file_path).parent() {
                // Get the last component of the path (the app directory name)
                if let Some(dir_name) = parent.file_name() {
                    app_dir_name = Some(dir_name.to_string_lossy().to_string());
                    break;
                }
            }
        }
    }

    let app_dir_name = app_dir_name.ok_or(anyhow!("No boot.dol found in archive"))?;

    // Create a new zip in memory with the app directory
    let mut zipped_app = Vec::new();
    {
        let cursor = Cursor::new(&mut zipped_app);
        let mut new_zip = zip::ZipWriter::new(cursor);

        // Configure options for storing files without compression
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        // Copy relevant files to the new zip
        for i in 0..source_archive.len() {
            let mut file = source_archive.by_index(i)?;
            let file_path = file.name();

            // Check if this file belongs to our app directory
            if file_path.contains(&format!("{}/", app_dir_name))
                || file_path.contains(&format!("{}\\", app_dir_name))
            {
                // Calculate the relative path starting from the app directory
                let relative_path = if let Some(pos) = file_path.rfind(&app_dir_name) {
                    &file_path[pos..]
                } else {
                    continue;
                };

                // Start a new file in the zip
                new_zip.start_file(relative_path, options)?;

                // Copy the file contents
                io::copy(&mut file, &mut new_zip)?;
            }
        }

        new_zip.finish()?;
    }

    // Store the uncompressed size for later use
    let zipped_app_len = zipped_app.len() as u32;
    let mut zipped_app = Cursor::new(zipped_app);

    // zlib-compressed data
    let mut compressed_app = Vec::<u8>::new();
    let mut cursor = Cursor::new(&mut compressed_app);
    let mut encoder = ZlibEncoder::new(&mut cursor, Compression::default());
    io::copy(&mut zipped_app, &mut encoder)?;
    encoder.flush()?;
    encoder.finish()?;

    let compressed_len = cursor.position() as u32;
    cursor.rewind()?;

    // Connect to the Wii on port 4299
    let addr = (wii_ip, WIILOAD_PORT)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Failed to resolve Wii IP"))?;

    let mut stream = TcpStream::connect_timeout(&addr, WIILOAD_TIMEOUT)?;
    stream.set_read_timeout(Some(WIILOAD_TIMEOUT))?;
    stream.set_write_timeout(Some(WIILOAD_TIMEOUT))?;

    // Send Wiiload header
    stream.write_all(WIILOAD_MAGIC)?;
    stream.write_all(WIILOAD_MAJOR)?;
    stream.write_all(WIILOAD_MINOR)?;
    stream.write_all(WIILOAD_ARGS_LEN)?;
    stream.write_all(&compressed_len.to_be_bytes())?;
    stream.write_all(&zipped_app_len.to_be_bytes())?;

    // Stream the compressed file in chunks of 128 KB
    let mut reader = BufReader::with_capacity(WIILOAD_CHUNK_SIZE, cursor);
    io::copy(&mut reader, &mut stream)?;

    // Send arguments
    stream.write_all(WIILOAD_ARGS)?;

    Ok(())
}
