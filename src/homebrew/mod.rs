// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::unzip};
use futures::stream::{self, StreamExt, TryStreamExt};
use std::path::PathBuf;

pub mod homebrew_app;
pub mod homebrew_app_list;
pub mod meta;

pub async fn import(root_path: PathBuf, paths: Vec<PathBuf>) -> Result<usize, Error> {
    let count = paths.len();

    stream::iter(paths)
        .map(async |p| unzip(&p, &root_path).await)
        .buffer_unordered(8)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(count)
}
