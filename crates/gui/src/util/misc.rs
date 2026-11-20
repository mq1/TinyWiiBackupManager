// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{
    fs::{self, File},
    path::PathBuf,
};
use zip::ZipArchive;

pub async fn unzip(src: impl Into<PathBuf>, dst: impl Into<PathBuf>) -> Result<()> {
    let src = src.into();
    let dst = dst.into();

    smol::unblock(move || {
        let mut file = File::open(&src)?;
        let mut zip = ZipArchive::new(&mut file)?;
        fs::create_dir_all(&dst)?;
        zip.extract(&dst)?;
        Ok(())
    })
    .await
}
