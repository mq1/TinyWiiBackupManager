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
        let mut strip_ranges = Vec::new();

        for partition in strip_partitions {
            let partition_start = partition.start_sector as u64 * SECTOR_SIZE as u64;

            strip_ranges.push((partition_start + 0x2B8)..(partition_start + 0x2BC)); // data_off
            strip_ranges.push((partition_start + 0x2BC)..(partition_start + 0x2C0)); // data_size

            let data_start = partition.data_start_sector as u64 * SECTOR_SIZE as u64;
            let data_end = data_start + partition.data_size();
            strip_ranges.push(data_start..data_end);
        }

        Self { reader, strip_ranges }
    }
}

impl DiscStream for FilterDiscReader {
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let read_range = offset..(offset + buf.len() as u64);

        // Find strip ranges relevant to this read request.
        let relevant_strips: Vec<_> = self.strip_ranges.iter()
            .filter(|strip| strip.start < read_range.end && strip.end > read_range.start)
            .collect();

        // Fast path: No overlap, read directly from the disc.
        if relevant_strips.is_empty() {
            self.reader.seek(SeekFrom::Start(offset))?;
            return self.reader.read_exact(buf);
        }

        // Fast path: Read is fully contained within a strip range, fill buffer with zeros.
        if relevant_strips.iter().any(|strip| strip.start <= read_range.start && strip.end >= read_range.end) {
            buf.fill(0);
            return Ok(());
        }

        // Fallback for partial overlaps: read from disc, then patch the buffer.
        self.reader.seek(SeekFrom::Start(offset))?;
        self.reader.read_exact(buf)?;

        for strip_range in relevant_strips {
            let overlap_start = max(read_range.start, strip_range.start);
            let overlap_end = min(read_range.end, strip_range.end);

            let start_in_buf = (overlap_start - offset) as usize;
            let end_in_buf = (overlap_end - offset) as usize;
            buf[start_in_buf..end_in_buf].fill(0);
        }

        Ok(())
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.reader.disc_size())
    }
}
