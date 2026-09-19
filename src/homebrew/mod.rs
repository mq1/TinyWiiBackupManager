// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::misc::unzip;
use anyhow::Result;
use smol::{
    fs,
    stream::{self, StreamExt},
};
use std::path::PathBuf;

pub mod homebrew_app;
pub mod homebrew_state;
pub mod meta;

pub async fn import(
    root_path: PathBuf,
    paths: Vec<PathBuf>,
    remove_sources: bool,
) -> Result<usize> {
    let count = paths.len();

    stream::iter(paths)
        .then(|p| {
            let root_path = root_path.clone();

            async move {
                unzip(&p, &root_path).await?;

                if remove_sources {
                    fs::remove_file(&p).await?;
                }

                Ok::<_, anyhow::Error>(())
            }
        })
        .try_collect::<(), anyhow::Error, Vec<_>>()
        .await?;

    Ok(count)
}
