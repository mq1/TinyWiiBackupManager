use std::{
    io,
    io::{BufRead, Read, Seek, SeekFrom},
    ops::Range,
};

use bytes::{Bytes, BytesMut};
use dyn_clone::DynClone;
use zerocopy::FromBytes;

use crate::{
    Result, ResultContext,
    common::{PartitionInfo, PartitionKind},
    disc::{
        SECTOR_SIZE,
        reader::DiscReader,
        wii::{HASHES_SIZE, SECTOR_DATA_SIZE, WII_PART_GROUP_OFF, WiiPartEntry, WiiPartGroup},
    },
    util::{aes::decrypt_sector_b2b, array_ref, array_ref_mut, lfg::LaggedFibonacci},
    write::{DataCallback, DiscFinalization, DiscWriterWeight, ProcessOptions},
};

/// A trait for writing disc images.
pub trait DiscWriter: DynClone {
    /// Processes the disc writer to completion.
    ///
    /// The data callback will be called, in order, for each block of data to write to the output
    /// file. The callback should write all data before returning, or return an error if writing
    /// fails.
    fn process(
        &self,
        data_callback: &mut DataCallback,
        options: &ProcessOptions,
    ) -> Result<DiscFinalization>;

    /// Returns the progress upper bound for the disc writer.
    ///
    /// For most formats, this has no relation to the written disc size, but can be used to display
    /// progress.
    fn progress_bound(&self) -> u64;

    /// Returns the weight of the disc writer.
    ///
    /// This can help determine the number of threads to dedicate for output processing, and may
    /// differ based on the format's configuration, such as whether compression is enabled.
    fn weight(&self) -> DiscWriterWeight;
}

dyn_clone::clone_trait_object!(DiscWriter);

#[derive(Default)]
pub struct BlockResult<T> {
    /// Input block index
    pub block_idx: u32,
    /// Input disc data (before processing)
    pub disc_data: Bytes,
    /// Output block data (after processing). If None, the disc data is used.
    pub block_data: Bytes,
    /// Output metadata
    pub meta: T,
}

pub trait BlockProcessor: Clone + Send {
    type BlockMeta;

    fn process_block(&mut self, block_idx: u32) -> io::Result<BlockResult<Self::BlockMeta>>;
}

pub fn read_block(reader: &mut DiscReader, block_size: usize) -> io::Result<(Bytes, Bytes)> {
    let initial_block = reader.fill_buf_internal()?;
    if initial_block.len() >= block_size {
        // Happy path: we have a full block that we can cheaply slice
        let data = initial_block.slice(0..block_size);
        reader.consume(block_size);
        return Ok((data.clone(), data));
    } else if initial_block.is_empty() {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
    }
    reader.consume(initial_block.len());

    // Combine smaller blocks into a new buffer
    let mut buf = BytesMut::zeroed(block_size);
    let mut len = initial_block.len();
    buf[..len].copy_from_slice(initial_block.as_ref());
    drop(initial_block);
    while len < block_size {
        let read = reader.read(&mut buf[len..])?;
        if read == 0 {
            break;
        }
        len += read;
    }
    // The block data is full size, padded with zeroes
    let block_data = buf.freeze();
    // The disc data is the actual data read, without padding
    let disc_data = block_data.slice(0..len);
    Ok((block_data, disc_data))
}

