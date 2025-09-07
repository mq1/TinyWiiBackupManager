// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::base_dir::BaseDir;
use anyhow::{Result, anyhow, bail};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::{BufReader, Cursor, Seek, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};
use tempfile::{NamedTempFile, tempdir};
use time::{Date, PrimitiveDateTime, format_description};
use zip::ZipArchive;
use zip_extensions::zip_create_from_directory;

fn parse_date_only<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let fmt = format_description::parse("[year][month][day][hour][minute][second]")
        .map_err(serde::de::Error::custom)?;
    let pdt = PrimitiveDateTime::parse(&s, &fmt).map_err(serde::de::Error::custom)?;
    Ok(pdt.date())
}

#[derive(Clone, Deserialize)]
pub struct WiiAppMeta {
    pub name: String,
    pub coder: String,
    pub version: String,
    #[serde(deserialize_with = "parse_date_only")]
    pub release_date: Date,
    pub short_description: String,
    pub long_description: String,
}

#[derive(Clone)]
pub struct WiiApp {
    pub path: PathBuf,
    pub size: u64,
    pub icon_uri: String,
    pub meta: WiiAppMeta,
    pub info_opened: bool,
    pub oscwii: String,
}

impl WiiApp {
    pub fn from_path(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            bail!("Path is not a directory");
        }

        let size = fs_extra::dir::get_size(&path)?;

        let icon_uri = format!("file://{}", path.join("icon.png").display());

        let meta_path = path.join("meta.xml");
        let meta_file = fs::read_to_string(meta_path)?;
        let meta = quick_xml::de::from_str(&meta_file)?;

        let file_name = path.file_name().ok_or(anyhow!("Failed to get file name"))?;
        let oscwii = format!(
            "https://oscwii.org/library/app/{}",
            file_name.to_string_lossy()
        );

        Ok(Self {
            path,
            size,
            icon_uri,
            meta,
            info_opened: false,
            oscwii,
        })
    }

    pub fn toggle_info(&mut self) {
        self.info_opened = !self.info_opened;
    }

    pub fn remove(&self) -> Result<()> {
        if rfd::MessageDialog::new()
            .set_title(format!("Remove {}", self.meta.name))
            .set_description(format!(
                "Are you sure you want to remove {}?",
                self.meta.name
            ))
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show()
            == rfd::MessageDialogResult::Ok
        {
            fs::remove_dir_all(&self.path)?;
        }

        Ok(())
    }
}

pub fn get_installed(base_dir: &BaseDir) -> Result<Vec<WiiApp>> {
    let apps = fs::read_dir(base_dir.apps_dir())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| WiiApp::from_path(path).ok())
        .collect();

    Ok(apps)
}

pub fn wiiload_push(source_zip: impl AsRef<Path>, wii_ip: &str) -> Result<()> {
    // Open the source zip file
    let source_zip = File::open(&source_zip)?;
    let mut source_archive = ZipArchive::new(source_zip)?;

    // Extract only from the "apps" root folder
    let source_dir = tempdir()?;
    source_archive.extract(&source_dir)?;

    // If the "apps" folder is present, cd into it
    let source_dir = if source_dir.path().join("apps").exists() {
        source_dir.path().join("apps")
    } else {
        source_dir.path().to_path_buf()
    };

    // Find first app directory
    let app_dir = fs::read_dir(&source_dir)?
        .filter_map(Result::ok)
        .find(|entry| entry.file_type().map_or(false, |ft| ft.is_dir()))
        .ok_or_else(|| anyhow!("No app folder found"))?
        .path();

    let app_name = app_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid app name"))?;

    let out_name = format!("{app_name}.zip");
    let args = format!("{out_name}\0");

    // Create a temporary zip file containing only that app folder
    let mut zipped_app = NamedTempFile::new()?;
    zip_create_from_directory(&zipped_app.path().to_path_buf(), &app_dir)?;
    let zipped_app_len = zipped_app.as_file().metadata()?.len() as u32;

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
    let addr = (wii_ip, 4299)
        .to_socket_addrs()?
        .next()
        .ok_or(anyhow!("Failed to resolve Wii IP"))?;

    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(10))?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;

    // Send Wiiload header
    stream.write_all(b"HAXX")?;
    stream.write_all(&[0])?; // major
    stream.write_all(&[5])?; // minor
    stream.write_all(&(args.len() as u16).to_be_bytes())?;
    stream.write_all(&compressed_len.to_be_bytes())?;
    stream.write_all(&zipped_app_len.to_be_bytes())?;

    // Stream the compressed file in chunks of 128 KB
    let mut reader = BufReader::with_capacity(128 * 1024, cursor);
    io::copy(&mut reader, &mut stream)?;

    // Send arguments
    stream.write_all(args.as_bytes())?;

    Ok(())
}
