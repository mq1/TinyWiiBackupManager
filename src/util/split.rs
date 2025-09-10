// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::fs::{File, OpenOptions};
use std::io;
use std::io::Seek;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct SplitWbfsFile {
    base_path: PathBuf,
    writer: BufWriter<File>,
    current_size: u64,
    max_size: u64,
    current_part: u8,
}

impl SplitWbfsFile {
    pub fn new(base_path: impl AsRef<Path>, max_size: u64) -> Result<Self, io::Error> {
        let base_path = base_path.as_ref().to_path_buf();
        let first_file = base_path.with_extension("wbfs");
        let current_file = File::create(&first_file)?;
        let writer = BufWriter::new(current_file);

        Ok(Self {
            base_path,
            writer,
            current_size: 0,
            max_size,
            current_part: 0,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), io::Error> {
        // Split if needed
        if self.current_size + data.len() as u64 > self.max_size {
            self.current_part += 1;

            let new_path = self
                .base_path
                .with_extension(format!("wbf{}", self.current_part));
            let new_file = File::create(&new_path)?;
            let new_writer = BufWriter::new(new_file);
            self.writer = new_writer;

            self.current_size = 0;
        }

        self.writer.write_all(data)?;
        self.current_size += data.len() as u64;
        Ok(())
    }

    pub fn write_to_start(&mut self, data: &[u8]) -> Result<(), io::Error> {
        // Open the initial .wbfs file for writing at the start
        let first_path = self.base_path.with_extension("wbfs");
        let first_file = OpenOptions::new().write(true).open(&first_path)?;

        self.writer = BufWriter::new(first_file);

        // Seek to the beginning and write the data
        self.writer.rewind()?;
        self.writer.write_all(data)?;

        Ok(())
    }
}