/// Process blocks in parallel, ensuring that they are written in order.
#[cfg_attr(not(feature = "threading"), inline)]
pub(crate) fn par_process<P, T>(
    mut processor: P,
    block_count: u32,
    #[cfg(feature = "threading")] num_threads: usize,
    mut callback: impl FnMut(BlockResult<T>) -> Result<()>,
) -> Result<()>
where
    T: Send,
    P: BlockProcessor<BlockMeta = T>,
{
    #[cfg(feature = "threading")]
    if num_threads > 0 {
        return std::thread::scope(|s| {
            use std::collections::VecDeque;

            use crate::Error;

            let (block_tx, block_rx) = crossbeam_channel::bounded(block_count as usize);
            for block_idx in 0..block_count {
                block_tx.send(block_idx).unwrap();
            }
            drop(block_tx); // Disconnect channel

            // Buffer up to one result per worker, allowing processing to continue while the
            // main thread reorders and writes completed blocks.
            let (result_tx, result_rx) = crossbeam_channel::bounded(num_threads);

            // Spawn threads to process blocks
            for _ in 0..num_threads - 1 {
                let block_rx = block_rx.clone();
                let result_tx = result_tx.clone();
                let mut processor = processor.clone();
                s.spawn(move || {
                    while let Ok(block_idx) = block_rx.recv() {
                        let result = processor
                            .process_block(block_idx)
                            .with_context(|| format!("Failed to process block {block_idx}"));
                        let failed = result.is_err(); // Stop processing if an error occurs
                        if result_tx.send(result).is_err() || failed {
                            break;
                        }
                    }
                });
            }

            // Last iteration moves instead of cloning
            s.spawn(move || {
                while let Ok(block_idx) = block_rx.recv() {
                    let result = processor
                        .process_block(block_idx)
                        .with_context(|| format!("Failed to process block {block_idx}"));
                    let failed = result.is_err(); // Stop processing if an error occurs
                    if result_tx.send(result).is_err() || failed {
                        break;
                    }
                }
            });

            // Main thread processes results
            let mut current_block = 0;
            let mut out_of_order = VecDeque::<BlockResult<T>>::new();
            while let Ok(result) = result_rx.recv() {
                let result = result?;
                if result.block_idx == current_block {
                    callback(result)?;
                    current_block += 1;
                    // Check if any out of order blocks can be written
                    while out_of_order.front().is_some_and(|r| r.block_idx == current_block) {
                        callback(out_of_order.pop_front().unwrap())?;
                        current_block += 1;
                    }
                } else {
                    // Insert sorted
                    match out_of_order.binary_search_by_key(&result.block_idx, |r| r.block_idx) {
                        Ok(idx) => Err(Error::Other(format!("Unexpected duplicate block {idx}")))?,
                        Err(idx) => out_of_order.insert(idx, result),
                    }
                }
            }

            Ok(())
        });
    }

    // Fall back to single-threaded processing
    for block_idx in 0..block_count {
        let block = processor
            .process_block(block_idx)
            .with_context(|| format!("Failed to process block {block_idx}"))?;
        callback(block)?;
    }
    Ok(())
}

/// The determined block type.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckBlockResult {
    Normal,
    Zeroed,
    Junk,
}

/// Stores the byte ranges within a partition's data area that contain real disc
/// content. Derived from the partition structure and FST rather than inspecting
/// block data, because block data may be zeroed or match the junk fill pattern.
pub(crate) struct PartitionUsage {
    ranges: Option<Vec<Range<u64>>>,
}

impl PartitionUsage {
    pub(crate) fn new(partition: &PartitionInfo) -> Self {
        let Some(fst) = partition.fst() else {
            return Self { ranges: None };
        };
        let is_wii = partition.disc_header().is_wii();
        let boot_header = partition.boot_header();

        let mut ranges = Vec::new();
        // Conservatively protect everything up to the end of the FST
        let system_end = boot_header.fst_offset(is_wii) + boot_header.fst_size(is_wii);
        ranges.push(0..system_end);

        for &node in fst.nodes {
            if !node.is_file() {
                continue;
            }
            let start = node.offset(is_wii);
            ranges.push(start..start + node.length() as u64);
        }

        ranges.sort_unstable_by_key(|r| r.start);
        let mut merged: Vec<Range<u64>> = Vec::with_capacity(ranges.len());
        for r in ranges {
            match merged.last_mut() {
                Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
                _ => merged.push(r),
            }
        }
        Self { ranges: Some(merged) }
    }

