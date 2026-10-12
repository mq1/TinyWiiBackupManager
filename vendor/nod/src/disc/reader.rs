use std::{
    io,
    io::{BufRead, Seek, SeekFrom},
    sync::Arc,
};

use bytes::Bytes;
use polonius_the_crab::{polonius, polonius_return};
use tracing::warn;
use zerocopy::{FromBytes, IntoBytes};

use crate::{
    Error, Result, ResultContext,
    common::{PartitionInfo, PartitionKind},
    disc::{
        BB2_OFFSET, BOOT_SIZE, BootHeader, DL_DVD_SIZE, DiscHeader, MINI_DVD_SIZE,
        SECTOR_GROUP_SIZE, SECTOR_SIZE, SL_DVD_SIZE,
        direct::{DirectDiscReader, DirectDiscReaderMode},
        fst::{Fst, NodeKind},
        gcn::{PartitionReaderGC, read_fst},
        preloader::{
            Preloader, SectorGroup, SectorGroupLoader, SectorGroupRequest, fetch_sector_group,
        },
        wii::{
            PartitionReaderWii, REGION_OFFSET, REGION_SIZE, WII_PART_GROUP_OFF, WiiPartEntry,
            WiiPartGroup, WiiPartitionHeader,
        },
    },
    io::block::BlockReader,
    read::{DiscMeta, DiscOptions, PartitionEncryption, PartitionOptions, PartitionReader},
    util::{
        array_ref, impl_read_for_bufread,
        read::{read_arc, read_from, read_vec},
    },
};

pub struct DiscReader {
    io: Box<dyn BlockReader>,
    preloader: Arc<Preloader>,
    pos: u64,
    size: u64,
    mode: PartitionEncryption,
    raw_boot: Arc<[u8; BOOT_SIZE]>,
    alt_disc_header: Option<Arc<DiscHeader>>,
    disc_data: DiscReaderData,
    sector_group: Option<SectorGroup>,
}

#[derive(Clone)]
enum DiscReaderData {
    GameCube {
        raw_fst: Option<Arc<[u8]>>,
    },
    Wii {
        partitions: Arc<[PartitionInfo]>,
        alt_partitions: Option<Arc<[PartitionInfo]>>,
        region: [u8; REGION_SIZE],
    },
}

impl Clone for DiscReader {
    fn clone(&self) -> Self {
        Self {
            io: self.io.clone(),
            preloader: self.preloader.clone(),
            pos: 0,
            size: self.size,
            mode: self.mode,
            raw_boot: self.raw_boot.clone(),
            alt_disc_header: self.alt_disc_header.clone(),
            disc_data: self.disc_data.clone(),
            sector_group: None,
        }
    }
}

