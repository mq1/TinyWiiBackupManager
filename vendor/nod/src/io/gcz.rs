use std::{
    io,
    io::{Seek, SeekFrom},
    mem::size_of,
    sync::Arc,
};

use adler2::adler32_slice;
use bytes::{BufMut, Bytes, BytesMut};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, little_endian::*};

use crate::{
    Error, Result, ResultContext,
    common::{Compression, Format, MagicBytes},
    disc::{
        SECTOR_SIZE,
        reader::DiscReader,
        writer::{BlockProcessor, BlockResult, DiscWriter, par_process, read_block},
    },
    io::block::{Block, BlockKind, BlockReader, GCZ_MAGIC},
    read::{DiscMeta, DiscStream},
    util::{
        compress::{Compressor, DecompressionKind},
        digest::DigestManager,
        read::{read_arc_slice_at, read_at},
        static_assert,
    },
    write::{DataCallback, DiscFinalization, DiscWriterWeight, FormatOptions, ProcessOptions},
};

/// GCZ header (little endian)
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct GCZHeader {
    magic: MagicBytes,
    disc_type: U32,
    compressed_size: U64,
    disc_size: U64,
    block_size: U32,
    block_count: U32,
}

static_assert!(size_of::<GCZHeader>() == 32);

pub struct BlockReaderGCZ {
    inner: Box<dyn DiscStream>,
    header: GCZHeader,
    block_map: Arc<[U64]>,
    block_hashes: Arc<[U32]>,
    block_buf: Box<[u8]>,
    data_offset: u64,
}

impl Clone for BlockReaderGCZ {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            header: self.header.clone(),
            block_map: self.block_map.clone(),
            block_hashes: self.block_hashes.clone(),
            block_buf: <[u8]>::new_box_zeroed_with_elems(self.block_buf.len()).unwrap(),
            data_offset: self.data_offset,
        }
    }
}

impl BlockReaderGCZ {
    pub fn new(mut inner: Box<dyn DiscStream>) -> Result<Box<Self>> {
        // Read header
        let header: GCZHeader = read_at(inner.as_mut(), 0).context("Reading GCZ header")?;
        if header.magic != GCZ_MAGIC {
            return Err(Error::DiscFormat("Invalid GCZ magic".to_string()));
        }

        // Read block map and hashes
        let block_count = header.block_count.get();
        let block_map =
            read_arc_slice_at(inner.as_mut(), block_count as usize, size_of::<GCZHeader>() as u64)
                .context("Reading GCZ block map")?;
        let block_hashes = read_arc_slice_at(
            inner.as_mut(),
            block_count as usize,
            size_of::<GCZHeader>() as u64 + block_count as u64 * 8,
        )
        .context("Reading GCZ block hashes")?;

        // header + block_count * (u64 + u32)
        let data_offset = size_of::<GCZHeader>() as u64 + block_count as u64 * 12;
        let block_buf = <[u8]>::new_box_zeroed_with_elems(header.block_size.get() as usize)?;
        Ok(Box::new(Self { inner, header, block_map, block_hashes, block_buf, data_offset }))
    }
}

impl BlockReader for BlockReaderGCZ {
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block> {
        let block_size = self.header.block_size.get();
        let block_idx = ((sector as u64 * SECTOR_SIZE as u64) / block_size as u64) as u32;
        if block_idx >= self.header.block_count.get() {
            // Out of bounds
            return Ok(Block::new(block_idx, block_size, BlockKind::None));
        }

        // Find block offset and size
        let mut file_offset = self.block_map[block_idx as usize].get();
        let mut compressed = true;
        if file_offset & (1 << 63) != 0 {
            file_offset &= !(1 << 63);
            compressed = false;
        }
        let compressed_size = ((self
            .block_map
            .get(block_idx as usize + 1)
            .unwrap_or(&self.header.compressed_size)
            .get()
            & !(1 << 63))
            - file_offset) as usize;
        if compressed_size > block_size as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Compressed block size exceeds block size: {} > {}",
                    compressed_size, block_size
                ),
            ));
        } else if !compressed && compressed_size != block_size as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Uncompressed block size does not match block size: {} != {}",
                    compressed_size, block_size
                ),
            ));
        }

        // Read block
        self.inner.read_exact_at(
            &mut self.block_buf[..compressed_size],
            self.data_offset + file_offset,
        )?;

        // Verify block checksum
        let checksum = adler32_slice(&self.block_buf[..compressed_size]);
        let expected_checksum = self.block_hashes[block_idx as usize].get();
        if checksum != expected_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Block {} checksum mismatch: {:#010x} != {:#010x}",
                    block_idx, checksum, expected_checksum
                ),
            ));
        }

        if compressed {
            // Decompress block
            let out_len =
                DecompressionKind::Deflate.decompress(&self.block_buf[..compressed_size], out)?;
            if out_len != block_size as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Block {} decompression failed: in: {}, out: {}",
                        block_idx, compressed_size, out_len
                    ),
                ));
            }
        } else {
            // Copy uncompressed block
            out.copy_from_slice(self.block_buf.as_ref());
        }

        Ok(Block::new(block_idx, block_size, BlockKind::Raw))
    }

    fn block_size(&self) -> u32 { self.header.block_size.get() }

    fn meta(&self) -> DiscMeta {
        DiscMeta {
            format: Format::Gcz,
            compression: Compression::Deflate(0),
            block_size: Some(self.header.block_size.get()),
            lossless: true,
            disc_size: Some(self.header.disc_size.get()),
            ..Default::default()
        }
    }
}

struct BlockProcessorGCZ {
    inner: DiscReader,
    header: GCZHeader,
    compressor: Compressor,
}

