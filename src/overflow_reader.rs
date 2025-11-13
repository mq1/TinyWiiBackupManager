// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

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
    split_position: u64,
    overflow: File,
    len: u64,
}

impl Clone for OverflowReader {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            main: self.main.try_clone().unwrap(),
            split_position: self.split_position,
            overflow: self.overflow.try_clone().unwrap(),
            len: self.len,
        }
    }
}

impl OverflowReader {
    pub fn new(main_path: &Path, overflow_path: &Path) -> io::Result<Self> {
        let main = File::open(main_path)?;
        let split_position = main.metadata()?.len();
        let overflow = File::open(overflow_path)?;
        let len = split_position + overflow.metadata()?.len();

        Ok(Self {
            position: 0,
            main,
            split_position,
            overflow,
            len,
        })
    }
}

impl Read for OverflowReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.len {
            return Ok(0);
        }

        let bytes_read = if self.position < self.len {
            self.main.seek(SeekFrom::Start(self.position))?;

            // read from main file AND overflow file
            if self.position + buf.len() as u64 > self.split_position {
                let split_point = (self.split_position - self.position) as usize;
                let main_bytes = self.main.read(&mut buf[..split_point])?;
                let overflow_bytes = self.overflow.read(&mut buf[split_point..])?;

                main_bytes + overflow_bytes
            } else {
                self.main.read(buf)?
            }
        } else {
            let overflow_pos = self.position - self.split_position;
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
            SeekFrom::End(end) => self.len as i64 + end,
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
