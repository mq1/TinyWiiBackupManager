// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::read::DiscStream;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub fn get_first_file(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|s| {
                    (s.ends_with(".wbfs") || s.ends_with(".iso")) && !s.ends_with(".part1.iso")
                })
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
}

pub fn get_second_file(first: &Path) -> Option<PathBuf> {
    if first.ends_with(".wbfs") {
        let second = first.with_extension("wbf1");
        if second.exists() {
            return Some(second);
        }
    } else if first.ends_with(".part0.iso") {
        let parent = first.parent()?;
        let name = first
            .file_name()?
            .to_str()?
            .replace(".part0.iso", ".part1.iso");

        let second = parent.join(name);
        if second.exists() {
            return Some(second);
        }
    }

    None
}

#[derive(Debug)]
pub struct DoubleReader {
    first: BufReader<File>,
    first_size: u64,
    second: BufReader<File>,
    second_size: u64,
}

impl Clone for DoubleReader {
    fn clone(&self) -> Self {
        let first_file = self.first.get_ref().try_clone().unwrap();
        let second_file = self.second.get_ref().try_clone().unwrap();

        DoubleReader {
            first: BufReader::new(first_file),
            first_size: self.first_size,
            second: BufReader::new(second_file),
            second_size: self.second_size,
        }
    }
}

impl DoubleReader {
    pub fn new(path1: &Path, path2: &Path) -> std::io::Result<Self> {
        let first = BufReader::new(File::open(path1)?);
        let first_size = path1.metadata()?.len();
        let second = BufReader::new(File::open(path2)?);
        let second_size = path2.metadata()?.len();

        Ok(DoubleReader {
            first,
            first_size,
            second,
            second_size,
        })
    }
}

impl DiscStream for DoubleReader {
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
        if offset < self.first_size {
            self.first.seek(SeekFrom::Start(offset))?;
            self.first.read_exact(buf)?;
        } else {
            self.second
                .seek(SeekFrom::Start(offset - self.first_size))?;
            self.second.read_exact(buf)?;
        }

        Ok(())
    }

    fn stream_len(&mut self) -> std::io::Result<u64> {
        Ok(self.first_size + self.second_size)
    }
}