impl DiscReader {
    pub fn new(inner: Box<dyn BlockReader>, options: &DiscOptions) -> Result<Self> {
        let mut reader = DirectDiscReader::new(inner)?;

        let raw_boot: Arc<[u8; BOOT_SIZE]> =
            read_arc(reader.as_mut()).context("Reading disc headers")?;
        let disc_header = DiscHeader::ref_from_bytes(&raw_boot[..size_of::<DiscHeader>()])
            .expect("Invalid disc header alignment");
        let disc_header_arc = Arc::from(disc_header.clone());

        let mut alt_disc_header = None;
        let disc_data = if disc_header.is_wii() {
            // Sanity check
            if disc_header.has_partition_encryption() && !disc_header.has_partition_hashes() {
                return Err(Error::DiscFormat(
                    "Wii disc is encrypted but has no partition hashes".to_string(),
                ));
            }
            if !disc_header.has_partition_hashes()
                && options.partition_encryption == PartitionEncryption::ForceEncrypted
            {
                return Err(Error::Other(
                    "Unsupported: Rebuilding encryption for Wii disc without hashes".to_string(),
                ));
            }

            // Read region info
            reader.seek(SeekFrom::Start(REGION_OFFSET)).context("Seeking to region info")?;
            let region: [u8; REGION_SIZE] =
                read_from(&mut reader).context("Reading region info")?;

            // Read partition info
            let partitions = Arc::<[PartitionInfo]>::from(read_partition_info(
                &mut reader,
                disc_header_arc.clone(),
            )?);
            let mut alt_partitions = None;

            // Update disc header with encryption mode
            if matches!(
                options.partition_encryption,
                PartitionEncryption::ForceDecrypted | PartitionEncryption::ForceEncrypted
            ) {
                let mut disc_header = Box::new(disc_header.clone());
                let mut partitions = Box::<[PartitionInfo]>::from(partitions.as_ref());
                disc_header.no_partition_encryption = match options.partition_encryption {
                    PartitionEncryption::ForceDecrypted => 1,
                    PartitionEncryption::ForceEncrypted => 0,
                    _ => unreachable!(),
                };
                for partition in &mut partitions {
                    partition.has_encryption = disc_header.has_partition_encryption();
                }
                alt_disc_header = Some(Arc::from(disc_header));
                alt_partitions = Some(Arc::from(partitions));
            }

            DiscReaderData::Wii { partitions, alt_partitions, region }
        } else if disc_header.is_gamecube() {
            DiscReaderData::GameCube { raw_fst: None }
        } else {
            return Err(Error::DiscFormat("Invalid disc header".to_string()));
        };

        // Calculate disc size
        let io = reader.into_inner();
        let partitions = match &disc_data {
            DiscReaderData::Wii { partitions, .. } => partitions,
            _ => &Arc::default(),
        };
        let size = io.meta().disc_size.unwrap_or_else(|| guess_disc_size(partitions));
        let preloader = Preloader::new(
            SectorGroupLoader::new(io.clone(), disc_header_arc, partitions.clone()),
            #[cfg(feature = "threading")]
            options.preloader_threads,
        );
        Ok(Self {
            io,
            preloader,
            pos: 0,
            size,
            mode: options.partition_encryption,
            raw_boot,
            disc_data,
            sector_group: None,
            alt_disc_header,
        })
    }

    #[inline]
    pub fn reset(&mut self) { self.pos = 0; }

    #[inline]
    pub fn position(&self) -> u64 { self.pos }

    #[inline]
    pub fn disc_size(&self) -> u64 { self.size }

    #[inline]
    pub fn header(&self) -> &DiscHeader {
        self.alt_disc_header.as_deref().unwrap_or_else(|| {
            DiscHeader::ref_from_bytes(&self.raw_boot[..size_of::<DiscHeader>()])
                .expect("Invalid disc header alignment")
        })
    }

    // #[inline]
    // pub fn orig_header(&self) -> &DiscHeader {
    //     DiscHeader::ref_from_bytes(&self.raw_boot[..size_of::<DiscHeader>()])
    //         .expect("Invalid disc header alignment")
    // }

    #[inline]
    pub fn region(&self) -> Option<&[u8; REGION_SIZE]> {
        match &self.disc_data {
            DiscReaderData::Wii { region, .. } => Some(region),
            _ => None,
        }
    }

    #[inline]
    pub fn partitions(&self) -> &[PartitionInfo] {
        match &self.disc_data {
            DiscReaderData::Wii { partitions, alt_partitions, .. } => {
                alt_partitions.as_deref().unwrap_or(partitions)
            }
            _ => &[],
        }
    }

    #[inline]
    pub fn orig_partitions(&self) -> &[PartitionInfo] {
        match &self.disc_data {
            DiscReaderData::Wii { partitions, .. } => partitions,
            _ => &[],
        }
    }

    /// A reference to the disc's boot header (BB2) for GameCube discs.
    /// For Wii discs, use the boot header from the appropriate [PartitionInfo].
    #[inline]
    pub fn boot_header(&self) -> Option<&BootHeader> {
        match &self.disc_data {
            DiscReaderData::GameCube { .. } => Some(
                BootHeader::ref_from_bytes(array_ref![
                    self.raw_boot,
                    BB2_OFFSET,
                    size_of::<BootHeader>()
                ])
                .expect("Invalid boot header alignment"),
            ),
            _ => None,
        }
    }

