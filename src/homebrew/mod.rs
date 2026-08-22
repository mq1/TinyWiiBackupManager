// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::unzip};
use smol::{
    fs,
    stream::{self, StreamExt},
};
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
        .then(|p| {
            let root_path = root_path.clone();

            async move {
                unzip(&p, &root_path).await?;

                if remove_sources {
                    fs::remove_file(&p).await?;
                }

                Ok::<_, Error>(())
            }
        })
        .try_collect::<(), Error, Vec<_>>()
        .await?;

    Ok(count)
}
