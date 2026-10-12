use std::{fs, io, io::Read, path::Path};

use dyn_clone::DynClone;

use crate::{
    Error, Result, ResultContext,
    common::{Format, KeyBytes, MagicBytes, PartitionInfo},
    disc::{
        DiscHeader, GCN_MAGIC, SECTOR_SIZE, WII_MAGIC,
        wii::{HASHES_SIZE, SECTOR_DATA_SIZE},
    },
    io::{
        split::SplitFileReader,
        wia::{WIAException, WIAExceptionList},
    },
    read::{DiscMeta, DiscStream},
    util::{
        aes::decrypt_sector,
        array_ref, array_ref_mut,
        lfg::LaggedFibonacci,
        read::{read_at, read_from},
    },
};

/// Block reader trait for reading disc images.
pub trait BlockReader: DynClone + Send {
    /// Reads a block from the disc image containing the specified sector.
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block>;

    /// The block size used for processing. Must be a multiple of the sector size (0x8000).
    fn block_size(&self) -> u32;

    /// Returns extra metadata included in the disc file format, if any.
    fn meta(&self) -> DiscMeta;
}

dyn_clone::clone_trait_object!(BlockReader);

/// Creates a new [`BlockReader`] instance from a stream.
pub fn new(mut stream: Box<dyn DiscStream>) -> Result<Box<dyn BlockReader>> {
    let io: Box<dyn BlockReader> =
        match detect_stream(stream.as_mut()).context("Detecting file type")? {
            Some(Format::Iso) => crate::io::iso::BlockReaderISO::new(stream)?,
            Some(Format::Ciso) => crate::io::ciso::BlockReaderCISO::new(stream)?,
            Some(Format::Gcz) => {
                #[cfg(feature = "compress-zlib")]
                {
                    crate::io::gcz::BlockReaderGCZ::new(stream)?
                }
                #[cfg(not(feature = "compress-zlib"))]
                return Err(Error::DiscFormat("GCZ support is disabled".to_string()));
            }
            Some(Format::Nfs) => {
                return Err(Error::DiscFormat("NFS requires a filesystem path".to_string()));
            }
            Some(Format::Wbfs) => crate::io::wbfs::BlockReaderWBFS::new(stream)?,
            Some(Format::Wia | Format::Rvz) => crate::io::wia::BlockReaderWIA::new(stream)?,
            Some(Format::Tgc) => crate::io::tgc::BlockReaderTGC::new(stream)?,
            None => return Err(Error::DiscFormat("Unknown disc format".to_string())),
        };
    check_block_size(io.as_ref())?;
    Ok(io)
}

/// Creates a new [`BlockReader`] instance from a filesystem path.
pub fn open(filename: &Path) -> Result<Box<dyn BlockReader>> {
    let path_result = fs::canonicalize(filename);
    if let Err(err) = path_result {
        return Err(Error::Io(format!("Failed to open {}", filename.display()), err));
    }
    let path = path_result.as_ref().unwrap();
    let meta = fs::metadata(path);
    if let Err(err) = meta {
        return Err(Error::Io(format!("Failed to open {}", filename.display()), err));
    }
    if !meta.unwrap().is_file() {
        return Err(Error::DiscFormat(format!("Input is not a file: {}", filename.display())));
    }
    let mut stream = Box::new(SplitFileReader::new(filename)?);
    let io: Box<dyn BlockReader> = match detect_stream(stream.as_mut())
        .context("Detecting file type")?
    {
        Some(Format::Iso) => crate::io::iso::BlockReaderISO::new(stream)?,
        Some(Format::Ciso) => crate::io::ciso::BlockReaderCISO::new(stream)?,
        Some(Format::Gcz) => {
            #[cfg(feature = "compress-zlib")]
            {
                crate::io::gcz::BlockReaderGCZ::new(stream)?
            }
            #[cfg(not(feature = "compress-zlib"))]
            return Err(Error::DiscFormat("GCZ support is disabled".to_string()));
        }
        Some(Format::Nfs) => match path.parent() {
            Some(parent) if parent.is_dir() => {
                crate::io::nfs::BlockReaderNFS::new(path.parent().unwrap())?
            }
            _ => {
                return Err(Error::DiscFormat("Failed to locate NFS parent directory".to_string()));
            }
        },
        Some(Format::Tgc) => crate::io::tgc::BlockReaderTGC::new(stream)?,
        Some(Format::Wbfs) => crate::io::wbfs::BlockReaderWBFS::new(stream)?,
        Some(Format::Wia | Format::Rvz) => crate::io::wia::BlockReaderWIA::new(stream)?,
        None => return Err(Error::DiscFormat("Unknown disc format".to_string())),
    };
    check_block_size(io.as_ref())?;
    Ok(io)
}