    /// A reference to the raw FST for GameCube discs.
    /// For Wii discs, use the FST from the appropriate [PartitionInfo].
    #[inline]
    pub fn fst(&self) -> Option<Fst<'_>> {
        match &self.disc_data {
            DiscReaderData::GameCube { raw_fst } => {
                raw_fst.as_deref().and_then(|v| Fst::new(v).ok())
            }
            _ => None,
        }
    }

    #[inline]
    pub fn meta(&self) -> DiscMeta { self.io.meta() }

    /// Opens a new, decrypted partition read stream for the specified partition index.
    pub fn open_partition(
        &self,
        index: usize,
        options: &PartitionOptions,
    ) -> Result<Box<dyn PartitionReader>> {
        match &self.disc_data {
            DiscReaderData::GameCube { .. } => {
                if index == 0 {
                    Ok(PartitionReaderGC::new(self.preloader.clone(), self.disc_size())?)
                } else {
                    Err(Error::DiscFormat("GameCube discs only have one partition".to_string()))
                }
            }
            DiscReaderData::Wii { partitions, .. } => {
                if let Some(part) = partitions.get(index) {
                    Ok(PartitionReaderWii::new(self.preloader.clone(), part, options)?)
                } else {
                    Err(Error::DiscFormat(format!("Partition {index} not found")))
                }
            }
        }
    }

    /// Opens a new, decrypted partition read stream for the first partition matching
    /// the specified kind.
    pub fn open_partition_kind(
        &self,
        kind: PartitionKind,
        options: &PartitionOptions,
    ) -> Result<Box<dyn PartitionReader>> {
        match &self.disc_data {
            DiscReaderData::GameCube { .. } => {
                if kind == PartitionKind::Data {
                    Ok(PartitionReaderGC::new(self.preloader.clone(), self.disc_size())?)
                } else {
                    Err(Error::DiscFormat("GameCube discs only have a data partition".to_string()))
                }
            }
            DiscReaderData::Wii { partitions, .. } => {
                if let Some(part) = partitions.iter().find(|v| v.kind == kind) {
                    Ok(PartitionReaderWii::new(self.preloader.clone(), part, options)?)
                } else {
                    Err(Error::DiscFormat(format!("Partition type {kind} not found")))
                }
            }
        }
    }

    pub fn load_sector_group(
        &mut self,
        abs_sector: u32,
        force_rehash: bool,
    ) -> io::Result<(&SectorGroup, bool)> {
        let (request, max_groups) = if let Some(partition) =
            self.orig_partitions().iter().find(|part| part.data_contains_sector(abs_sector))
        {
            let group_idx = (abs_sector - partition.data_start_sector) / 64;
            let max_groups = (partition.data_end_sector - partition.data_start_sector).div_ceil(64);
            let request = SectorGroupRequest {
                group_idx,
                partition_idx: Some(partition.index as u8),
                mode: self.mode,
                force_rehash,
            };
            (request, max_groups)
        } else {
            let group_idx = abs_sector / 64;
            let max_groups = self.size.div_ceil(SECTOR_GROUP_SIZE as u64) as u32;
            let request = SectorGroupRequest {
                group_idx,
                partition_idx: None,
                mode: self.mode,
                force_rehash,
            };
            (request, max_groups)
        };

        // Load sector group
        let (sector_group, updated) =
            fetch_sector_group(request, max_groups, &mut self.sector_group, &self.preloader)?;

        Ok((sector_group, updated))
    }

    pub fn fill_buf_internal(&mut self) -> io::Result<Bytes> {
        let pos = self.pos;
        let size = self.size;
        if pos >= size {
            return Ok(Bytes::new());
        }

        // Read from modified disc header
        if pos < size_of::<DiscHeader>() as u64 {
            if let Some(alt_disc_header) = &self.alt_disc_header {
                return Ok(Bytes::copy_from_slice(&alt_disc_header.as_bytes()[pos as usize..]));
            }
        }

        // Load sector group
        let abs_sector = (pos / SECTOR_SIZE as u64) as u32;
        let (sector_group, _updated) = self.load_sector_group(abs_sector, false)?;

        // Calculate the number of consecutive sectors in the group
        let group_sector = abs_sector - sector_group.start_sector;
        let consecutive_sectors = sector_group.consecutive_sectors(group_sector);
        if consecutive_sectors == 0 {
            return Ok(Bytes::new());
        }
        let num_sectors = group_sector + consecutive_sectors;

        // Read from sector group buffer
        let group_start = sector_group.start_sector as u64 * SECTOR_SIZE as u64;
        let offset = (pos - group_start) as usize;
        let end = (num_sectors as u64 * SECTOR_SIZE as u64).min(size - group_start) as usize;
        Ok(sector_group.data.slice(offset..end))
    }
}

