// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::get_optimal_preloader_threads};
use nod::read::{DiscOptions, DiscReader};
use ouroboros::self_referencing;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};
use tempfile::tempfile;
use zip::{ZipArchive, read::ZipFile};

#[self_referencing]
struct ZipEntryStream {
    archive: ZipArchive<File>,

    #[borrows(mut archive)]
    #[not_covariant]
    entry: ZipFile<'this, File>,
}

struct BufferedZipEntry {
    stream: ZipEntryStream,
    buffer: File,
    buffer_pos: u64,
    extracted_len: u64,
    uncompressed_len: u64,
}

impl BufferedZipEntry {
    fn new(archive: ZipArchive<File>, index: usize) -> Result<Self, Error> {
        let stream = ZipEntryStreamTryBuilder {
            archive,
            entry_builder: |a| a.by_index(index),
        }
        .try_build()?;

        let uncompressed_len = stream.with_entry(|e| e.size());

        Ok(Self {
            stream,
            buffer: tempfile()?,
            buffer_pos: 0,
            extracted_len: 0,
            uncompressed_len,
        })
    }

    fn extract_until(&mut self, target_pos: u64) -> io::Result<()> {
        if self.extracted_len >= target_pos {
            return Ok(());
        }

        self.buffer.seek(SeekFrom::End(0))?;

        let read = self.stream.with_entry_mut(|entry| {
            let to_extract = target_pos - self.extracted_len;
            let mut chunk = entry.take(to_extract);
            io::copy(&mut chunk, &mut self.buffer)
        })?;

        self.extracted_len += read as u64;
        Ok(())
    }
}

impl Read for BufferedZipEntry {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let target_pos = (self.buffer_pos + buf.len() as u64).min(self.uncompressed_len);

        self.extract_until(target_pos)?;
        self.buffer.seek(SeekFrom::Start(self.buffer_pos))?;
        let n = self.buffer.read(buf)?;

        self.buffer_pos += n as u64;
        Ok(n)
    }
}

impl Seek for BufferedZipEntry {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::End(pos) => self
                .uncompressed_len
                .checked_add_signed(pos)
                .ok_or(io::ErrorKind::InvalidInput)?,
            SeekFrom::Current(pos) => self
                .buffer_pos
                .checked_add_signed(pos)
                .ok_or(io::ErrorKind::InvalidInput)?,
        };

        if new_pos > self.uncompressed_len {
            return Err(io::ErrorKind::InvalidInput.into());
        }

        self.buffer_pos = new_pos;
        Ok(self.buffer_pos)
    }
}

pub fn disc_file_reader(path: &Path) -> Result<DiscReader, Error> {
    let disc_opts = DiscOptions {
        preloader_threads: get_optimal_preloader_threads(),
        ..Default::default()
    };

    let ext = path.extension().ok_or(Error::InvalidFilename)?;

    let reader = if ext.eq_ignore_ascii_case("zip") {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;
        let entry = BufferedZipEntry::new(archive, 0)?;
        DiscReader::new_from_non_cloneable_read(entry, &disc_opts)?
    } else {
        DiscReader::new(path, &disc_opts)?
    };

    Ok(reader)
}
