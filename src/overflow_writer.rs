// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::BufWriter;
use std::path::PathBuf;
use std::{
    fs::File,
    io::{self, Seek, Write},
    path::Path,
};

const SPLIT_SIZE: usize = 4294934528; // 4 GiB - 32 KiB (fits in a u32 on 32-bit systems)

pub struct OverflowWriter {
    main_pos: usize,
    main: BufWriter<File>,
    must_split: bool,
    overflow_path: PathBuf,
    overflow: Option<BufWriter<File>>,
}

impl OverflowWriter {
    pub fn new(main_path: &Path, overflow_path: PathBuf, must_split: bool) -> io::Result<Self> {
        let main = BufWriter::new(File::create(main_path)?);

        Ok(Self {
            main_pos: 0,
            main,
            must_split,
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
            overflow.write(buf)
        }
        // Main file not full, we write to main file and create the overflow file if needed
        else if self.must_split {
            let remaining_in_main = SPLIT_SIZE - self.main_pos;

            // Hey, you. You’re finally awake. You were trying to cross the border, right?
            if remaining_in_main < buf.len() {
                let main_n = self.main.write(&buf[..remaining_in_main])?;
                let mut overflow = BufWriter::new(File::create(&self.overflow_path)?);
                let overflow_n = overflow.write(&buf[remaining_in_main..])?;
                self.overflow = Some(overflow);

                Ok(main_n + overflow_n)
            }
            // Main file not near split size, we write to main file
            else {
                let bytes_written = self.main.write(buf)?;
                self.main_pos += bytes_written;
                Ok(bytes_written)
            }
        }
        // Main file not split, we write to main file
        else {
            self.main.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.main.flush()?;
        if let Some(overflow) = &mut self.overflow {
            overflow.flush()?;
        };
        Ok(())
    }
}
