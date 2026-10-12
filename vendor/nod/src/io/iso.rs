use std::{io, io::BufRead};

use crate::{
    Result, ResultContext,
    common::Format,
    disc::{SECTOR_SIZE, reader::DiscReader, writer::DiscWriter},
    io::block::{Block, BlockKind, BlockReader},
    read::{DiscMeta, DiscStream},
    util::digest::DigestManager,
    write::{DataCallback, DiscFinalization, DiscWriterWeight, ProcessOptions},
};

#[derive(Clone)]
pub struct BlockReaderISO {
    inner: Box<dyn DiscStream>,
    disc_size: u64,
}

impl BlockReaderISO {
    pub fn new(mut inner: Box<dyn DiscStream>) -> Result<Box<Self>> {
        let disc_size = inner.stream_len().context("Determining stream length")?;
        Ok(Box::new(Self { inner, disc_size }))
    }
}

impl BlockReader for BlockReaderISO {
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block> {
        let pos = sector as u64 * SECTOR_SIZE as u64;
        if pos >= self.disc_size {
            // End of file
            return Ok(Block::sector(sector, BlockKind::None));
        }

        if pos + SECTOR_SIZE as u64 > self.disc_size {
            // If the last block is not a full sector, fill the rest with zeroes
            let read = (self.disc_size - pos) as usize;
            self.inner.read_exact_at(&mut out[..read], pos)?;
            out[read..].fill(0);
        } else {
            self.inner.read_exact_at(out, pos)?;
        }

        Ok(Block::sector(sector, BlockKind::Raw))
    }

    fn block_size(&self) -> u32 { SECTOR_SIZE as u32 }

    fn meta(&self) -> DiscMeta {
        DiscMeta {
            format: Format::Iso,
            lossless: true,
            disc_size: Some(self.disc_size),
            ..Default::default()
        }
    }
}

impl DiscWriter for DiscReader {
    fn process(
        &self,
        data_callback: &mut DataCallback,
        options: &ProcessOptions,
    ) -> Result<DiscFinalization> {
        let mut reader = self.clone();
        let digest = DigestManager::new(options);
        loop {
            let pos = reader.position();
            let data = reader
                .fill_buf_internal()
                .with_context(|| format!("Reading disc data at offset {pos}"))?;
            let len = data.len();
            if len == 0 {
                break;
            }
            // Update hashers
            digest.send(data.clone());
            data_callback(data, pos + len as u64, reader.disc_size())
                .context("Failed to write disc data")?;
            reader.consume(len);
        }
        let mut finalization = DiscFinalization::default();
        finalization.apply_digests(&digest.finish());
        Ok(finalization)
    }

    fn progress_bound(&self) -> u64 { self.disc_size() }

    fn weight(&self) -> DiscWriterWeight { DiscWriterWeight::Light }
}
