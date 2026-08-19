// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::get_optimal_preloader_threads};
use async_zip::base::read::seek::ZipFileReader;
use memmap2::Mmap;
use nod::read::{DiscOptions, DiscReader};
use std::{fs::File, io, path::Path, sync::Arc};
use tempfile::tempfile;

struct SharedFileReader {
    file: Arc<File>,
    cursor: io::Cursor<Mmap>,
}

impl SharedFileReader {
    pub fn new(file: Arc<File>, initial_pos: u64) -> io::Result<Self> {
        use std::io::Seek;

        let mmap = unsafe { Mmap::map(file.as_ref())? };
        let mut cursor = io::Cursor::new(mmap);
        cursor.seek(io::SeekFrom::Start(initial_pos))?;

        Ok(Self { file, cursor })
    }
}

impl io::Read for SharedFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl io::Seek for SharedFileReader {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl Clone for SharedFileReader {
    fn clone(&self) -> Self {
        let file = self.file.clone();
        let initial_pos = self.cursor.position();

        Self::new(file, initial_pos).unwrap()
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

            let mut writer = futures::io::AllowStdIo::new(tempfile()?);
            futures::io::copy(&mut entry, &mut writer).await?;
            writer.flush().await?;
            writer.seek(io::SeekFrom::Start(0)).await?;

            let tmp = writer.into_inner();
            Ok::<_, Error>(tmp)
        })?;

        let tmp = SharedFileReader::new(Arc::new(tmp), 0)?;
        DiscReader::new_from_cloneable_read(tmp, &disc_opts)?
    } else {
        DiscReader::new(path, &disc_opts)?
    };

    Ok(reader)
}
