// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use std::{
    fs::{self, File},
    path::PathBuf,
    sync::LazyLock,
};
use zip::ZipArchive;

pub struct OptimalThreads {
    pub preloader: usize,
    pub processor: usize,
}

pub static OPTIMAL_THREADS: LazyLock<OptimalThreads> = LazyLock::new(|| {
    let cores = std::thread::available_parallelism().map_or(0, |n| n.get());

    let preloader = match cores {
        0 | 1 => 0,
        2..=4 => 1,
        5..=8 => 2,
        _ => 4,
    };

    let processor = cores - preloader;

    OptimalThreads {
        preloader,
        processor,
    }
});

pub async fn unzip(src: impl Into<PathBuf>, dst: impl Into<PathBuf>) -> Result<(), Error> {
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

pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    let needle_len = needle.chars().flat_map(char::to_lowercase).count();

    needle_len == 0
        || haystack.char_indices().any(|(i, _)| {
            haystack[i..]
                .chars()
                .flat_map(char::to_lowercase)
                .take(needle_len)
                .eq(needle.chars().flat_map(char::to_lowercase))
        })
}