    fn overlaps_used(&self, start: u64, end: u64) -> bool {
        let Some(ranges) = &self.ranges else { return false };
        let idx = ranges.partition_point(|r| r.end <= start);
        ranges.get(idx).is_some_and(|r| r.start < end)
    }
}

/// Used when scrubbing a partition, so the output doesn't claim to still
/// have a real partition where the data has been discarded.
pub(crate) struct PartitionRemoval {
    edits: Vec<(u64, Vec<u8>)>,
}

impl PartitionRemoval {
    pub(crate) fn new(disc: &mut DiscReader, kind: PartitionKind) -> io::Result<Option<Self>> {
        if !disc.header().is_wii() {
            return Ok(None);
        }

        disc.seek(SeekFrom::Start(WII_PART_GROUP_OFF))?;
        let mut group_buf = [0u8; 32]; // 4 groups * 8 bytes each
        disc.read_exact(&mut group_buf)?;

        let mut edits = Vec::new();
        for group_idx in 0..4u64 {
            let group_bytes = &group_buf[group_idx as usize * 8..group_idx as usize * 8 + 8];
            let group = WiiPartGroup::read_from_bytes(group_bytes)
                .map_err(|_| io::Error::other("Invalid partition group"))?;
            let part_count = group.part_count.get();
            if part_count == 0 {
                continue;
            }
            let entry_off = group.part_entry_off();
            disc.seek(SeekFrom::Start(entry_off))?;
            let mut entry_buf = vec![0u8; part_count as usize * 8];
            disc.read_exact(&mut entry_buf)?;

            let mut removed_idx = None;
            for i in 0..part_count as usize {
                let entry = WiiPartEntry::read_from_bytes(&entry_buf[i * 8..i * 8 + 8])
                    .map_err(|_| io::Error::other("Invalid partition entry"))?;
                if PartitionKind::from(entry.kind.get()) == kind {
                    removed_idx = Some(i);
                    break;
                }
            }
            let Some(removed_idx) = removed_idx else { continue };

            let new_count = part_count - 1;
            edits.push((WII_PART_GROUP_OFF + group_idx * 8, new_count.to_be_bytes().to_vec()));

            // Shift every entry after the removed one back by one slot
            let remaining = entry_buf[(removed_idx + 1) * 8..part_count as usize * 8].to_vec();
            if !remaining.is_empty() {
                edits.push((entry_off + removed_idx as u64 * 8, remaining));
            }
        }

        if edits.is_empty() { Ok(None) } else { Ok(Some(Self { edits })) }
    }

    pub(crate) fn overlaps(&self, block_start: u64, block_len: u64) -> bool {
        let block_end = block_start + block_len;
        self.edits
            .iter()
            .any(|(offset, data)| *offset < block_end && offset + data.len() as u64 > block_start)
    }

    pub(crate) fn apply(&self, block: &mut [u8], block_start: u64) {
        let block_end = block_start + block.len() as u64;
        for (offset, data) in &self.edits {
            let edit_end = offset + data.len() as u64;
            if *offset >= block_end || edit_end <= block_start {
                continue;
            }
            let overlap_start = (*offset).max(block_start);
            let overlap_end = edit_end.min(block_end);
            let src = (overlap_start - offset) as usize..(overlap_end - offset) as usize;
            let dst = (overlap_start - block_start) as usize..(overlap_end - block_start) as usize;
            block[dst].copy_from_slice(&data[src]);
        }
    }
}

