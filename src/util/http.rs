// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use isahc::{AsyncReadResponseExt, HttpClient};
use smol::fs;
use std::{path::Path, sync::LazyLock};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

static CLIENT: LazyLock<HttpClient> = LazyLock::new(|| {
    HttpClient::builder()
        .default_header("User-Agent", USER_AGENT)
        .build()
        .unwrap()
});

pub async fn download_file(uri: &str, dest: impl AsRef<Path>) -> Result<(), Error> {
    let body = CLIENT.get_async(uri).await?.bytes().await?;
    fs::write(dest, body).await?;

    Ok(())
}
