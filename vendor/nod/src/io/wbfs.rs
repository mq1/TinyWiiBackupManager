use std::{
    io,
    io::{Seek, SeekFrom},
    mem::size_of,
    sync::Arc,
};

use bytes::{BufMut, Bytes, BytesMut};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, big_endian::*};

use crate::{
    Error, Result, ResultContext,
    common::{Compression, Format, MagicBytes, PartitionKind},
    disc::{
        SECTOR_SIZE,
        reader::DiscReader,
        writer::{
            BlockProcessor, BlockResult, CheckBlockResult, DiscWriter, PartitionRemoval,
            PartitionUsage, check_block, par_process, read_block,
        },
    },
    io::{
        block::{Block, BlockKind, BlockReader, WBFS_MAGIC},
        nkit::{JunkBits, NKitHeader},
    },
    read::{DiscMeta, DiscStream},
    util::{
        array_ref,
        digest::DigestManager,
        lfg::LaggedFibonacci,
        read::{read_arc_slice_at, read_at, read_box_slice_at},
    },
    write::{
        DataCallback, DiscFinalization, DiscWriterWeight, FormatOptions, ProcessOptions, ScrubLevel,
    },
};

#[derive(Debug, Clone, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct WBFSHeader {
    magic: MagicBytes,
    num_sectors: U32,
    sector_size_shift: u8,
    block_size_shift: u8,
    version: u8,
    _pad: u8,
}

impl WBFSHeader {
    fn sector_size(&self) -> u32 { 1 << self.sector_size_shift }

    fn block_size(&self) -> u32 { 1 << self.block_size_shift }

    fn max_blocks(&self) -> u32 { NUM_WII_SECTORS >> (self.block_size_shift - 15) }
}

const DISC_HEADER_SIZE: usize = 0x100;
const NUM_WII_SECTORS: u32 = 143432 * 2; // Double layer discs
const NKIT_HEADER_OFFSET: u64 = 0x10000;

#[derive(Clone)]
pub struct BlockReaderWBFS {
    inner: Box<dyn DiscStream>,
    /// WBFS header
    header: WBFSHeader,
    /// Map of Wii LBAs to WBFS LBAs
    block_map: Arc<[U16]>,
    /// Optional NKit header
    nkit_header: Option<NKitHeader>,
}

impl BlockReaderWBFS {
    pub fn new(mut inner: Box<dyn DiscStream>) -> Result<Box<Self>> {
        let header: WBFSHeader = read_at(inner.as_mut(), 0).context("Reading WBFS header")?;
        if header.magic != WBFS_MAGIC {
            return Err(Error::DiscFormat("Invalid WBFS magic".to_string()));
        }
        let file_len = inner.stream_len().context("Determining stream length")?;
        let expected_file_len = header.num_sectors.get() as u64 * header.sector_size() as u64;
        if file_len != expected_file_len {
            return Err(Error::DiscFormat(format!(
                "Invalid WBFS file size: {}, expected {}",
                file_len, expected_file_len
            )));
        }

        let disc_table: Box<[u8]> = read_box_slice_at(
            inner.as_mut(),
            header.sector_size() as usize - size_of::<WBFSHeader>(),
            size_of::<WBFSHeader>() as u64,
        )
        .context("Reading WBFS disc table")?;
        if disc_table[0] != 1 {
            return Err(Error::DiscFormat("WBFS doesn't contain a disc".to_string()));
        }
        if disc_table[1../*max_disc as usize*/].iter().any(|&x| x != 0) {
            return Err(Error::DiscFormat("Only single WBFS discs are supported".to_string()));
        }

        // Read WBFS LBA map
        let block_map: Arc<[U16]> = read_arc_slice_at(
            inner.as_mut(),
            header.max_blocks() as usize,
            header.sector_size() as u64 + DISC_HEADER_SIZE as u64,
        )
        .context("Reading WBFS LBA table")?;

        // Read NKit header if present (always at 0x10000)
        let nkit_header = NKitHeader::try_read_from(
            inner.as_mut(),
            NKIT_HEADER_OFFSET,
            header.block_size(),
            true,
        );

        Ok(Box::new(Self { inner, header, block_map, nkit_header }))
    }
}

impl BlockReader for BlockReaderWBFS {
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block> {
        let block_size = self.header.block_size();
        let block_idx = ((sector as u64 * SECTOR_SIZE as u64) / block_size as u64) as u32;
        if block_idx >= self.header.max_blocks() {
            // Out of bounds
            return Ok(Block::new(block_idx, block_size, BlockKind::None));
        }

        // Find the block in the map
        let phys_block = self.block_map[block_idx as usize].get();
        if phys_block == 0 {
            // Check if block is junk data
            if self.nkit_header.as_ref().and_then(|h| h.is_junk_block(block_idx)).unwrap_or(false) {
                return Ok(Block::new(block_idx, block_size, BlockKind::Junk));
            }

            // Otherwise, read zeroes
            return Ok(Block::new(block_idx, block_size, BlockKind::Zero));
        }

        // Read block
        let block_start = block_size as u64 * phys_block as u64;
        self.inner.read_exact_at(out, block_start)?;

        Ok(Block::new(block_idx, block_size, BlockKind::Raw))
    }