pub const CISO_MAGIC: MagicBytes = *b"CISO";
pub const GCZ_MAGIC: MagicBytes = [0x01, 0xC0, 0x0B, 0xB1];
pub const NFS_MAGIC: MagicBytes = *b"EGGS";
pub const TGC_MAGIC: MagicBytes = [0xAE, 0x0F, 0x38, 0xA2];
pub const WBFS_MAGIC: MagicBytes = *b"WBFS";
pub const WIA_MAGIC: MagicBytes = *b"WIA\x01";
pub const RVZ_MAGIC: MagicBytes = *b"RVZ\x01";

pub fn detect<R>(stream: &mut R) -> io::Result<Option<Format>>
where R: Read + ?Sized {
    match read_from(stream) {
        Ok(ref magic) => Ok(detect_internal(magic)),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e),
    }
}

fn detect_stream(stream: &mut dyn DiscStream) -> io::Result<Option<Format>> {
    match read_at(stream, 0) {
        Ok(ref magic) => Ok(detect_internal(magic)),
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
        Err(e) => Err(e),
    }
}

fn detect_internal(data: &[u8; 0x20]) -> Option<Format> {
    match *array_ref!(data, 0, 4) {
        CISO_MAGIC => Some(Format::Ciso),
        GCZ_MAGIC => Some(Format::Gcz),
        NFS_MAGIC => Some(Format::Nfs),
        TGC_MAGIC => Some(Format::Tgc),
        WBFS_MAGIC => Some(Format::Wbfs),
        WIA_MAGIC => Some(Format::Wia),
        RVZ_MAGIC => Some(Format::Rvz),
        _ if *array_ref!(data, 0x18, 4) == WII_MAGIC || *array_ref!(data, 0x1C, 4) == GCN_MAGIC => {
            Some(Format::Iso)
        }
        _ => None,
    }
}

fn check_block_size(io: &dyn BlockReader) -> Result<()> {
    if io.block_size() % SECTOR_SIZE as u32 != 0 {
        return Err(Error::DiscFormat(format!(
            "Block size {} is not a multiple of sector size {}",
            io.block_size(),
            SECTOR_SIZE
        )));
    }
    Ok(())
}

/// A block of sectors within a disc image.
#[derive(Debug, Clone, Default)]
pub struct Block {
    /// The starting sector of the block.
    pub sector: u32,
    /// The number of sectors in the block.
    pub count: u32,
    /// The block kind.
    pub kind: BlockKind,
    /// Any hash exceptions for the block.
    pub hash_exceptions: Box<[WIAExceptionList]>,
    /// The duration of I/O operations, if available.
    pub io_duration: Option<std::time::Duration>,
}

impl Block {
    /// Creates a new block from a block of sectors.
    #[inline]
    pub fn new(block_idx: u32, block_size: u32, kind: BlockKind) -> Self {
        let sectors_per_block = block_size / SECTOR_SIZE as u32;
        Self {
            sector: block_idx * sectors_per_block,
            count: sectors_per_block,
            kind,
            hash_exceptions: Default::default(),
            io_duration: None,
        }
    }

    /// Creates a new block from a single sector.
    #[inline]
    pub fn sector(sector: u32, kind: BlockKind) -> Self {
        Self { sector, count: 1, kind, hash_exceptions: Default::default(), io_duration: None }
    }

    /// Creates a new block from a range of sectors.
    #[inline]
    pub fn sectors(sector: u32, count: u32, kind: BlockKind) -> Self {
        Self { sector, count, kind, hash_exceptions: Default::default(), io_duration: None }
    }

    /// Returns whether the block contains the specified sector.
    #[inline]
    pub fn contains(&self, sector: u32) -> bool {
        sector >= self.sector && sector < self.sector + self.count
    }