impl Clone for BlockProcessorGCZ {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            header: self.header.clone(),
            compressor: self.compressor.clone(),
        }
    }
}

struct BlockMetaGCZ {
    is_compressed: bool,
    block_hash: u32,
}

impl BlockProcessor for BlockProcessorGCZ {
    type BlockMeta = BlockMetaGCZ;

    fn process_block(&mut self, block_idx: u32) -> io::Result<BlockResult<Self::BlockMeta>> {
        let block_size = self.header.block_size.get();
        self.inner.seek(SeekFrom::Start(block_idx as u64 * block_size as u64))?;
        let (mut block_data, disc_data) = read_block(&mut self.inner, block_size as usize)?;

        // Try to compress block
        let is_compressed = if self.compressor.compress(&block_data)? {
            println!("Compressed block {} to {}", block_idx, self.compressor.buffer.len());
            block_data = Bytes::copy_from_slice(self.compressor.buffer.as_slice());
            true
        } else {
            false
        };

        let block_hash = adler32_slice(block_data.as_ref());
        Ok(BlockResult {
            block_idx,
            disc_data,
            block_data,
            meta: BlockMetaGCZ { is_compressed, block_hash },
        })
    }
}

#[derive(Clone)]
pub struct DiscWriterGCZ {
    inner: DiscReader,
    header: GCZHeader,
    compression: Compression,
}

pub const DEFAULT_BLOCK_SIZE: u32 = 0x8000; // 32 KiB

// Level 0 will be converted to the default level in [`Compression::validate_level`]
pub const DEFAULT_COMPRESSION: Compression = Compression::Deflate(0);

impl DiscWriterGCZ {
    pub fn new(inner: DiscReader, options: &FormatOptions) -> Result<Box<dyn DiscWriter>> {
        if options.format != Format::Gcz {
            return Err(Error::DiscFormat("Invalid format for GCZ writer".to_string()));
        }
        if !matches!(options.compression, Compression::Deflate(_)) {
            return Err(Error::DiscFormat(format!(
                "Unsupported compression for GCZ: {:?}",
                options.compression
            )));
        }

        let block_size = options.block_size;
        if block_size < SECTOR_SIZE as u32 || block_size % SECTOR_SIZE as u32 != 0 {
            return Err(Error::DiscFormat("Invalid block size for GCZ".to_string()));
        }

        let disc_header = inner.header();
        let disc_size = inner.disc_size();
        let block_count = disc_size.div_ceil(block_size as u64) as u32;

        // Generate header
        let header = GCZHeader {
            magic: GCZ_MAGIC,
            disc_type: if disc_header.is_wii() { 1 } else { 0 }.into(),
            compressed_size: 0.into(), // Written when finalized
            disc_size: disc_size.into(),
            block_size: block_size.into(),
            block_count: block_count.into(),
        };

        Ok(Box::new(Self { inner, header, compression: options.compression }))
    }
}

impl DiscWriter for DiscWriterGCZ {
    fn process(
        &self,
        data_callback: &mut DataCallback,
        options: &ProcessOptions,
    ) -> Result<DiscFinalization> {
        let disc_size = self.header.disc_size.get();
        let block_size = self.header.block_size.get();
        let block_count = self.header.block_count.get();

        // Create hashers
        let digest = DigestManager::new(options);

        // Generate block map and hashes
        let mut block_map = <[U64]>::new_box_zeroed_with_elems(block_count as usize)?;
        let mut block_hashes = <[U32]>::new_box_zeroed_with_elems(block_count as usize)?;

        let header_data_size = size_of::<GCZHeader>()
            + size_of_val(block_map.as_ref())
            + size_of_val(block_hashes.as_ref());
        let mut header_data = BytesMut::with_capacity(header_data_size);
        header_data.put_slice(self.header.as_bytes());
        header_data.resize(header_data_size, 0);
        data_callback(header_data.freeze(), 0, disc_size).context("Failed to write GCZ header")?;

        let mut input_position = 0;
        let mut data_position = 0;
        par_process(
            BlockProcessorGCZ {
                inner: self.inner.clone(),
                header: self.header.clone(),
                compressor: Compressor::new(self.compression, block_size as usize),
            },
            block_count,
            #[cfg(feature = "threading")]
            options.processor_threads,
            |block| {
                // Update hashers
                input_position += block.disc_data.len() as u64;
                digest.send(block.disc_data);

                // Update block map and hash
                let uncompressed_bit = (!block.meta.is_compressed as u64) << 63;
                block_map[block.block_idx as usize] = (data_position | uncompressed_bit).into();
                block_hashes[block.block_idx as usize] = block.meta.block_hash.into();

                // Write block data
                data_position += block.block_data.len() as u64;
                data_callback(block.block_data, input_position, disc_size)
                    .with_context(|| format!("Failed to write block {}", block.block_idx))?;
                Ok(())
            },
        )?;

        // Write updated header, block map and hashes
        let mut header = self.header.clone();
        header.compressed_size = data_position.into();
        let mut header_data = BytesMut::with_capacity(header_data_size);
        header_data.extend_from_slice(header.as_bytes());
        header_data.extend_from_slice(block_map.as_bytes());
        header_data.extend_from_slice(block_hashes.as_bytes());

        let mut finalization =
            DiscFinalization { header: header_data.freeze(), ..Default::default() };
        finalization.apply_digests(&digest.finish());
        Ok(finalization)
    }

    fn progress_bound(&self) -> u64 { self.header.disc_size.get() }

    fn weight(&self) -> DiscWriterWeight { DiscWriterWeight::Heavy }
}
