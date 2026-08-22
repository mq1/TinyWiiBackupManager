// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use smol::{
    fs,
    stream::{self, StreamExt},
};
use std::path::PathBuf;
use zip::ZipArchive;

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
        .then(move |p| {
            let root_path = root_path.clone();

            async move {
                smol::unblock({
                    let root_path = root_path.clone();
                    let p = p.clone();

                    move || {
                        let file = std::fs::File::open(&p)?;
                        let mut zip = ZipArchive::new(file)?;
                        zip.extract(&root_path)?;

                        Ok::<_, Error>(())
                    }
                })
                .await?;

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
