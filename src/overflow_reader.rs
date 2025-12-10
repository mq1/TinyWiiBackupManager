// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::read::DiscStream;
use positioned_io::{RandomAccessFile, ReadAt};
use std::ffi::OsStr;
use std::fs::Metadata;
use std::sync::Arc;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

pub fn get_main_file(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.to_str().is_some_and(|p| !p.ends_with(".part1.iso")))
        .find(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| matches!(ext, "iso" | "wbfs" | "ciso" | "gcm"))
        })
}

pub fn get_overflow_file_name(main_file_name: &OsStr) -> Option<String> {
    let main_file_name = main_file_name.to_str()?;

    if main_file_name.ends_with(".wbfs") {
        Some(main_file_name.replace(".wbfs", ".wbf1"))
    } else if main_file_name.ends_with(".part0.iso") {
        Some(main_file_name.replace(".part0.iso", ".part1.iso"))
    } else {
        None
    }
}

pub fn get_overflow_file(main: &Path) -> Option<PathBuf> {
    let parent = main.parent()?;
    let main_file_name = main.file_name()?;
    let file_name = get_overflow_file_name(main_file_name)?;
    let path = parent.join(file_name);

    if path.exists() { Some(path) } else { None }
}

#[derive(Debug, Clone)]
pub struct OverflowReader {
    main: Arc<RandomAccessFile>,
    pub overflow_path: Option<PathBuf>,
    overflow: Option<Arc<RandomAccessFile>>,
    main_len: u64,
    total_len: u64,
}

impl OverflowReader {
    pub fn new(main_path: &Path) -> io::Result<Self> {
        let overflow_path = get_overflow_file(main_path);

        let main_file = File::open(main_path)?;
        let overflow_file = overflow_path.as_ref().map(File::open).transpose()?;

        let main_len = main_file.metadata()?.len();
        let overflow_len = overflow_file
            .as_ref()
            .map(File::metadata)
            .transpose()?
            .as_ref()
            .map(Metadata::len);

        let total_len = match overflow_len {
            Some(overflow_len) => main_len + overflow_len,
            None => main_len,
        };

        let main = Arc::new(RandomAccessFile::try_new(main_file)?);
        let overflow = overflow_file
            .map(RandomAccessFile::try_new)
            .transpose()?
            .map(Arc::new);

        Ok(Self {
            main,
            overflow_path,
            overflow,
            main_len,
            total_len,
        })
    }
}

impl DiscStream for OverflowReader {
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let buf_len = buf.len() as u64;
        let end = offset + buf_len;

        if end <= self.main_len {
            self.main.read_exact_at(offset, buf)?;
        } else if offset >= self.main_len {
            match &self.overflow {
                Some(overflow) => {
                    let overflow_offset = offset - self.main_len;
                    overflow.read_exact_at(overflow_offset, buf)?;
                }
                None => {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
            }
        } else {
            match &self.overflow {
                Some(overflow) => {
                    let bytes_in_main = (self.main_len - offset) as usize;
                    self.main.read_exact_at(offset, &mut buf[..bytes_in_main])?;
                    overflow.read_exact_at(0, &mut buf[bytes_in_main..])?;
                }
                None => {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
            }
        }

        Ok(())
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.total_len)
    }
}
