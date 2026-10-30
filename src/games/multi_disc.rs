// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use arrayvec::ArrayVec;
use std::{ffi::OsStr, fs::File, path::Path};

pub fn retrieve_multi_disc_files(path: &Path) -> Result<ArrayVec<File, 4>> {
    let mut files = ArrayVec::new();

    let filename = path
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    files.push(File::open(path)?);

    if let Some(filename) = filename.strip_suffix(".part0.iso") {
        let disc1_path = path.with_file_name(format!("{filename}.part1.iso"));
        files.push(File::open(&disc1_path)?);
    } else if let Some(filename) = filename.strip_suffix(".PART0.ISO") {
        let disc1_path = path.with_file_name(format!("{filename}.PART1.ISO"));
        files.push(File::open(&disc1_path)?);
    } else if let Some(filename) = filename.strip_suffix(".wbfs") {
        for i in 1..=3 {
            let wbfx_path = path.with_file_name(format!("{filename}.wbf{i}"));
            if !wbfx_path.exists() {
                break;
            }
            files.push(File::open(&wbfx_path)?);
        }
    } else if let Some(filename) = filename.strip_suffix(".WBFS") {
        for i in 1..=3 {
            let wbfx_path = path.with_file_name(format!("{filename}.WBF{i}"));
            if !wbfx_path.exists() {
                break;
            }
            files.push(File::open(&wbfx_path)?);
        }
    }

    Ok(files)
}