/// Check if a block is zeroed, junk data, or safe to drop because it
/// belongs to a partition being scrubbed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_block(
    buf: &[u8],
    decrypted_block: &mut [u8],
    input_position: u64,
    partition_info: &[PartitionInfo],
    partition_usage: &[PartitionUsage],
    lfg: &mut LaggedFibonacci,
    disc_id: [u8; 4],
    disc_num: u8,
    scrub_partition_kind: Option<PartitionKind>,
) -> io::Result<CheckBlockResult> {
    let start_sector = (input_position / SECTOR_SIZE as u64) as u32;
    let end_sector = ((input_position + buf.len() as u64) / SECTOR_SIZE as u64) as u32;

    // A block fully inside the scrubbed partition(s) can be dropped outright.
    // A block that only partially overlaps can still be dropped, but only
    // if the leftover is droppable, and isn't part of a different partition.
    if let Some(kind) = scrub_partition_kind {
        let mut covered: Vec<Range<u32>> = partition_info
            .iter()
            .filter(|p| p.kind == kind)
            .map(|p| p.start_sector.max(start_sector)..p.data_end_sector.min(end_sector))
            .filter(|r| !r.is_empty())
            .collect();
        if !covered.is_empty() {
            covered.sort_by_key(|r| r.start);
            let mut merged: Vec<Range<u32>> = Vec::with_capacity(covered.len());
            for r in covered {
                match merged.last_mut() {
                    Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
                    _ => merged.push(r),
                }
            }

            if merged.len() == 1 && merged[0].start <= start_sector && merged[0].end >= end_sector {
                return Ok(CheckBlockResult::Zeroed);
            }

            let mut leftover_droppable = |range: Range<u32>| -> bool {
                if range.is_empty() {
                    return true;
                }
                if partition_info.iter().any(|p| {
                    p.kind != kind && range.start < p.data_end_sector && range.end > p.start_sector
                }) {
                    return false;
                }
                let rel_start = (range.start - start_sector) as usize * SECTOR_SIZE;
                let rel_end = (range.end - start_sector) as usize * SECTOR_SIZE;
                let chunk = &buf[rel_start..rel_end];
                if chunk.iter().all(|&b| b == 0) {
                    return true;
                }
                let chunk_pos = input_position + rel_start as u64;
                lfg.check_sector_chunked(chunk, disc_id, disc_num, chunk_pos) == chunk.len()
            };

            let mut cursor = start_sector;
            let mut ok = true;
            for r in &merged {
                if !leftover_droppable(cursor..r.start) {
                    ok = false;
                    break;
                }
                cursor = r.end;
            }
            if ok && leftover_droppable(cursor..end_sector) {
                return Ok(CheckBlockResult::Zeroed);
            }
        }
    }

    if let Some((partition_idx, partition)) = partition_info.iter().enumerate().find(|(_, p)| {
        p.has_hashes && start_sector >= p.data_start_sector && end_sector < p.data_end_sector
    }) {
        if input_position % SECTOR_SIZE as u64 != 0 {
            return Err(io::Error::other("Partition block not aligned to sector boundary"));
        }
        if buf.len() % SECTOR_SIZE != 0 {
            return Err(io::Error::other("Partition block not a multiple of sector size"));
        }
        let block = if partition.has_encryption {
            if decrypted_block.len() < buf.len() {
                return Err(io::Error::other("Decrypted block buffer too small"));
            }
            for i in 0..buf.len() / SECTOR_SIZE {
                decrypt_sector_b2b(
                    array_ref![buf, SECTOR_SIZE * i, SECTOR_SIZE],
                    array_ref_mut![decrypted_block, SECTOR_SIZE * i, SECTOR_SIZE],
                    &partition.key,
                );
            }
            &decrypted_block[..buf.len()]
        } else {
            buf
        };

        let partition_start = partition.data_start_sector as u64 * SECTOR_SIZE as u64;
        let partition_offset =
            ((input_position - partition_start) / SECTOR_SIZE as u64) * SECTOR_DATA_SIZE as u64;
        let num_sectors = block.len() as u64 / SECTOR_SIZE as u64;
        let block_end = partition_offset + num_sectors * SECTOR_DATA_SIZE as u64;

        // Only drop blocks outside of used ranges, regardless of content
        if !partition_usage[partition_idx].overlaps_used(partition_offset, block_end) {
            if sector_data_iter(block).all(|sector_data| sector_data.iter().all(|&b| b == 0)) {
                return Ok(CheckBlockResult::Zeroed);
            }
            // Junk data within a partition is seeded from the partition's own disc header,
            // which is also what junk regeneration uses at read time. It usually matches the
            // outer disc header, but nothing guarantees that.
            let partition_header = partition.disc_header();
            let disc_id = *array_ref![partition_header.game_id, 0, 4];
            let disc_num = partition_header.disc_num;
            if sector_data_iter(block).enumerate().all(|(i, sector_data)| {
                let sector_offset = partition_offset + i as u64 * SECTOR_DATA_SIZE as u64;
                lfg.check_sector_chunked(sector_data, disc_id, disc_num, sector_offset)
                    == sector_data.len()
            }) {
                return Ok(CheckBlockResult::Junk);
            }
        }
    } else {
        if buf.iter().all(|&b| b == 0) {
            return Ok(CheckBlockResult::Zeroed);
        }
        if lfg.check_sector_chunked(buf, disc_id, disc_num, input_position) == buf.len() {
            return Ok(CheckBlockResult::Junk);
        }
    }
    Ok(CheckBlockResult::Normal)
}

