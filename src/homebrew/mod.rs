// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::fs::unzip};
use futures::stream::{self, StreamExt, TryStreamExt};
use smol::fs;
use std::path::PathBuf;

pub mod homebrew_app;
pub mod homebrew_app_list;
pub mod meta;

pub async fn import(
    root_path: PathBuf,
    paths: Vec<PathBuf>,
    remove_sources: bool,
) -> Result<usize, Error> {
    let count = paths.len();

    stream::iter(paths)
        .map(async |p| {
            unzip(&p, &root_path).await?;
            if remove_sources {
                fs::remove_file(&p).await?;
            }
            Ok::<_, Error>(())
        })
        .buffer_unordered(8)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(count)
}
