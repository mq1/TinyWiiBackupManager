// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};
use http_req::{request::Request, uri::Uri};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};
use tempfile::tempfile;
use zip::ZipArchive;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn get(uri: &str, body_size: Option<usize>) -> Result<Vec<u8>> {
    let uri = Uri::try_from(uri)?;

    let mut body = if let Some(size) = body_size {
        Vec::with_capacity(size)
    } else {
        Vec::new()
    };

    let res = Request::new(&uri)
        .header("User-Agent", USER_AGENT)
        .send(&mut body)?;

    let status_code = res.status_code();
    if !status_code.is_success() {
        bail!("HTTP request failed with status code {}", status_code);
    }

    Ok(body)
}

pub fn download_file(uri: &str, dest: &Path) -> Result<()> {
    let uri = Uri::try_from(uri)?;

    let mut writer = BufWriter::new(File::create(dest)?);

    let res = Request::new(&uri)
        .header("User-Agent", USER_AGENT)
        .send(&mut writer)?;

    let status_code = res.status_code();
    if !status_code.is_success() {
        bail!("HTTP request failed with status code {}", status_code);
    }

    Ok(())
}

pub fn download_into_file(uri: &str, file: &File) -> Result<()> {
    let uri = Uri::try_from(uri)?;

    let mut writer = BufWriter::new(file);

    let res = Request::new(&uri)
        .header("User-Agent", USER_AGENT)
        .send(&mut writer)?;

    let status_code = res.status_code();
    if !status_code.is_success() {
        bail!("HTTP request failed with status code {}", status_code);
    }

    Ok(())
}

pub fn download_and_extract_zip(uri: &str, dest_dir: &Path) -> Result<()> {
    let uri = Uri::try_from(uri)?;

    let tmp = tempfile()?;

    {
        let mut writer = BufWriter::new(&tmp);

        let res = Request::new(&uri)
            .header("User-Agent", USER_AGENT)
            .send(&mut writer)?;

        let status_code = res.status_code();
        if !status_code.is_success() {
            bail!("HTTP request failed with status code {}", status_code);
        }
    }

    {
        let reader = BufReader::new(&tmp);
        let mut zip = ZipArchive::new(reader)?;
        zip.extract(dest_dir)?;
    }

    Ok(())
}
