// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::get_optimal_preloader_threads};
use arrayvec::ArrayVec;
use async_zip::base::read::seek::ZipFileReader;
use futures::AsyncWriteExt;
use nod::read::{DiscOptions, DiscReader, DiscStream};
use positioned_io::{RandomAccessFile, ReadAt};
use std::{ffi::OsStr, fs::File, io, path::Path, sync::Arc};
use tempfile::tempfile;

#[derive(Debug, Clone)]
struct SharedFileReader {
    inner: Arc<RandomAccessFile>,
    stream_len: u64,
}

impl SharedFileReader {
    pub fn new(file: File) -> io::Result<Self> {
        let stream_len = file.metadata()?.len();
        let file = RandomAccessFile::try_new(file)?;

        Ok(Self {
            inner: Arc::new(file),
            stream_len,
        })
    }
}

impl DiscStream for SharedFileReader {
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        self.inner.read_exact_at(offset, buf)
    }
}

#[derive(Debug, Clone)]
struct SharedMultiFileReader {
    inner: Arc<ArrayVec<(RandomAccessFile, u64), 4>>,
    stream_len: u64,
}

impl SharedMultiFileReader {
    pub fn new(files: ArrayVec<File, 4>) -> io::Result<Self> {
        let mut inner = ArrayVec::new();
        let mut stream_len = 0;

        for file in files {
            let size = file.metadata()?.len();
            let raf = RandomAccessFile::try_new(file)?;
            inner.push((raf, size));
            stream_len += size;
        }

        Ok(Self {
            inner: Arc::new(inner),
            stream_len,
        })
    }
}

impl DiscStream for SharedMultiFileReader {
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let mut offset = offset;
        let mut buf = buf;

        for (file, len) in self.inner.iter() {
            // If the offset is past the end of the file, skip it
            if offset >= *len {
                offset -= len;
                continue;
            }

            let n = ((*len - offset) as usize).min(buf.len());
            file.read_exact_at(offset, &mut buf[..n])?;
            buf = &mut buf[n..];
            offset = 0;

            if buf.is_empty() {
                return Ok(());
            }
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(io::ErrorKind::UnexpectedEof.into())
        }
    }
}

pub fn get_disc_reader(path: &Path) -> Result<DiscReader, Error> {
    let disc_opts = DiscOptions {
        preloader_threads: get_optimal_preloader_threads(),
        ..Default::default()
    };

    let ext = path.extension().ok_or(Error::InvalidFilename)?;

    let stream: Box<dyn DiscStream> = if ext.eq_ignore_ascii_case("zip") {
        let tmp = futures::executor::block_on(async {
            let file = futures::io::AllowStdIo::new(File::open(path)?);
            let mut reader = futures::io::BufReader::new(file);
            let mut zip = ZipFileReader::new(&mut reader).await?;
            let mut entry = zip.reader_without_entry(0).await?;

            let mut writer = futures::io::AllowStdIo::new(tempfile()?);
            futures::io::copy(&mut entry, &mut writer).await?;
            writer.flush().await?;

            let tmp = writer.into_inner();
            Ok::<_, Error>(tmp)
        })?;

        Box::new(SharedFileReader::new(tmp)?)
    } else {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        let mut files = ArrayVec::new();
        files.push(File::open(path)?);

        if let Some(filename) = filename.strip_suffix(".part0.iso") {
            let disc1_path = path.with_file_name(format!("{filename}.part1.iso"));
            files.push(File::open(&disc1_path)?);
        }

        if let Some(filename) = filename.strip_suffix(".wbfs") {
            for i in 1..=3 {
                let wbfx_path = path.with_file_name(format!("{filename}.wbf{i}"));
                if !wbfx_path.exists() {
                    break;
                }
                files.push(File::open(&wbfx_path)?);
            }
        }

        if files.len() == 1 {
            let file = files.pop().unwrap();
            Box::new(SharedFileReader::new(file)?)
        } else {
            Box::new(SharedMultiFileReader::new(files)?)
        }
    };

    let disc_reader = DiscReader::new_stream(stream, &disc_opts)?;
    Ok(disc_reader)
}
