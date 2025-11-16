// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs::{self, File},
    io::{self, Seek, Write},
    path::Path,
};

pub struct OverflowWriter {
    position: u64,
    main: File,
    split_size: u64,
    overflow: File,
}

impl OverflowWriter {
    pub fn new(main_path: &Path, overflow_path: &Path, split_size: u64) -> io::Result<Self> {
        let main = File::create(main_path)?;
        let overflow = File::create(overflow_path)?;

        Ok(Self {
            position: 0,
            main,
            split_size,
            overflow,
        })
    }

    pub fn write_header(&mut self, buf: &[u8]) -> io::Result<()> {
        self.main.rewind()?;
        self.main.write_all(buf)
    }
}

impl Write for OverflowWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let remaining_in_main = self.split_size.saturating_sub(self.position);

        let bytes_written = if remaining_in_main == 0 {
            self.overflow.write(buf)?
        } else {
            let bytes_to_write = buf.len().min(saturating_u64_to_usize(remaining_in_main));
            self.main.write(&buf[..bytes_to_write])?
        };

        self.position += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.main.flush()?;
        self.overflow.flush()
    }
}

pub fn delete_file_if_empty(path: &Path) -> io::Result<()> {
    if path.exists() && path.metadata()?.len() == 0 {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

fn saturating_u64_to_usize(n: u64) -> usize {
    n.try_into().unwrap_or(usize::MAX)
}