    fn block_size(&self) -> u32 { self.header.block_size() }

    fn meta(&self) -> DiscMeta {
        let mut result = DiscMeta {
            format: Format::Wbfs,
            block_size: Some(self.header.block_size()),
            ..Default::default()
        };
        if let Some(nkit_header) = &self.nkit_header {
            nkit_header.apply(&mut result);
        }
        result
    }
}

struct BlockProcessorWBFS {
    inner: DiscReader,
    header: WBFSHeader,
    partition_usage: Arc<[PartitionUsage]>,
    partition_removal: Option<Arc<PartitionRemoval>>,
    decrypted_block: Box<[u8]>,
    lfg: LaggedFibonacci,
    disc_id: [u8; 4],
    disc_num: u8,
    scrub_partition_kind: Option<PartitionKind>,
}

impl Clone for BlockProcessorWBFS {
    fn clone(&self) -> Self {
        let block_size = self.header.block_size() as usize;
        Self {
            inner: self.inner.clone(),
            header: self.header.clone(),
            partition_usage: self.partition_usage.clone(),
            partition_removal: self.partition_removal.clone(),
            decrypted_block: <[u8]>::new_box_zeroed_with_elems(block_size).unwrap(),
            lfg: LaggedFibonacci::default(),
            disc_id: self.disc_id,
            disc_num: self.disc_num,
            scrub_partition_kind: self.scrub_partition_kind,
        }
    }
}

impl BlockProcessor for BlockProcessorWBFS {
    type BlockMeta = CheckBlockResult;

    fn process_block(&mut self, block_idx: u32) -> io::Result<BlockResult<Self::BlockMeta>> {
        let block_size = self.header.block_size() as usize;
        let input_position = block_idx as u64 * block_size as u64;
        self.inner.seek(SeekFrom::Start(input_position))?;
        let (block_data, disc_data) = read_block(&mut self.inner, block_size)?;

        // If this block holds part of the scrubbed partition's table entry,
        // remove it. Other partitions are left untouched.
        let (block_data, disc_data) = match &self.partition_removal {
            Some(patch) if patch.overlaps(input_position, block_data.len() as u64) => {
                let mut buf = BytesMut::from(block_data.as_ref());
                patch.apply(&mut buf, input_position);
                let block_data = buf.freeze();
                let disc_data = block_data.slice(0..disc_data.len());
                (block_data, disc_data)
            }
            _ => (block_data, disc_data),
        };

        // Check if block is zeroed or junk
        let result = match check_block(
            disc_data.as_ref(),
            &mut self.decrypted_block,
            input_position,
            self.inner.partitions(),
            &self.partition_usage,
            &mut self.lfg,
            self.disc_id,
            self.disc_num,
            self.scrub_partition_kind,
        )? {
            CheckBlockResult::Normal => {
                BlockResult { block_idx, disc_data, block_data, meta: CheckBlockResult::Normal }
            }
            CheckBlockResult::Zeroed => BlockResult {
                block_idx,
                disc_data,
                block_data: Bytes::new(),
                meta: CheckBlockResult::Zeroed,
            },
            CheckBlockResult::Junk => BlockResult {
                block_idx,
                disc_data,
                block_data: Bytes::new(),
                meta: CheckBlockResult::Junk,
            },
        };
        Ok(result)
    }
}

#[derive(Clone)]
pub struct DiscWriterWBFS {
    inner: DiscReader,
    header: WBFSHeader,
    disc_table: Box<[u8]>,
    block_count: u16,
}

pub const DEFAULT_BLOCK_SIZE: u32 = 0x200000; // 2 MiB

impl DiscWriterWBFS {
    pub fn new(mut inner: DiscReader, options: &FormatOptions) -> Result<Box<dyn DiscWriter>> {
        if options.format != Format::Wbfs {
            return Err(Error::DiscFormat("Invalid format for WBFS writer".to_string()));
        }
        if options.compression != Compression::None {
            return Err(Error::DiscFormat("WBFS does not support compression".to_string()));
        }
        let block_size = options.block_size;
        if block_size < SECTOR_SIZE as u32 || block_size % SECTOR_SIZE as u32 != 0 {
            return Err(Error::DiscFormat("Invalid block size for WBFS".to_string()));
        }
        let sector_size = 512u32;

        let disc_size = inner.disc_size();
        let block_count = disc_size.div_ceil(block_size as u64);
        if block_count > u16::MAX as u64 {
            return Err(Error::DiscFormat("Block size too small".to_string()));
        }
        let block_count = block_count as u16;

        // Create header
        let header = WBFSHeader {
            magic: WBFS_MAGIC,
            num_sectors: 0.into(), // Written during finalization
            sector_size_shift: sector_size.trailing_zeros() as u8,
            block_size_shift: block_size.trailing_zeros() as u8,
            version: 1,
            _pad: 0,
        };

        // Create disc table
        let mut disc_table =
            <[u8]>::new_box_zeroed_with_elems(sector_size as usize - size_of::<WBFSHeader>())?;
        disc_table[0] = 1;

        let mut header_size = size_of::<WBFSHeader>();
        header_size += size_of_val(disc_table.as_ref());
        header_size += DISC_HEADER_SIZE;
        header_size += header.max_blocks() as usize * size_of::<U16>();
        if header_size > block_size as usize {
            return Err(Error::Other("WBFS info too large for block".to_string()));
        }

        inner.rewind().context("Seeking to start")?;
        Ok(Box::new(Self { inner, header, disc_table, block_count }))
    }
}

