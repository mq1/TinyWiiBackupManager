// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use async_zip::base::read::seek::ZipFileReader;
use futures::{
    future,
    stream::{self, StreamExt},
};
use path_clean::PathClean;
use size::Size;
use smol::{
    fs::{self, File},
    io::{self, AsyncWriteExt, BufReader, BufWriter},
};
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

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

pub async fn get_dir_size(path: &Path) -> Size {
    let mut size = 0;

    let mut entries = vec![path.to_path_buf()];
    while let Some(entry) = entries.pop() {
        let Ok(meta) = fs::symlink_metadata(&entry).await else {
            continue;
        };

        if meta.is_file() {
            size += meta.len();
        } else if meta.is_dir()
            && let Ok(new_entries) = fs::read_dir(&entry).await
        {
            let new_entries = new_entries
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .filter_map(Result::ok)
                .map(|entry| entry.path());

            entries.extend(new_entries);
        }
    }

    Size::from_bytes(size)
}

pub async fn unzip(path: impl AsRef<Path>, target: &Path) -> Result<(), Error> {
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut zip = ZipFileReader::new(&mut reader).await?;

    let target = target.clean();

    for index in 0..zip.file().entries().len() {
        let (filename, is_dir) = {
            let entry = &zip.file().entries()[index];
            (Path::new(entry.filename().as_str()?), entry.dir()?)
        };

        let path = target.join(filename).clean();
        if !path.starts_with(&target) {
            return Err(Error::Zip("Path traversal detected".into()));
        }

        if is_dir {
            fs::create_dir_all(&path).await?;
        } else {
            let mut entry_reader = zip.reader_without_entry(index).await?;

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let file = File::create(&path).await?;
            let mut writer = BufWriter::with_capacity(0x8000, file);

            io::copy(&mut entry_reader, &mut writer).await?;
            writer.flush().await?;
        }
    }

    Ok(())
}

pub async fn filter_valid_games(games: Vec<PathBuf>) -> Vec<PathBuf> {
    async fn is_game(path: &Path) -> Result<bool, Error> {
        let mut file = File::open(path).await?;

        let is_game = if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            let mut reader = BufReader::new(file);
            let mut zip = ZipFileReader::new(&mut reader).await?;
            let mut entry = zip.reader_without_entry(0).await?;
            wii_disc_info::Meta::read(&mut entry).await.is_ok()
        } else {
            wii_disc_info::Meta::read(&mut file).await.is_ok()
        };

        Ok(is_game)
    }

    stream::iter(games)
        .map(|p| async move { is_game(&p).await.unwrap_or(false).then_some(p) })
        .buffer_unordered(8)
        .filter_map(future::ready)
        .collect()
        .await
}