impl BufRead for DiscReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let pos = self.pos;
        let size = self.size;
        if pos >= size {
            return Ok(&[]);
        }

        let mut this = self;
        polonius!(|this| -> io::Result<&'polonius [u8]> {
            // Read from modified disc header
            if pos < size_of::<DiscHeader>() as u64 {
                if let Some(alt_disc_header) = &this.alt_disc_header {
                    polonius_return!(Ok(&alt_disc_header.as_bytes()[pos as usize..]));
                }
            }
        });

        // Load sector group
        let abs_sector = (pos / SECTOR_SIZE as u64) as u32;
        let (sector_group, _updated) = this.load_sector_group(abs_sector, false)?;

        // Calculate the number of consecutive sectors in the group
        let group_sector = abs_sector - sector_group.start_sector;
        let consecutive_sectors = sector_group.consecutive_sectors(group_sector);
        if consecutive_sectors == 0 {
            return Ok(&[]);
        }
        let num_sectors = group_sector + consecutive_sectors;

        // Read from sector group buffer
        let group_start = sector_group.start_sector as u64 * SECTOR_SIZE as u64;
        let offset = (pos - group_start) as usize;
        let end = (num_sectors as u64 * SECTOR_SIZE as u64).min(size - group_start) as usize;
        Ok(&sector_group.data[offset..end])
    }

    #[inline]
    fn consume(&mut self, amt: usize) { self.pos += amt as u64; }
}

impl_read_for_bufread!(DiscReader);

impl Seek for DiscReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(v) => v,
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "BlockIOReader: SeekFrom::End is not supported".to_string(),
                ));
            }
            SeekFrom::Current(v) => self.pos.saturating_add_signed(v),
        };
        Ok(self.pos)
    }
}

