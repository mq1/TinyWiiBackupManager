use std::{
    io,
    io::{BufRead, Seek, SeekFrom},
    sync::Arc,
};

use zerocopy::FromZeros;

use crate::{
    Result,
    common::KeyBytes,
    disc::{DiscHeader, SECTOR_SIZE, wii::SECTOR_DATA_SIZE},
    io::block::{Block, BlockReader},
    read::{PartitionMeta, PartitionReader},
    util::impl_read_for_bufread,
};

#[derive(Clone)]
pub enum DirectDiscReaderMode {
    Raw,
    Partition { disc_header: Arc<DiscHeader>, data_start_sector: u32, key: KeyBytes },
}

/// Simplified disc reader that uses a block reader directly.
///
/// This is used to read disc and partition metadata before we can construct a full disc reader.
pub struct DirectDiscReader {
    io: Box<dyn BlockReader>,
    block: Block,
    block_buf: Box<[u8]>,
    block_decrypted: bool,
    pos: u64,
    mode: DirectDiscReaderMode,
}

impl Clone for DirectDiscReader {
    fn clone(&self) -> Self {
        Self {
            io: self.io.clone(),
            block: Block::default(),
            block_buf: <[u8]>::new_box_zeroed_with_elems(self.block_buf.len()).unwrap(),
            block_decrypted: false,
            pos: 0,
            mode: self.mode.clone(),
        }
    }
}

impl DirectDiscReader {
    pub fn new(inner: Box<dyn BlockReader>) -> Result<Box<Self>> {
        let block_size = inner.block_size() as usize;
        Ok(Box::new(Self {
            io: inner,
            block: Block::default(),
            block_buf: <[u8]>::new_box_zeroed_with_elems(block_size)?,
            block_decrypted: false,
            pos: 0,
            mode: DirectDiscReaderMode::Raw,
        }))
    }

    pub fn reset(&mut self, mode: DirectDiscReaderMode) {
        self.block = Block::default();
        self.block_decrypted = false;
        self.pos = 0;
        self.mode = mode;
    }

    pub fn into_inner(self) -> Box<dyn BlockReader> { self.io }
}

impl BufRead for DirectDiscReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match &self.mode {
            DirectDiscReaderMode::Raw => {
                // Read new block if necessary
                let sector = (self.pos / SECTOR_SIZE as u64) as u32;
                if self.block_decrypted || !self.block.contains(sector) {
                    self.block = self.io.read_block(self.block_buf.as_mut(), sector)?;
                    self.block_decrypted = false;
                }
                self.block.data(self.block_buf.as_ref(), self.pos)
            }
            DirectDiscReaderMode::Partition { disc_header, data_start_sector, key } => {
                let has_encryption = disc_header.has_partition_encryption();
                let has_hashes = disc_header.has_partition_hashes();
                let part_sector = if has_hashes {
                    (self.pos / SECTOR_DATA_SIZE as u64) as u32
                } else {
                    (self.pos / SECTOR_SIZE as u64) as u32
                };

                // Read new block if necessary
                let abs_sector = data_start_sector + part_sector;
                if !self.block.contains(abs_sector) {
                    self.block = self.io.read_block(self.block_buf.as_mut(), abs_sector)?;
                    self.block_decrypted = false;
                }

                // Allow reusing the same block from raw mode, just decrypt it if necessary
                if !self.block_decrypted {
                    self.block
                        .decrypt_block(self.block_buf.as_mut(), has_encryption.then_some(*key))?;
                    self.block_decrypted = true;
                }

                self.block.partition_data(
                    self.block_buf.as_ref(),
                    self.pos,
                    *data_start_sector,
                    has_hashes,
                )
            }
        }
    }

    #[inline]
    fn consume(&mut self, amt: usize) { self.pos += amt as u64; }
}

impl_read_for_bufread!(DirectDiscReader);

impl Seek for DirectDiscReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(v) => v,
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "DirectDiscReader: SeekFrom::End is not supported",
                ));
            }
            SeekFrom::Current(v) => self.pos.saturating_add_signed(v),
        };
        Ok(self.pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> { Ok(self.pos) }
}

impl PartitionReader for DirectDiscReader {
    fn is_wii(&self) -> bool { unimplemented!() }

    fn meta(&mut self) -> Result<PartitionMeta> { unimplemented!() }
}
