// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::get_optimal_preloader_threads};
use async_zip::base::read::seek::ZipFileReader;
use nod::read::{DiscOptions, DiscReader};
use std::{fs::File, io, path::Path, sync::Arc};
use tempfile::NamedTempFile;

struct ClonableFileReader {
    inner: File,
    temp_guard: Arc<NamedTempFile>,
}

impl ClonableFileReader {
    pub fn new(tmp: NamedTempFile) -> io::Result<Self> {
        let inner = tmp.reopen()?;
        let temp_guard = Arc::new(tmp);

        Ok(Self { inner, temp_guard })
    }
}

impl io::Read for ClonableFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl io::Seek for ClonableFileReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl Clone for ClonableFileReader {
    fn clone(&self) -> Self {
        let inner = self.temp_guard.reopen().unwrap();
        let temp_guard = self.temp_guard.clone();

        Self { inner, temp_guard }
    }
}

pub fn disc_file_reader(path: &Path) -> Result<DiscReader, Error> {
    let disc_opts = DiscOptions {
        preloader_threads: get_optimal_preloader_threads(),
        ..Default::default()
    };

    let ext = path.extension().ok_or(Error::InvalidFilename)?;

    let reader = if ext.eq_ignore_ascii_case("zip") {
        let tmp = futures::executor::block_on(async {
            use futures::{AsyncSeekExt, AsyncWriteExt};

            let file = futures::io::AllowStdIo::new(File::open(path)?);
            let mut reader = futures::io::BufReader::new(file);
            let mut zip = ZipFileReader::new(&mut reader).await?;
            let mut entry = zip.reader_without_entry(0).await?;

            let mut writer = futures::io::AllowStdIo::new(NamedTempFile::new()?);
            futures::io::copy(&mut entry, &mut writer).await?;
            writer.flush().await?;
            writer.seek(io::SeekFrom::Start(0)).await?;

            let tmp = writer.into_inner();
            Ok::<_, Error>(tmp)
        })?;

        let tmp = ClonableFileReader::new(tmp)?;
        DiscReader::new_from_cloneable_read(tmp, &disc_opts)?
    } else {
        DiscReader::new(path, &disc_opts)?
    };

    Ok(reader)
}
