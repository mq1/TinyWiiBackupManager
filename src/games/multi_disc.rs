// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use std::{ffi::OsStr, fs::File, path::Path};

const SUFFIXES: [(&str, &str, &str); 6] = [
    (".part0.iso", ".part", ".iso"),
    (".PART0.ISO", ".PART", ".ISO"),
    (".wbfs", ".wbf", ""),
    (".WBFS", ".WBF", ""),
    (".iso", ".iso.", ""),
    (".ISO", ".ISO.", ""),
];

pub fn open_multi_disc_files(path: &Path) -> Result<Vec<File>> {
    let mut files = Vec::with_capacity(2);

    let filename = path
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    files.push(File::open(path)?);

    for (suffix, mid, end) in SUFFIXES {
        if let Some(prefix) = filename.strip_suffix(suffix) {
            for i in 1.. {
                let discx_path = path.with_file_name(format!("{prefix}{mid}{i}{end}"));
                if !discx_path.exists() {
                    break;
                }
                files.push(File::open(&discx_path)?);
            }
            break;
        }
    }

    Ok(files)
}
