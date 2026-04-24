// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use nod::{
    common::{Compression, Format},
    write::FormatOptions,
};
use std::{borrow::Cow, ffi::OsStr, fs::File, num::NonZeroUsize, path::Path};
use zip::ZipArchive;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB
pub const HEADER_SIZE: usize = 131_072;

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

pub fn install_zips(
    root_dir: &Path,
    zips: impl IntoIterator<Item = impl AsRef<Path>>,
) -> Result<()> {
    for zip in zips {
        let mut f = File::open(zip)?;
        let mut archive = ZipArchive::new(&mut f)?;
        archive.extract(root_dir)?
    }

    Ok(())
}

pub fn str_to_format(s: &str) -> Option<Format> {
    match s.to_ascii_lowercase().as_str() {
        "iso" | "gcm" => Some(Format::Iso),
        "ciso" => Some(Format::Ciso),
        "gcz" => Some(Format::Gcz),
        "nfs" => Some(Format::Nfs),
        "rvz" => Some(Format::Rvz),
        "wbfs" => Some(Format::Wbfs),
        "wia" => Some(Format::Wia),
        "tgc" => Some(Format::Tgc),
        _ => None,
    }
}

pub fn ext_to_format(ext: &OsStr) -> Option<Format> {
    ext.to_str().and_then(str_to_format)
}

pub fn format_to_opts(format: Format) -> FormatOptions {
    match format {
        Format::Rvz => FormatOptions {
            format: Format::Rvz,
            compression: Compression::Zstandard(19),
            block_size: Format::Rvz.default_block_size(),
        },
        format => FormatOptions::new(format),
    }
}
