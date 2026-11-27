// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

pub mod homebrew_app;
pub mod homebrew_app_manager;
pub mod homebrew_app_meta;

pub fn import(root_path: &Path, paths: &[PathBuf], remove_sources: bool) -> Result<usize> {
    for path in paths {
        let mut file = File::open(path)?;
        let mut zip = ZipArchive::new(&mut file)?;
        zip.extract(root_path)?;

        if remove_sources {
            fs::remove_file(path)?;
        }
    }

    Ok(paths.len())
}