impl DiscWriter for DiscWriterWBFS {
    fn process(
        &self,
        data_callback: &mut DataCallback,
        options: &ProcessOptions,
    ) -> Result<DiscFinalization> {
        let block_size = self.header.block_size();
        let max_blocks = self.header.max_blocks();
        let mut block_map = <[U16]>::new_box_zeroed_with_elems(max_blocks as usize)?;

        let disc_size = self.inner.disc_size();
        let mut header_data = BytesMut::with_capacity(block_size as usize);
        header_data.put_slice(self.header.as_bytes());
        header_data.put_slice(self.disc_table.as_ref());
        header_data.put_slice(&self.inner.header().as_bytes()[..DISC_HEADER_SIZE]);
        header_data.put_slice(block_map.as_bytes());
        header_data.resize(block_size as usize, 0);
        data_callback(header_data.freeze(), 0, disc_size).context("Failed to write header")?;

        // Determine junk data values
        let disc_header = self.inner.header();
        let disc_id = *array_ref![disc_header.game_id, 0, 4];
        let disc_num = disc_header.disc_num;

        // Build each partition's usage map once, rather than per block
        let partition_usage: Arc<[PartitionUsage]> =
            self.inner.partitions().iter().map(PartitionUsage::new).collect();

        // Remove the partition table entry when scrubbing
        let scrub_enabled = options.scrub == ScrubLevel::UpdatePartition;
        let scrub_partition_kind = scrub_enabled.then_some(PartitionKind::Update);
        let partition_removal: Option<Arc<PartitionRemoval>> =
            if let Some(kind) = scrub_partition_kind {
                let mut disc = self.inner.clone();
                PartitionRemoval::new(&mut disc, kind)
                    .context("Building partition removal")?
                    .map(Arc::new)
            } else {
                None
            };

        // Create hashers
        let digest = DigestManager::new(options);
        let mut junk_bits = JunkBits::new(block_size);
        let mut input_position = 0;

        let mut phys_block = 1;
        par_process(
            BlockProcessorWBFS {
                inner: self.inner.clone(),
                header: self.header.clone(),
                partition_usage,
                partition_removal,
                decrypted_block: <[u8]>::new_box_zeroed_with_elems(block_size as usize).unwrap(),
                lfg: LaggedFibonacci::default(),
                disc_id,
                disc_num,
                scrub_partition_kind,
            },
            self.block_count as u32,
            #[cfg(feature = "threading")]
            options.processor_threads,
            |block| -> Result<()> {
                // Update hashers
                let disc_data_len = block.disc_data.len() as u64;
                digest.send(block.disc_data);

                // Check if block is zeroed or junk
                match block.meta {
                    CheckBlockResult::Normal => {
                        block_map[block.block_idx as usize] = phys_block.into();
                        phys_block += 1;
                    }
                    CheckBlockResult::Zeroed => {}
                    CheckBlockResult::Junk => {
                        junk_bits.set(block.block_idx, true);
                    }
                }

                input_position += disc_data_len;
                data_callback(block.block_data.clone(), input_position, disc_size)
                    .with_context(|| format!("Failed to write block {}", block.block_idx))?;
                Ok(())
            },
        )?;

        // Collect hash results
        let digest_results = digest.finish();
        let mut nkit_header = NKitHeader {
            version: 2,
            size: Some(disc_size),
            crc32: None,
            md5: None,
            sha1: None,
            xxh64: None,
            junk_bits: Some(junk_bits),
            encrypted: true,
        };
        nkit_header.apply_digests(&digest_results);

        // Update header
        let mut header = self.header.clone();
        header.num_sectors = (((phys_block as u64 * header.block_size() as u64)
            / header.sector_size() as u64) as u32)
            .into();
        let mut header_data = BytesMut::with_capacity(block_size as usize);
        header_data.put_slice(header.as_bytes());
        header_data.put_slice(&self.disc_table);
        header_data.put_slice(&self.inner.header().as_bytes()[..DISC_HEADER_SIZE]);
        header_data.put_slice(block_map.as_bytes());
        header_data.resize(NKIT_HEADER_OFFSET as usize, 0);
        let mut w = header_data.writer();
        nkit_header.write_to(&mut w).context("Writing NKit header")?;
        let header_data = w.into_inner().freeze();

        let mut finalization = DiscFinalization { header: header_data, ..Default::default() };
        finalization.apply_digests(&digest_results);
        Ok(finalization)
    }

    fn progress_bound(&self) -> u64 { self.inner.disc_size() }

    fn weight(&self) -> DiscWriterWeight { DiscWriterWeight::Medium }
}
