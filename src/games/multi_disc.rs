// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use std::{ffi::OsStr, fs::File, path::Path};

pub fn open_multi_disc_files(path: &Path) -> Result<Vec<File>> {
    let mut files = Vec::new();

    let filename = path
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    files.push(File::open(path)?);

    if let Some(filename) = filename.strip_suffix(".part0.iso") {
        for i in 1..=3 {
            let discx_path = path.with_file_name(format!("{filename}.part{i}.iso"));
            if !discx_path.exists() {
                break;
            }
            files.push(File::open(&discx_path)?);
        }
    } else if let Some(filename) = filename.strip_suffix(".PART0.ISO") {
        for i in 1..=3 {
            let discx_path = path.with_file_name(format!("{filename}.PART{i}.ISO"));
            if !discx_path.exists() {
                break;
            }
            files.push(File::open(&discx_path)?);
        }
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