    /// Returns an error if the block does not contain the specified sector.
    pub fn ensure_contains(&self, sector: u32) -> io::Result<()> {
        if !self.contains(sector) {
            return Err(io::Error::other(format!(
                "Sector {} not in block range {}-{}",
                sector,
                self.sector,
                self.sector + self.count
            )));
        }
        Ok(())
    }

    /// Decrypts block data in-place. The decrypted data can be accessed using
    /// [`partition_data`](Block::partition_data).
    pub(crate) fn decrypt_block(&self, data: &mut [u8], key: Option<KeyBytes>) -> io::Result<()> {
        match self.kind {
            BlockKind::None => {}
            BlockKind::Raw => {
                if let Some(key) = key {
                    for i in 0..self.count as usize {
                        decrypt_sector(array_ref_mut![data, i * SECTOR_SIZE, SECTOR_SIZE], &key);
                    }
                }
            }
            BlockKind::PartDecrypted { .. } => {
                // no-op
            }
            BlockKind::Junk => {
                // unsupported, used for DirectDiscReader
                data.fill(0);
            }
            BlockKind::Zero => data.fill(0),
        }
        Ok(())
    }

    /// Copies a sector's raw data to the output buffer. Returns whether the sector is encrypted
    /// and whether it has hashes.
    pub(crate) fn copy_sector(
        &self,
        out: &mut [u8; SECTOR_SIZE],
        data: &[u8],
        abs_sector: u32,
        disc_header: &DiscHeader,
        partition: Option<&PartitionInfo>,
    ) -> io::Result<(bool, bool)> {
        let mut encrypted = false;
        let mut has_hashes = false;
        match self.kind {
            BlockKind::None => {}
            BlockKind::Raw => {
                *out = *self.sector_buf(data, abs_sector)?;
                if partition.is_some_and(|p| p.has_encryption) {
                    encrypted = true;
                }
                if partition.is_some_and(|p| p.has_hashes) {
                    has_hashes = true;
                }
            }
            BlockKind::PartDecrypted { hash_block } => {
                if hash_block {
                    *out = *self.sector_buf(data, abs_sector)?;
                    has_hashes = partition.is_some_and(|p| p.has_hashes);
                } else {
                    *array_ref_mut![out, HASHES_SIZE, SECTOR_DATA_SIZE] =
                        *self.sector_data_buf(data, abs_sector)?;
                }
            }
            BlockKind::Junk => generate_junk_sector(out, abs_sector, partition, disc_header),
            BlockKind::Zero => out.fill(0),
        }
        Ok((encrypted, has_hashes))
    }

