// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    cmp,
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
        .filter(|path| {
            path.to_str()
                .map(|p| !p.ends_with(".part1.iso"))
                .unwrap_or(false)
        })
        .find(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| matches!(ext, "iso" | "wbfs" | "ciso" | "gcm"))
                .unwrap_or(false)
        })
}

pub fn get_overflow_file_name(main_file_name: &str) -> Option<String> {
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
    let main_file_name = main.file_name()?.to_str()?;
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
    overflow_len: u64,
    total_len: u64,
}

impl Clone for OverflowReader {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            main: self.main.try_clone().unwrap(),
            main_len: self.main_len,
            overflow: self.overflow.try_clone().unwrap(),
            overflow_len: self.overflow_len,
            total_len: self.total_len,
        }
    }
}

impl OverflowReader {
    pub fn new(main_path: &Path, overflow_path: &Path) -> io::Result<Self> {
        let main = File::open(main_path)?;
        let main_len = main.metadata()?.len();
        let overflow = File::open(overflow_path)?;
        let overflow_len = overflow.metadata()?.len();

        Ok(OverflowReader {
            position: 0,
            main,
            main_len,
            overflow,
            overflow_len,
            total_len: main_len + overflow_len,
        })
    }
}

impl Read for OverflowReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.total_len {
            return Ok(0);
        }

        let bytes_read = if self.position < self.main_len {
            self.main.seek(SeekFrom::Start(self.position))?;
            let read_size = cmp::min(buf.len() as u64, self.main_len - self.position) as usize;
            self.main.read(&mut buf[..read_size])?
        } else {
            let overflow_pos = self.position - self.main_len;
            self.overflow.seek(SeekFrom::Start(overflow_pos))?;
            self.overflow.read(buf)?
        };

        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Seek for OverflowReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(start) => start as i64,
            SeekFrom::End(end) => (self.main_len + self.overflow_len) as i64 + end,
            SeekFrom::Current(current) => self.position as i64 + current,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative position",
            ));
        }

        self.position = new_pos as u64;
        Ok(self.position)
    }
}
