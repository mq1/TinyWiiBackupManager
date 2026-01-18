// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util;
use anyhow::Result;
use minreq::Response;
use smol::{
    channel::{Sender, bounded, unbounded},
    fs,
};
use std::{path::Path, sync::LazyLock};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

type Job = (String, Sender<Result<Vec<u8>>>);

static HTTP_WORKER: LazyLock<Sender<Job>> = LazyLock::new(|| {
    let (tx, rx) = unbounded::<Job>();

    std::thread::spawn(move || {
        while let Ok((url, reply_tx)) = rx.recv_blocking() {
            let res = minreq::get(&url)
                .with_header("User-Agent", USER_AGENT)
                .send()
                .map(Response::into_bytes)
                .map_err(Into::into);

            let _ = reply_tx.send_blocking(res);
        }
    });

    tx
});

pub async fn get(url: String) -> Result<Vec<u8>> {
    let (reply_tx, reply_rx) = bounded(1);
    HTTP_WORKER.send((url, reply_tx)).await?;
    reply_rx.recv().await?
}

pub async fn download_file(url: String, dest_path: &Path) -> Result<()> {
    let body = get(url).await?;
    fs::write(dest_path, body).await?;

    Ok(())
}

pub async fn download_and_extract_zip(url: String, dest_dir: &Path) -> Result<()> {
    println!(
        "Downloading and extracting \"{}\" into \"{}\"",
        &url,
        dest_dir.display()
    );

    let body = get(url).await?;
    util::extract_zip_bytes(body, dest_dir).await
}