#[inline]
fn sector_data_iter(buf: &[u8]) -> impl Iterator<Item = &[u8; SECTOR_DATA_SIZE]> {
    buf.chunks_exact(SECTOR_SIZE).map(|chunk| (&chunk[HASHES_SIZE..]).try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zerocopy::FromZeros;

    use super::*;
    use crate::{
        common::PartitionKind,
        disc::{BOOT_SIZE, wii::WiiPartitionHeader},
        util::lfg::LaggedFibonacci,
    };

    const PARTITION_ID: [u8; 4] = *b"GKQJ";
    const OUTER_ID: [u8; 4] = *b"GKQE";

    fn partition_info() -> PartitionInfo {
        let mut raw_boot = [0u8; BOOT_SIZE];
        raw_boot[..4].copy_from_slice(&PARTITION_ID);
        raw_boot[4..6].copy_from_slice(b"01");
        PartitionInfo {
            index: 0,
            kind: PartitionKind::Data,
            start_sector: 0,
            data_start_sector: 0,
            data_end_sector: 16,
            key: [0u8; 16],
            header: Arc::new(WiiPartitionHeader::new_zeroed()),
            has_encryption: false,
            has_hashes: true,
            raw_boot: Arc::new(raw_boot),
            raw_fst: None,
        }
    }

    fn junk_sector(disc_id: [u8; 4]) -> Vec<u8> {
        let mut buf = vec![0u8; SECTOR_SIZE];
        let mut lfg = LaggedFibonacci::default();
        lfg.fill_sector_chunked(&mut buf[HASHES_SIZE..], disc_id, 0, 0);
        buf
    }

    fn run_check_block(buf: &[u8], partition: &PartitionInfo) -> CheckBlockResult {
        let mut decrypted = vec![0u8; buf.len()];
        let partition_usage = [PartitionUsage::new(partition)];
        check_block(
            buf,
            &mut decrypted,
            0,
            std::slice::from_ref(partition),
            &partition_usage,
            &mut LaggedFibonacci::default(),
            OUTER_ID,
            0,
            None,
        )
        .unwrap()
    }

    /// Junk inside a partition is seeded from the partition's own disc header, which may differ
    /// from the outer disc header (and is what read-time regeneration uses).
    #[test]
    fn check_block_seeds_partition_junk_from_partition_header() {
        let partition = partition_info();
        let result = run_check_block(&junk_sector(PARTITION_ID), &partition);
        assert!(matches!(result, CheckBlockResult::Junk));
    }

    /// Junk generated from the outer disc header's ID must NOT be detected inside a partition
    /// with a different ID: read-time regeneration would produce different bytes.
    #[test]
    fn check_block_rejects_outer_header_junk_in_partition() {
        let partition = partition_info();
        let result = run_check_block(&junk_sector(OUTER_ID), &partition);
        assert!(matches!(result, CheckBlockResult::Normal));
    }
}
