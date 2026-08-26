// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use isahc::HttpClient;
use smol::{
    fs::{self, File},
    io::{self, AsyncWriteExt},
};
use std::{path::Path, sync::LazyLock};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

static CLIENT: LazyLock<HttpClient> = LazyLock::new(|| {
    HttpClient::builder()
        .default_header("User-Agent", USER_AGENT)
        .build()
        .unwrap()
});

/// Downloads a file, creating the parent directory if needed
/// Skips if the file already exists
pub async fn download_file(uri: &str, dest: impl AsRef<Path>) -> Result<(), Error> {
    let dest = dest.as_ref();

    if fs::symlink_metadata(&dest).await.is_ok() {
        println!("INFO: {} already exists, skipping", dest.display());
        return Ok(());
    }

    let dest_parent = dest.parent().ok_or(Error::InvalidFilename)?;

    let mut body = CLIENT.get_async(uri).await?.into_body();

    fs::create_dir_all(dest_parent).await?;

    let tmp_path = dest.with_added_extension("part");
    let mut tmp = File::create(&tmp_path).await?;

    if let Err(e) = io::copy(&mut body, &mut tmp).await {
        let _ = tmp.flush().await;
        drop(tmp);
        let _ = fs::remove_file(&tmp_path).await;
        Err(e.into())
    } else {
        tmp.flush().await?;
        drop(tmp);
        fs::rename(&tmp_path, dest).await?;
        Ok(())
    }
}

pub async fn download_file_with_fallback(
    uri: &str,
    dest: impl AsRef<Path>,
    fallback: &str,
) -> Result<(), Error> {
    if download_file(uri, &dest).await.is_err() {
        download_file(fallback, dest).await
    } else {
        Ok(())
    }
}