fn read_partition_info(
    reader: &mut DirectDiscReader,
    disc_header: Arc<DiscHeader>,
) -> Result<Vec<PartitionInfo>> {
    reader.seek(SeekFrom::Start(WII_PART_GROUP_OFF)).context("Seeking to partition groups")?;
    let part_groups: [WiiPartGroup; 4] = read_from(reader).context("Reading partition groups")?;
    let mut part_info = Vec::new();
    for (group_idx, group) in part_groups.iter().enumerate() {
        let part_count = group.part_count.get();
        if part_count == 0 {
            continue;
        }
        reader
            .seek(SeekFrom::Start(group.part_entry_off()))
            .with_context(|| format!("Seeking to partition group {group_idx}"))?;
        let entries: Vec<WiiPartEntry> = read_vec(reader, part_count as usize)
            .with_context(|| format!("Reading partition group {group_idx}"))?;
        for (part_idx, entry) in entries.iter().enumerate() {
            let offset = entry.offset();
            reader
                .seek(SeekFrom::Start(offset))
                .with_context(|| format!("Seeking to partition data {group_idx}:{part_idx}"))?;
            let header: Arc<WiiPartitionHeader> = read_arc(reader)
                .with_context(|| format!("Reading partition header {group_idx}:{part_idx}"))?;

            let key = header.ticket.decrypt_title_key()?;
            let start_offset = entry.offset();
            if start_offset % SECTOR_SIZE as u64 != 0 {
                return Err(Error::DiscFormat(format!(
                    "Partition {group_idx}:{part_idx} offset is not sector aligned",
                )));
            }

            let data_start_offset = entry.offset() + header.data_off();
            let data_size = header.data_size();
            let data_end_offset = data_start_offset + data_size;
            if data_start_offset % SECTOR_SIZE as u64 != 0
                || data_end_offset % SECTOR_SIZE as u64 != 0
            {
                return Err(Error::DiscFormat(format!(
                    "Partition {group_idx}:{part_idx} data is not sector aligned",
                )));
            }
            let start_sector = (start_offset / SECTOR_SIZE as u64) as u32;
            let data_start_sector = (data_start_offset / SECTOR_SIZE as u64) as u32;
            let mut data_end_sector = (data_end_offset / SECTOR_SIZE as u64) as u32;

            reader.reset(DirectDiscReaderMode::Partition {
                disc_header: disc_header.clone(),
                data_start_sector,
                key,
            });
            let raw_boot: Arc<[u8; BOOT_SIZE]> = read_arc(reader).context("Reading boot data")?;
            let partition_disc_header =
                DiscHeader::ref_from_bytes(array_ref![raw_boot, 0, size_of::<DiscHeader>()])
                    .expect("Invalid disc header alignment");
            let boot_header = BootHeader::ref_from_bytes(&raw_boot[BB2_OFFSET..])
                .expect("Invalid boot header alignment");
            let raw_fst = if partition_disc_header.is_wii() {
                let raw_fst = read_fst(reader, boot_header, true)?;
                match Fst::new(&raw_fst) {
                    Ok(fst) => {
                        let max_fst_offset = fst
                            .nodes
                            .iter()
                            .filter_map(|n| match n.kind() {
                                NodeKind::File => Some(n.offset(true) + n.length() as u64),
                                _ => None,
                            })
                            .max()
                            .unwrap_or(0);
                        if max_fst_offset > data_size {
                            if data_size == 0 {
                                // Guess data size for decrypted partitions
                                data_end_sector =
                                    max_fst_offset.div_ceil(SECTOR_SIZE as u64) as u32;
                            } else {
                                return Err(Error::DiscFormat(format!(
                                    "Partition {group_idx}:{part_idx} FST exceeds data size",
                                )));
                            }
                        }
                        Some(raw_fst)
                    }
                    Err(e) => {
                        warn!("Partition {group_idx}:{part_idx} FST is not valid: {e}");
                        None
                    }
                }
            } else {
                warn!("Partition {group_idx}:{part_idx} is not valid");
                None
            };
            reader.reset(DirectDiscReaderMode::Raw);

            part_info.push(PartitionInfo {
                index: part_info.len(),
                kind: entry.kind.get().into(),
                start_sector,
                data_start_sector,
                data_end_sector,
                key,
                header,
                has_encryption: disc_header.has_partition_encryption(),
                has_hashes: disc_header.has_partition_hashes(),
                raw_boot,
                raw_fst,
            });
        }
    }
    Ok(part_info)
}

fn guess_disc_size(part_info: &[PartitionInfo]) -> u64 {
    let max_offset = part_info
        .iter()
        .map(|v| v.data_end_sector as u64 * SECTOR_SIZE as u64)
        .max()
        .unwrap_or(0x50000);
    if max_offset <= MINI_DVD_SIZE && !part_info.iter().any(|v| v.kind == PartitionKind::Data) {
        // Datel disc
        MINI_DVD_SIZE
    } else if max_offset < SL_DVD_SIZE {
        SL_DVD_SIZE
    } else {
        DL_DVD_SIZE
    }
}
