// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub const GIB: f32 = 1024. * 1024. * 1024.;
pub const MIB: f32 = 1024. * 1024.;

pub fn sanitize(s: &str) -> Cow<'_, str> {
    let opts = sanitize_filename::Options {
        truncate: true,
        windows: true,
        replacement: "",
    };

    sanitize_filename::sanitize_with_options(s, opts)
}

pub fn get_threads_num() -> (usize, usize) {
    let cpus = num_cpus::get();

    let preloader_threads = match cpus {
        0..=4 => 1,
        5..=8 => 2,
        _ => 4,
    };

    let processor_threads = cpus - preloader_threads;

    (preloader_threads, processor_threads)
}

pub fn get_disc_path(dir: &Path) -> Result<PathBuf> {
    let entries = dir.read_dir()?;

    for entry in entries.filter_map(Result::ok) {
        if !entry.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }

        let path = entry.path();

        let Some(filename) = path.file_name().and_then(OsStr::to_str) else {
            continue;
        };

        if filename.starts_with('.') {
            continue;
        }

        if filename.ends_with(".part1.iso") {
            continue;
        }

        let Some(ext) = path.extension() else {
            continue;
        };

        if ext.eq_ignore_ascii_case("iso")
            || ext.eq_ignore_ascii_case("wbfs")
            || ext.eq_ignore_ascii_case("ciso")
        {
            return Ok(path);
        }
    }

    Err(anyhow!("No disc found"))
}
