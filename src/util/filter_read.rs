// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::cmp::{max, min};
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Range;
use nod::common::PartitionInfo;
use nod::disc::SECTOR_SIZE;
use nod::read::{DiscReader, DiscStream};

#[derive(Clone)]
pub struct FilterDiscReader {
    reader: DiscReader,
    strip_ranges: Vec<Range<u64>>,
}

impl FilterDiscReader {
    pub fn new(reader: DiscReader, strip_partitions: &Vec<PartitionInfo>) -> Self {
        let mut ranges = Vec::new();

        for partition in strip_partitions {
            if partition.data_size() == 0 {
                continue;
            }

            let partition_start = partition.start_sector as u64 * SECTOR_SIZE as u64;

            // Range to zero out `data_off` in the header
            let offset_data_off = partition_start + 0x2B8;
            ranges.push(offset_data_off..(offset_data_off + 4));

            // Range to zero out `data_size` in the header
            let offset_data_size = partition_start + 0x2BC;
            ranges.push(offset_data_size..(offset_data_size + 4));

            // Range to zero out the actual partition data
            let data_start = partition.data_start_sector as u64 * SECTOR_SIZE as u64;
            let data_end = data_start + partition.data_size();

            if data_start < data_end {
                ranges.push(data_start..data_end);
            }
        }

        Self {
            reader,
            strip_ranges: ranges,
        }
    }
}

impl DiscStream for FilterDiscReader {
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        // --- STEP 1: Always read the real data from the disc first ---
        // This fills the buffer with the original data. We will now "patch" it.
        self.reader.seek(SeekFrom::Start(offset))?;
        self.reader.read_exact(buf)?;

        // If there is nothing to strip, we are done.
        if self.strip_ranges.is_empty() {
            return Ok(());
        }

        // --- STEP 2: Zero out the parts of the buffer that overlap ---
        let read_len = buf.len() as u64;
        let read_range = offset..(offset + read_len);

        for strip_range in &self.strip_ranges {
            // Calculate the exact overlap between the read request and the range to strip.
            let overlap_start = max(read_range.start, strip_range.start);
            let overlap_end = min(read_range.end, strip_range.end);

            // If a real overlap exists...
            if overlap_start < overlap_end {
                // ...translate the absolute offsets into indices relative to the buffer...
                let start_in_buf = (overlap_start - offset) as usize;
                let end_in_buf = (overlap_end - offset) as usize;

                // ...and zero out only that specific slice of the buffer.
                buf[start_in_buf..end_in_buf].fill(0);
            }
        }

        Ok(())
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        // A virtual reader must ALWAYS report the original disc size.
        // Otherwise, callers may think valid data at high offsets is out of bounds.
        Ok(self.reader.disc_size())
    }
}
