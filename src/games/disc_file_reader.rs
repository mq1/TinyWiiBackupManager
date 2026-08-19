// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::get_optimal_preloader_threads};
use async_zip::base::read::seek::ZipFileReader;
use nod::read::{DiscOptions, DiscReader, DiscStream};
use std::{fs::File, io, path::Path, sync::Arc};
use tempfile::tempfile;

#[derive(Debug, Clone)]
struct SharedFileReader {
    file: Arc<File>,
}

impl SharedFileReader {
    pub fn new(file: Arc<File>) -> io::Result<Self> {
        Ok(Self { file })
    }
}

impl DiscStream for SharedFileReader {
    fn stream_len(&mut self) -> io::Result<u64> {
        let metadata = self.file.metadata()?;
        Ok(metadata.len())
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        use positioned_io::ReadAt;
        self.file.read_exact_at(offset, buf)
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

        let tmp = SharedFileReader::new(Arc::new(tmp))?;
        DiscReader::new_stream(Box::new(tmp), &disc_opts)?
    } else {
        DiscReader::new(path, &disc_opts)?
    };

    Ok(reader)
}
