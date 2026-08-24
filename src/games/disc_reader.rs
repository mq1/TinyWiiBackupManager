// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, util::misc::OPTIMAL_THREADS};
use arrayvec::ArrayVec;
use nod::read::{DiscOptions, DiscReader, DiscStream};
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Write},
    path::Path,
    sync::Arc,
};
use tempfile::tempfile;
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

#[cfg(windows)]
trait ReadExactAt {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> io::Result<()>;
}

#[cfg(windows)]
impl ReadExactAt for File {
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.seek_read(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct SharedMultiFileReader {
    inner: ArrayVec<(Arc<File>, u64), 4>,
    stream_len: u64,
}

impl SharedMultiFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> io::Result<Self> {
        let mut inner = ArrayVec::new();
        let mut stream_len = 0;

        for file in files {
            let size = file.metadata()?.len();
            inner.push((Arc::new(file), size));
            stream_len += size;
        }

        Ok(Self { inner, stream_len })
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
            file.read_exact_at(&mut buf[..n], offset)?;
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
        preloader_threads: OPTIMAL_THREADS.preloader,
        ..Default::default()
    };

    let ext = path.extension().ok_or(Error::InvalidFilename)?;

    let reader = if ext.eq_ignore_ascii_case("zip") {
        let mut zip = ZipArchive::new(File::open(path)?)?;
        let mut entry = zip.by_index(0)?;

        let mut tmp = tempfile()?;
        io::copy(&mut entry, &mut tmp)?;
        tmp.flush()?;

        SharedMultiFileReader::new([tmp])?
    } else {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        let mut files = ArrayVec::<_, 4>::new();
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

        SharedMultiFileReader::new(files)?
    };

    let disc_reader = DiscReader::new_stream(Box::new(reader), &disc_opts)?;
    Ok(disc_reader)
}
