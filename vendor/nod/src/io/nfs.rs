use std::{
    fs::File,
    io,
    io::{BufReader, Read},
    mem::size_of,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, big_endian::U32};

use crate::{
    Error, Result, ResultContext,
    common::{Format, KeyBytes, MagicBytes},
    disc::SECTOR_SIZE,
    io::{
        block::{Block, BlockKind, BlockReader, NFS_MAGIC},
        split::SplitFileReader,
    },
    read::{DiscMeta, DiscStream},
    util::{aes::aes_cbc_decrypt, array_ref_mut, read::read_arc, static_assert},
};

pub const NFS_END_MAGIC: MagicBytes = *b"SGGE";

#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct LBARange {
    start_sector: U32,
    num_sectors: U32,
}

#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
struct NFSHeader {
    magic: MagicBytes,
    version: U32,
    unk1: U32,
    unk2: U32,
    num_lba_ranges: U32,
    lba_ranges: [LBARange; 61],
    end_magic: MagicBytes,
}

static_assert!(size_of::<NFSHeader>() == 0x200);

impl NFSHeader {
    fn validate(&self) -> Result<()> {
        if self.magic != NFS_MAGIC {
            return Err(Error::DiscFormat("Invalid NFS magic".to_string()));
        }
        if self.num_lba_ranges.get() > 61 {
            return Err(Error::DiscFormat("Invalid NFS LBA range count".to_string()));
        }
        if self.end_magic != NFS_END_MAGIC {
            return Err(Error::DiscFormat("Invalid NFS end magic".to_string()));
        }
        Ok(())
    }

    fn lba_ranges(&self) -> &[LBARange] { &self.lba_ranges[..self.num_lba_ranges.get() as usize] }

    fn calculate_num_files(&self) -> u32 {
        let sector_count =
            self.lba_ranges().iter().fold(0u32, |acc, range| acc + range.num_sectors.get());
        (((sector_count as u64) * (SECTOR_SIZE as u64)
            + (size_of::<NFSHeader>() as u64 + 0xF9FFFFFu64))
            / 0xFA00000u64) as u32
    }

    fn phys_sector(&self, sector: u32) -> u32 {
        let mut cur_sector = 0u32;
        for range in self.lba_ranges().iter() {
            if sector >= range.start_sector.get()
                && sector - range.start_sector.get() < range.num_sectors.get()
            {
                return cur_sector + (sector - range.start_sector.get());
            }
            cur_sector += range.num_sectors.get();
        }
        u32::MAX
    }
}

#[derive(Clone)]
pub struct BlockReaderNFS {
    inner: SplitFileReader,
    header: Arc<NFSHeader>,
    raw_size: u64,
    disc_size: u64,
    key: KeyBytes,
}

impl BlockReaderNFS {
    pub fn new(directory: &Path) -> Result<Box<Self>> {
        let mut disc_io = Box::new(Self {
            inner: SplitFileReader::empty(),
            header: Arc::new(NFSHeader::new_zeroed()),
            raw_size: 0,
            disc_size: 0,
            key: [0; 16],
        });
        disc_io.load_files(directory)?;
        Ok(disc_io)
    }
}

impl BlockReader for BlockReaderNFS {
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block> {
        // Calculate physical sector
        let phys_sector = self.header.phys_sector(sector);
        if phys_sector == u32::MAX {
            // Logical zero sector
            return Ok(Block::sector(sector, BlockKind::Raw));
        }

        // Read sector
        let offset = size_of::<NFSHeader>() as u64 + phys_sector as u64 * SECTOR_SIZE as u64;
        self.inner.read_exact_at(out, offset)?;

        // Decrypt
        let mut iv = [0u8; 0x10];
        *array_ref_mut!(iv, 12, 4) = sector.to_be_bytes();
        aes_cbc_decrypt(&self.key, &iv, out);

        Ok(Block::sector(sector, BlockKind::PartDecrypted { hash_block: true }))
    }

    fn block_size(&self) -> u32 { SECTOR_SIZE as u32 }

    fn meta(&self) -> DiscMeta {
        DiscMeta { format: Format::Nfs, decrypted: true, ..Default::default() }
    }
}

fn get_path<P>(directory: &Path, path: P) -> PathBuf
where P: AsRef<Path> {
    let mut buf = directory.to_path_buf();
    for component in path.as_ref().components() {
        match component {
            Component::ParentDir => {
                buf.pop();
            }
            _ => buf.push(component),
        }
    }
    buf
}

fn get_nfs(directory: &Path, num: u32) -> Result<PathBuf> {
    let path = get_path(directory, format!("hif_{:06}.nfs", num));
    if path.exists() {
        Ok(path)
    } else {
        Err(Error::DiscFormat(format!("Failed to locate {}", path.display())))
    }
}

impl BlockReaderNFS {
    pub fn load_files(&mut self, directory: &Path) -> Result<()> {
        {
            // Load key file
            let primary_key_path =
                get_path(directory, ["..", "code", "htk.bin"].iter().collect::<PathBuf>());
            let secondary_key_path = get_path(directory, "htk.bin");
            let mut key_path = primary_key_path.canonicalize();
            if key_path.is_err() {
                key_path = secondary_key_path.canonicalize();
            }
            if key_path.is_err() {
                return Err(Error::DiscFormat(format!(
                    "Failed to locate {} or {}",
                    primary_key_path.display(),
                    secondary_key_path.display()
                )));
            }
            let resolved_path = key_path.unwrap();
            File::open(resolved_path.as_path())
                .map_err(|v| Error::Io(format!("Failed to open {}", resolved_path.display()), v))?
                .read_exact(&mut self.key)
                .map_err(|v| Error::Io(format!("Failed to read {}", resolved_path.display()), v))?;
        }

        {
            // Load header from first file
            let path = get_nfs(directory, 0)?;
            self.inner.add(&path)?;

            let mut file = BufReader::new(
                File::open(&path).with_context(|| format!("Opening file {}", path.display()))?,
            );
            let header: Arc<NFSHeader> = read_arc(&mut file)
                .with_context(|| format!("Reading NFS header from file {}", path.display()))?;
            header.validate()?;
            // log::debug!("{:?}", header);

            // Ensure remaining files exist
            for i in 1..header.calculate_num_files() {
                self.inner.add(&get_nfs(directory, i)?)?;
            }

            // Calculate sizes
            let num_sectors =
                header.lba_ranges().iter().map(|range| range.num_sectors.get()).sum::<u32>();
            let max_sector = header
                .lba_ranges()
                .iter()
                .map(|range| range.start_sector.get() + range.num_sectors.get())
                .max()
                .unwrap();
            let raw_size = size_of::<NFSHeader>() + (num_sectors as usize * SECTOR_SIZE);
            let data_size = max_sector as usize * SECTOR_SIZE;
            if raw_size > self.inner.len() as usize {
                return Err(Error::DiscFormat(format!(
                    "NFS raw size mismatch: expected at least {}, got {}",
                    raw_size,
                    self.inner.len()
                )));
            }

            self.header = header;
            self.raw_size = raw_size as u64;
            self.disc_size = data_size as u64;
        }
        Ok(())
    }
}
