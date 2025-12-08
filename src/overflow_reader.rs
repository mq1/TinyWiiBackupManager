// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::OsStr;
use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
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

#[derive(Debug)]
pub struct OverflowReader {
    position: u64,
    main: File,
    main_len: u64,
    overflow: File,
    len: u64,
}

impl OverflowReader {
    pub fn new(main_path: &Path, overflow_path: &Path) -> io::Result<Self> {
        let main = File::open(main_path)?;
        let main_len = main.metadata()?.len();
        let overflow = File::open(overflow_path)?;
        let len = main_len + overflow.metadata()?.len();

        Ok(Self {
            position: 0,
            main,
            main_len,
            overflow,
            len,
        })
    }
}

impl Read for OverflowReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining_in_main = (self.len - self.position).min(usize::MAX as u64) as usize;

        let bytes_read = if self.position < self.main_len {
            if remaining_in_main < buf.len() {
                self.main.read_exact(&mut buf[..remaining_in_main])?;
                let overflow_n = self.overflow.read(&mut buf[remaining_in_main..])?;
                remaining_in_main + overflow_n
            } else {
                self.main.read(buf)?
            }
        } else {
            self.overflow.read(buf)?
        };

        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Seek for OverflowReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::Current(offset) => self.position.checked_add_signed(offset),
            SeekFrom::End(offset) => self.len.checked_add_signed(offset),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"))?;

        if new_pos < self.main_len {
            self.main.seek(SeekFrom::Start(new_pos))?;
            self.overflow.seek(SeekFrom::Start(0))?;
        } else {
            let overflow_pos = new_pos - self.main_len;
            self.overflow.seek(SeekFrom::Start(overflow_pos))?;
            self.main.seek(SeekFrom::End(0))?;
        }

        self.position = new_pos;
        Ok(new_pos)
    }
}
