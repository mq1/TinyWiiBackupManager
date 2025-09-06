// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Range;
use nod::common::PartitionInfo;
use nod::disc::SECTOR_SIZE;
use nod::read::{DiscReader, DiscStream};

#[derive(Clone)]
pub struct FilterDiscReader {
    reader: DiscReader,
    strip_partitions: Vec<Range<u64>>,
}

impl FilterDiscReader {
    pub fn new(reader: DiscReader, strip_partitions: &Vec<PartitionInfo>) -> Self {
        let ranges = strip_partitions
            .iter()
            .map(|partition| {
                let start = partition.data_start_sector as u64 * SECTOR_SIZE as u64;
                let end = partition.data_end_sector as u64 * SECTOR_SIZE as u64;
                start..end
            })
            .collect();

        Self {
            reader,
            strip_partitions: ranges,
        }
    }
}

impl DiscStream for FilterDiscReader {
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        self.reader.seek(SeekFrom::Start(offset))?;

        if self.strip_partitions.is_empty() {
            return self.reader.read_exact(buf);
        }

        if self.strip_partitions.iter().any(|range| range.contains(&offset)) {
            // Fill the buffer with 0x00
            buf.fill(0);
            return Ok(());
        }

        self.reader.read_exact(buf)
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.reader.disc_size())
    }

}
