// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use isahc::HttpClient;
use smol::{
    Unblock, fs,
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

pub async fn download_file(uri: &str, dest: impl AsRef<Path>) -> Result<(), Error> {
    let dest = dest.as_ref();

    let dest_filename = dest.file_name().ok_or(Error::InvalidFilename)?;
    let dest_parent = dest.parent().ok_or(Error::InvalidFilename)?;

    let mut body = CLIENT.get_async(uri).await?.into_body();

    fs::create_dir_all(dest_parent).await?;

    let mut tmp = Unblock::new(
        tempfile::Builder::new()
            .prefix(dest_filename)
            .suffix(".part")
            .rand_bytes(0)
            .tempfile_in(dest_parent)?,
    );

    io::copy(&mut body, &mut tmp).await?;
    tmp.flush().await?;
    tmp.into_inner().await.persist(dest)?;

    Ok(())
}
