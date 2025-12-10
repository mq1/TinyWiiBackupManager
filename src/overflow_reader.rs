// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::read::DiscStream;
use positioned_io::{RandomAccessFile, ReadAt};
use std::ffi::OsStr;
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
    overflow: Arc<RandomAccessFile>,
    main_len: u64,
    total_len: u64,
}

impl OverflowReader {
    pub fn new(main_path: &Path, overflow_path: &Path) -> io::Result<Self> {
        let main_file = File::open(main_path)?;
        let overflow_file = File::open(overflow_path)?;

        let main_len = main_file.metadata()?.len();
        let total_len = main_len + overflow_file.metadata()?.len();

        let main = RandomAccessFile::try_new(main_file)?;
        let overflow = RandomAccessFile::try_new(overflow_file)?;

        Ok(Self {
            main: Arc::new(main),
            overflow: Arc::new(overflow),
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
            let overflow_offset = offset - self.main_len;
            self.overflow.read_exact_at(overflow_offset, buf)?;
        } else {
            let bytes_in_main = (self.main_len - offset) as usize;
            self.main.read_exact_at(offset, &mut buf[..bytes_in_main])?;
            self.overflow.read_exact_at(0, &mut buf[bytes_in_main..])?;
        }

        Ok(())
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.total_len)
    }
}