    /// Returns a sector's data from the block buffer.
    pub(crate) fn sector_buf<'a>(
        &self,
        data: &'a [u8],
        abs_sector: u32,
    ) -> io::Result<&'a [u8; SECTOR_SIZE]> {
        self.ensure_contains(abs_sector)?;
        let block_offset = ((abs_sector - self.sector) * SECTOR_SIZE as u32) as usize;
        Ok(array_ref!(data, block_offset, SECTOR_SIZE))
    }

    /// Returns a sector's partition data (excluding hashes) from the block buffer.
    pub(crate) fn sector_data_buf<'a>(
        &self,
        data: &'a [u8],
        abs_sector: u32,
    ) -> io::Result<&'a [u8; SECTOR_DATA_SIZE]> {
        self.ensure_contains(abs_sector)?;
        let block_offset = ((abs_sector - self.sector) * SECTOR_DATA_SIZE as u32) as usize;
        Ok(array_ref!(data, block_offset, SECTOR_DATA_SIZE))
    }

    /// Returns raw data from the block buffer, starting at the specified position.
    pub(crate) fn data<'a>(&self, data: &'a [u8], pos: u64) -> io::Result<&'a [u8]> {
        if self.kind == BlockKind::None {
            return Ok(&[]);
        }
        self.ensure_contains((pos / SECTOR_SIZE as u64) as u32)?;
        let offset = (pos - self.sector as u64 * SECTOR_SIZE as u64) as usize;
        let end = self.count as usize * SECTOR_SIZE;
        Ok(&data[offset..end])
    }

    /// Returns partition data (excluding hashes) from the block buffer, starting at the specified
    /// position within the partition.
    ///
    /// If the block does not contain hashes, this will return the full block data. Otherwise, this
    /// will return only the corresponding sector's data, ending at the sector boundary, to avoid
    /// reading into the next sector's hash block.
    pub(crate) fn partition_data<'a>(
        &self,
        data: &'a [u8],
        pos: u64,
        data_start_sector: u32,
        partition_has_hashes: bool,
    ) -> io::Result<&'a [u8]> {
        let block_has_hashes = match self.kind {
            BlockKind::Raw => partition_has_hashes,
            BlockKind::PartDecrypted { hash_block, .. } => hash_block && partition_has_hashes,
            BlockKind::Junk | BlockKind::Zero => false,
            BlockKind::None => return Ok(&[]),
        };
        let (part_sector, sector_offset) = if partition_has_hashes {
            ((pos / SECTOR_DATA_SIZE as u64) as u32, (pos % SECTOR_DATA_SIZE as u64) as usize)
        } else {
            ((pos / SECTOR_SIZE as u64) as u32, (pos % SECTOR_SIZE as u64) as usize)
        };
        let abs_sector = part_sector + data_start_sector;
        self.ensure_contains(abs_sector)?;
        let block_sector = (abs_sector - self.sector) as usize;
        if block_has_hashes {
            let offset = block_sector * SECTOR_SIZE + HASHES_SIZE + sector_offset;
            let end = (block_sector + 1) * SECTOR_SIZE; // end of sector
            Ok(&data[offset..end])
        } else if partition_has_hashes {
            let offset = block_sector * SECTOR_DATA_SIZE + sector_offset;
            let end = self.count as usize * SECTOR_DATA_SIZE; // end of block
            Ok(&data[offset..end])
        } else {
            let offset = block_sector * SECTOR_SIZE + sector_offset;
            let end = self.count as usize * SECTOR_SIZE; // end of block
            Ok(&data[offset..end])
        }
    }

    pub(crate) fn append_hash_exceptions(
        &self,
        abs_sector: u32,
        group_sector: u32,
        out: &mut Vec<WIAException>,
    ) -> io::Result<()> {
        self.ensure_contains(abs_sector)?;
        let block_sector = abs_sector - self.sector;
        let group = (block_sector / 64) as usize;
        let base_offset = ((block_sector % 64) as usize * HASHES_SIZE) as u16;
        let new_base_offset = (group_sector * HASHES_SIZE as u32) as u16;
        out.extend(self.hash_exceptions.get(group).iter().flat_map(|list| {
            list.iter().filter_map(|exception| {
                let offset = exception.offset.get();
                // The exclusive end of sector 63 is 65536, beyond u16::MAX.
                if offset >= base_offset
                    && u32::from(offset) < u32::from(base_offset) + HASHES_SIZE as u32
                {
                    let new_offset = (offset - base_offset) + new_base_offset;
                    Some(WIAException { offset: new_offset.into(), hash: exception.hash })
                } else {
                    None
                }
            })
        }));
        Ok(())
    }
}

/// The block kind.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub enum BlockKind {
    /// Empty block, likely end of disc
    #[default]
    None,
    /// Raw data or encrypted Wii partition data
    Raw,
    /// Decrypted Wii partition data
    PartDecrypted {
        /// Whether the sector has its hash block intact
        hash_block: bool,
    },
    /// Wii partition junk data
    Junk,
    /// All zeroes
    Zero,
}

/// Generates junk data for a single sector.
pub fn generate_junk_sector(
    out: &mut [u8; SECTOR_SIZE],
    abs_sector: u32,
    partition: Option<&PartitionInfo>,
    disc_header: &DiscHeader,
) {
    let (pos, offset) = if partition.is_some_and(|p| p.has_hashes) {
        let sector = abs_sector - partition.unwrap().data_start_sector;
        (sector as u64 * SECTOR_DATA_SIZE as u64, HASHES_SIZE)
    } else {
        (abs_sector as u64 * SECTOR_SIZE as u64, 0)
    };
    out[..offset].fill(0);
    let mut lfg = LaggedFibonacci::default();
    lfg.fill_sector_chunked(
        &mut out[offset..],
        *array_ref![disc_header.game_id, 0, 4],
        disc_header.disc_num,
        pos,
    );
}
