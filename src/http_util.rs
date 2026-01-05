// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow, bail};
use async_zip::base::read::mem::ZipFileReader;
use smol::{
    fs::{self, File},
    io::{self, BufWriter},
};
use std::path::Path;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn get(url: &str) -> Result<Box<[u8]>> {
    let body = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send()?
        .into_bytes()
        .into_boxed_slice();

    Ok(body)
}

pub async fn download_file(url: &str, dest_path: &Path) -> Result<()> {
    let body = minreq::get(url)
        .with_header("User-Agent", USER_AGENT)
        .send()?
        .into_bytes();

    fs::write(dest_path, body).await?;

    Ok(())
}

pub async fn download_and_extract_zip(uri: &str, dest_dir: &Path) -> Result<()> {
    let body = minreq::get(uri)
        .with_header("User-Agent", USER_AGENT)
        .send()?
        .into_bytes();

    let zip = ZipFileReader::new(body).await?;
    for (i, entry) in zip.file().entries().iter().enumerate() {
        let filename = entry.filename().as_str()?;
        let is_dir = entry.dir()?;

        let rel_path = Path::new(filename);
        let dest_path = dest_dir.join(rel_path).canonicalize()?;

        if !dest_path.starts_with(dest_dir) {
            bail!("Directory traversal attempt detected");
        }

        if is_dir {
            fs::create_dir_all(&dest_path).await?;
            continue;
        }

        let parent = dest_path
            .parent()
            .ok_or(anyhow!("Failed to get parent dir"))?;

        let mut reader = zip.reader_without_entry(i).await?;

        fs::create_dir_all(parent).await?;
        let file = File::create(dest_path).await?;
        let mut writer = BufWriter::new(file);
        io::copy(&mut reader, &mut writer).await?;
    }

    Ok(())
}
