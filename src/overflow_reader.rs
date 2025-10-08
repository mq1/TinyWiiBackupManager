// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::read::DiscStream;
use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub fn get_main_file(dir: &Path) -> Option<PathBuf> {
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

pub fn get_overflow_file(main: &Path) -> Option<PathBuf> {
    if main.ends_with(".wbfs") {
        let overflow = main.with_extension("wbf1");
        if overflow.exists() {
            return Some(overflow);
        }
    } else if main.ends_with(".part0.iso") {
        let parent = main.parent()?;
        let name = main
            .file_name()?
            .to_str()?
            .replace(".part0.iso", ".part1.iso");

        let overflow = parent.join(name);
        if overflow.exists() {
            return Some(overflow);
        }
    }

    None
}

#[derive(Debug)]
pub struct OverflowReader {
    main: File,
    overflow: File,
}

impl Clone for OverflowReader {
    fn clone(&self) -> Self {
        OverflowReader {
            main: self.main.try_clone().unwrap(),
            overflow: self.overflow.try_clone().unwrap(),
        }
    }
}

impl OverflowReader {
    pub fn new(main_path: &Path, overflow_path: &Path) -> std::io::Result<Self> {
        Ok(OverflowReader {
            main: File::open(main_path)?,
            overflow: File::open(overflow_path)?,
        })
    }
}

impl DiscStream for OverflowReader {
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
        let main_len = self.main.metadata()?.len();

        if offset < main_len {
            self.main.seek(SeekFrom::Start(offset))?;
            self.main.read_exact(buf)
        } else {
            self.overflow.seek(SeekFrom::Start(offset - main_len))?;
            self.overflow.read_exact(buf)
        }
    }

    fn stream_len(&mut self) -> std::io::Result<u64> {
        Ok(self.main.metadata()?.len() + self.overflow.metadata()?.len())
    }
}
