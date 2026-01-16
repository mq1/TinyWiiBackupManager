// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::can_write_over_4gb;
use std::io::BufWriter;
use std::path::PathBuf;
use std::{
    fs,
    fs::File,
    io::{self, Seek, Write},
    path::Path,
};

const SPLIT_SIZE: usize = 4294934528; // 4 GiB - 32 KiB (fits in a u32 on 32-bit systems)

pub fn get_overflow_path(main_path: &Path) -> Option<PathBuf> {
    let parent = main_path.parent()?;
    let file_name = main_path.file_name()?.to_str()?;

    if file_name.ends_with(".part0.iso") {
        let overflow_file_name = file_name.replace(".part0.iso", ".part1.iso");
        Some(parent.join(overflow_file_name))
    } else if file_name.ends_with(".wbfs") {
        let overflow_file_name = file_name.replace(".wbfs", ".wbf1");
        Some(parent.join(overflow_file_name))
    } else {
        None
    }
}

pub struct OverflowWriter {
    main_pos: usize,
    main: BufWriter<File>,
    overflow_path: Option<PathBuf>,
    overflow: Option<BufWriter<File>>,
}

impl OverflowWriter {
    pub fn new(main_path: &Path, always_split: bool) -> io::Result<Self> {
        let main_parent = main_path
            .parent()
            .ok_or(io::Error::other("No parent directory"))?;

        fs::create_dir_all(main_parent)?;
        let main = BufWriter::new(File::create(main_path)?);

        let overflow_path = if let Some(path) = get_overflow_path(main_path)
            && (always_split || !can_write_over_4gb(main_parent))
        {
            Some(path)
        } else {
            None
        };

        Ok(Self {
            main_pos: 0,
            main,
            overflow_path,
            overflow: None,
        })
    }

    // This is the last thing we do, so we don't need to update the position
    pub fn write_header(&mut self, buf: &[u8]) -> io::Result<()> {
        self.main.rewind()?;
        self.main.write_all(buf)
    }
}

impl Write for OverflowWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Main file already full, we write to overflow
        if let Some(overflow) = &mut self.overflow {
            return overflow.write(buf);
        }

        // Main file not full, we write to main file and create the overflow file if needed
        if let Some(overflow_path) = &self.overflow_path {
            let remaining_in_main = SPLIT_SIZE - self.main_pos;

            // Main file is full, we create the overflow file
            if remaining_in_main == 0 {
                self.overflow = Some(BufWriter::new(File::create(overflow_path)?));
                return self.write(buf);
            }

            // Hey, you. You’re finally awake. You were trying to cross the border, right?
            if remaining_in_main < buf.len() {
                let n = self.main.write(&buf[..remaining_in_main])?;
                self.main_pos += n;
                return Ok(n);
            }

            // Main file not near split size, we write to main file
            let n = self.main.write(buf)?;
            self.main_pos += n;
            return Ok(n);
        }

        // Main file not split, we write to main file
        self.main.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.main.flush()?;
        if let Some(overflow) = &mut self.overflow {
            overflow.flush()?;
        };
        Ok(())
    }
}
