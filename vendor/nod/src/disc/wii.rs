//! Wii disc types.

use std::{
    ffi::CStr,
    io,
    io::{BufRead, Seek, SeekFrom},
    mem::size_of,
    sync::Arc,
};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, big_endian::*};

use crate::{
    Error, Result, ResultContext,
    common::{HashBytes, KeyBytes, PartitionInfo},
    disc::{
        SECTOR_GROUP_SIZE, SECTOR_SIZE,
        gcn::{PartitionReaderGC, read_part_meta},
        preloader::{Preloader, SectorGroup, SectorGroupRequest, fetch_sector_group},
    },
    read::{PartitionEncryption, PartitionMeta, PartitionOptions, PartitionReader},
    util::{
        aes::aes_cbc_decrypt,
        array_ref,
        digest::sha1_hash,
        div_rem, impl_read_for_bufread,
        read::{read_arc, read_arc_slice},
        static_assert,
    },
};

/// Size in bytes of the hashes block in a Wii disc sector
pub const HASHES_SIZE: usize = 0x400;

/// Size in bytes of the data block in a Wii disc sector (excluding hashes)
pub const SECTOR_DATA_SIZE: usize = SECTOR_SIZE - HASHES_SIZE; // 0x7C00

/// Size in bytes of the disc region info (region.bin)
pub const REGION_SIZE: usize = 0x20;

/// Size in bytes of the H3 table (h3.bin)
pub const H3_TABLE_SIZE: usize = 0x18000;

/// Offset of the disc region info
pub const REGION_OFFSET: u64 = 0x4E000;

// ppki (Retail)
pub(crate) const RVL_CERT_ISSUER_PPKI_TICKET: &str = "Root-CA00000001-XS00000003";
#[rustfmt::skip]
pub(crate) static RETAIL_COMMON_KEYS: [KeyBytes; 3] = [
    /* RVL_KEY_RETAIL */
    [0xeb, 0xe4, 0x2a, 0x22, 0x5e, 0x85, 0x93, 0xe4, 0x48, 0xd9, 0xc5, 0x45, 0x73, 0x81, 0xaa, 0xf7],
    /* RVL_KEY_KOREAN */
    [0x63, 0xb8, 0x2b, 0xb4, 0xf4, 0x61, 0x4e, 0x2e, 0x13, 0xf2, 0xfe, 0xfb, 0xba, 0x4c, 0x9b, 0x7e],
    /* vWii_KEY_RETAIL */
    [0x30, 0xbf, 0xc7, 0x6e, 0x7c, 0x19, 0xaf, 0xbb, 0x23, 0x16, 0x33, 0x30, 0xce, 0xd7, 0xc2, 0x8d],
];

// dpki (Debug)
pub(crate) const RVL_CERT_ISSUER_DPKI_TICKET: &str = "Root-CA00000002-XS00000006";
#[rustfmt::skip]
pub(crate) static DEBUG_COMMON_KEYS: [KeyBytes; 3] = [
    /* RVL_KEY_DEBUG */
    [0xa1, 0x60, 0x4a, 0x6a, 0x71, 0x23, 0xb5, 0x29, 0xae, 0x8b, 0xec, 0x32, 0xc8, 0x16, 0xfc, 0xaa],
    /* RVL_KEY_KOREAN_DEBUG */
    [0x67, 0x45, 0x8b, 0x6b, 0xc6, 0x23, 0x7b, 0x32, 0x69, 0x98, 0x3c, 0x64, 0x73, 0x48, 0x33, 0x66],
    /* vWii_KEY_DEBUG */
    [0x2f, 0x5c, 0x1b, 0x29, 0x44, 0xe7, 0xfd, 0x6f, 0xc3, 0x97, 0x96, 0x4b, 0x05, 0x76, 0x91, 0xfa],
];

#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub(crate) struct WiiPartEntry {
    pub(crate) offset: U32,
    pub(crate) kind: U32,
}

static_assert!(size_of::<WiiPartEntry>() == 8);

impl WiiPartEntry {
    pub(crate) fn offset(&self) -> u64 { (self.offset.get() as u64) << 2 }
}

pub(crate) const WII_PART_GROUP_OFF: u64 = 0x40000;

#[derive(Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub(crate) struct WiiPartGroup {
    pub(crate) part_count: U32,
    pub(crate) part_entry_off: U32,
}

static_assert!(size_of::<WiiPartGroup>() == 8);

impl WiiPartGroup {
    pub(crate) fn part_entry_off(&self) -> u64 { (self.part_entry_off.get() as u64) << 2 }
}

/// Signed blob header
#[derive(Debug, Clone, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct SignedHeader {
    /// Signature type, always 0x00010001 (RSA-2048)
    pub sig_type: U32,
    /// RSA-2048 signature
    pub sig: [u8; 256],
    _pad: [u8; 60],
}

static_assert!(size_of::<SignedHeader>() == 0x140);

/// Ticket limit
#[derive(Debug, Clone, PartialEq, Default, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct TicketLimit {
    /// Limit type
    pub limit_type: U32,
    /// Maximum value for the limit
    pub max_value: U32,
}

static_assert!(size_of::<TicketLimit>() == 8);

/// Wii ticket
#[derive(Debug, Clone, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct Ticket {
    /// Signed blob header
    pub header: SignedHeader,
    /// Signature issuer
    pub sig_issuer: [u8; 64],
    /// ECDH data
    pub ecdh: [u8; 60],
    /// Ticket format version
    pub version: u8,
    _pad1: U16,
    /// Title key (encrypted)
    pub title_key: KeyBytes,
    _pad2: u8,
    /// Ticket ID
    pub ticket_id: [u8; 8],
    /// Console ID
    pub console_id: [u8; 4],
    /// Title ID
    pub title_id: [u8; 8],
    _pad3: U16,
    /// Ticket title version
    pub ticket_title_version: U16,
    /// Permitted titles mask
    pub permitted_titles_mask: U32,
    /// Permit mask
    pub permit_mask: U32,
    /// Title export allowed
    pub title_export_allowed: u8,
    /// Common key index
    pub common_key_idx: u8,
    _pad4: [u8; 48],
    /// Content access permissions
    pub content_access_permissions: [u8; 64],
    _pad5: [u8; 2],
    /// Ticket limits
    pub limits: [TicketLimit; 8],
}

static_assert!(size_of::<Ticket>() == 0x2A4);

impl Ticket {
    /// Decrypts the ticket title key using the appropriate common key
    pub fn decrypt_title_key(&self) -> Result<KeyBytes> {
        let mut iv: KeyBytes = [0; 16];
        iv[..8].copy_from_slice(&self.title_id);
        let cert_issuer_ticket =
            CStr::from_bytes_until_nul(&self.sig_issuer).ok().and_then(|c| c.to_str().ok());
        let common_keys = match cert_issuer_ticket {
            Some(RVL_CERT_ISSUER_PPKI_TICKET) => &RETAIL_COMMON_KEYS,
            Some(RVL_CERT_ISSUER_DPKI_TICKET) => &DEBUG_COMMON_KEYS,
            Some(v) => {
                return Err(Error::DiscFormat(format!("unknown certificate issuer {:?}", v)));
            }
            None => {
                return Err(Error::DiscFormat("failed to parse certificate issuer".to_string()));
            }
        };
        let common_key = common_keys.get(self.common_key_idx as usize).ok_or(Error::DiscFormat(
            format!("unknown common key index {}", self.common_key_idx),
        ))?;
        let mut title_key = self.title_key;
        aes_cbc_decrypt(common_key, &iv, &mut title_key);
        Ok(title_key)
    }
}

/// Title metadata header
#[derive(Debug, Clone, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct TmdHeader {
    /// Signed blob header
    pub header: SignedHeader,
    /// Signature issuer
    pub sig_issuer: [u8; 64],
    /// Version
    pub version: u8,
    /// CA CRL version
    pub ca_crl_version: u8,
    /// Signer CRL version
    pub signer_crl_version: u8,
    /// Is vWii title
    pub is_vwii: u8,
    /// IOS ID
    pub ios_id: [u8; 8],
    /// Title ID
    pub title_id: [u8; 8],
    /// Title type
    pub title_type: u32,
    /// Group ID
    pub group_id: U16,
    _pad1: [u8; 2],
    /// Region
    pub region: U16,
    /// Ratings
    pub ratings: KeyBytes,
    _pad2: [u8; 12],
    /// IPC mask
    pub ipc_mask: [u8; 12],
    _pad3: [u8; 18],
    /// Access flags
    pub access_flags: U32,
    /// Title version
    pub title_version: U16,
    /// Number of contents
    pub num_contents: U16,
    /// Boot index
    pub boot_idx: U16,
    /// Minor version (unused)
    pub minor_version: U16,
}

static_assert!(size_of::<TmdHeader>() == 0x1E4);

/// TMD content metadata
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct ContentMetadata {
    /// Content ID
    pub content_id: U32,
    /// Content index
    pub content_index: U16,
    /// Content type
    pub content_type: U16,
    /// Content size
    pub size: U64,
    /// Content hash
    pub hash: HashBytes,
}

static_assert!(size_of::<ContentMetadata>() == 0x24);

/// Wii partition header.
#[derive(Debug, Clone, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WiiPartitionHeader {
    /// Ticket
    pub ticket: Ticket,
    tmd_size: U32,
    tmd_off: U32,
    cert_chain_size: U32,
    cert_chain_off: U32,
    h3_table_off: U32,
    data_off: U32,
    data_size: U32,
}

static_assert!(size_of::<WiiPartitionHeader>() == 0x2C0);

impl WiiPartitionHeader {
    /// TMD size in bytes
    pub fn tmd_size(&self) -> u64 { self.tmd_size.get() as u64 }

    /// TMD offset in bytes (relative to the partition start)
    pub fn tmd_off(&self) -> u64 { (self.tmd_off.get() as u64) << 2 }

    /// Certificate chain size in bytes
    pub fn cert_chain_size(&self) -> u64 { self.cert_chain_size.get() as u64 }

    /// Certificate chain offset in bytes (relative to the partition start)
    pub fn cert_chain_off(&self) -> u64 { (self.cert_chain_off.get() as u64) << 2 }

    /// H3 table offset in bytes (relative to the partition start)
    pub fn h3_table_off(&self) -> u64 { (self.h3_table_off.get() as u64) << 2 }

    /// H3 table size in bytes (always H3_TABLE_SIZE)
    pub fn h3_table_size(&self) -> u64 { H3_TABLE_SIZE as u64 }

    /// Data offset in bytes (relative to the partition start)
    pub fn data_off(&self) -> u64 { (self.data_off.get() as u64) << 2 }

    /// Data size in bytes
    pub fn data_size(&self) -> u64 { (self.data_size.get() as u64) << 2 }
}

pub(crate) struct PartitionReaderWii {
    preloader: Arc<Preloader>,
    partition: PartitionInfo,
    pos: u64,
    options: PartitionOptions,
    sector_group: Option<SectorGroup>,
    meta: Option<PartitionMeta>,
}

impl Clone for PartitionReaderWii {
    fn clone(&self) -> Self {
        Self {
            preloader: self.preloader.clone(),
            partition: self.partition.clone(),
            pos: 0,
            options: self.options.clone(),
            sector_group: None,
            meta: self.meta.clone(),
        }
    }
}

impl PartitionReaderWii {
    pub fn new(
        preloader: Arc<Preloader>,
        partition: &PartitionInfo,
        options: &PartitionOptions,
    ) -> Result<Box<Self>> {
        let mut reader = Self {
            preloader,
            partition: partition.clone(),
            pos: 0,
            options: options.clone(),
            sector_group: None,
            meta: None,
        };
        if options.validate_hashes {
            // Ensure we cache the H3 table
            reader.meta()?;
        }
        Ok(Box::new(reader))
    }

    #[inline]
    pub fn len(&self) -> u64 { self.partition.data_size() }
}

impl BufRead for PartitionReaderWii {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let (part_sector, sector_offset) = if self.partition.has_hashes {
            (
                (self.pos / SECTOR_DATA_SIZE as u64) as u32,
                (self.pos % SECTOR_DATA_SIZE as u64) as usize,
            )
        } else {
            ((self.pos / SECTOR_SIZE as u64) as u32, (self.pos % SECTOR_SIZE as u64) as usize)
        };
        let abs_sector = self.partition.data_start_sector + part_sector;
        if abs_sector >= self.partition.data_end_sector {
            return Ok(&[]);
        }

        let group_idx = part_sector / 64;
        let group_sector = part_sector % 64;

        let max_groups =
            (self.partition.data_end_sector - self.partition.data_start_sector).div_ceil(64);
        let request = SectorGroupRequest {
            group_idx,
            partition_idx: Some(self.partition.index as u8),
            mode: if self.options.validate_hashes {
                PartitionEncryption::ForceDecrypted
            } else {
                PartitionEncryption::ForceDecryptedNoHashes
            },
            force_rehash: false,
        };

        // Load sector group
        let (sector_group, updated) =
            fetch_sector_group(request, max_groups, &mut self.sector_group, &self.preloader)?;
        if updated && self.options.validate_hashes {
            if let Some(h3_table) = self.meta.as_ref().and_then(|m| m.raw_h3_table.as_deref()) {
                verify_hashes(
                    array_ref![sector_group.data, 0, SECTOR_GROUP_SIZE],
                    group_idx,
                    h3_table,
                )?;
            }
        }

        // Read from sector group buffer
        let consecutive_sectors = sector_group.consecutive_sectors(group_sector);
        if consecutive_sectors == 0 {
            return Ok(&[]);
        }
        let group_sector_offset = group_sector as usize * SECTOR_SIZE;
        if self.partition.has_hashes {
            // Read until end of sector (avoid the next hash block)
            let offset = group_sector_offset + HASHES_SIZE + sector_offset;
            let end = group_sector_offset + SECTOR_SIZE;
            Ok(&sector_group.data[offset..end])
        } else {
            // Read until end of sector group (no hashes)
            let offset = group_sector_offset + sector_offset;
            let end = (group_sector + consecutive_sectors) as usize * SECTOR_SIZE;
            Ok(&sector_group.data[offset..end])
        }
    }

    #[inline]
    fn consume(&mut self, amt: usize) { self.pos += amt as u64; }
}

impl_read_for_bufread!(PartitionReaderWii);

impl Seek for PartitionReaderWii {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = match pos {
            SeekFrom::Start(v) => v,
            SeekFrom::End(v) => self.len().saturating_add_signed(v),
            SeekFrom::Current(v) => self.pos.saturating_add_signed(v),
        };
        Ok(self.pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> { Ok(self.pos) }
}

fn verify_hashes(buf: &[u8; SECTOR_GROUP_SIZE], group_idx: u32, h3_table: &[u8]) -> io::Result<()> {
    for sector in 0..64 {
        let buf = array_ref![buf, sector * SECTOR_SIZE, SECTOR_SIZE];
        let part_sector = group_idx * 64 + sector as u32;
        let (cluster, sector) = div_rem(part_sector as usize, 8);
        let (group, sub_group) = div_rem(cluster, 8);

        // H0 hashes
        for i in 0..31 {
            let expected = array_ref![buf, i * 20, 20];
            let output = sha1_hash(array_ref![buf, (i + 1) * 0x400, 0x400]);
            if output != *expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid H0 hash! (block {i})"),
                ));
            }
        }

        // H1 hash
        {
            let expected = array_ref![buf, 0x280 + sector * 20, 20];
            let output = sha1_hash(array_ref![buf, 0, 0x26C]);
            if output != *expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid H1 hash! (sector {sector})",),
                ));
            }
        }

        // H2 hash
        {
            let expected = array_ref![buf, 0x340 + sub_group * 20, 20];
            let output = sha1_hash(array_ref![buf, 0x280, 0xA0]);
            if output != *expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid H2 hash! (subgroup {sub_group})"),
                ));
            }
        }

        // H3 hash
        {
            let expected = array_ref![h3_table, group * 20, 20];
            let output = sha1_hash(array_ref![buf, 0x340, 0xA0]);
            if output != *expected {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Invalid H3 hash! (group {group})"),
                ));
            }
        }
    }

    Ok(())
}

impl PartitionReader for PartitionReaderWii {
    fn is_wii(&self) -> bool { true }

    fn meta(&mut self) -> Result<PartitionMeta> {
        if let Some(meta) = &self.meta {
            return Ok(meta.clone());
        }
        self.rewind().context("Seeking to partition header")?;
        let mut meta = read_part_meta(self, true)?;
        meta.raw_ticket = Some(Arc::from(self.partition.header.ticket.as_bytes()));

        // Read TMD, cert chain, and H3 table
        let mut reader = PartitionReaderGC::new(self.preloader.clone(), u64::MAX)?;
        let offset = self.partition.start_sector as u64 * SECTOR_SIZE as u64;
        meta.raw_tmd = if self.partition.header.tmd_size() != 0 {
            reader
                .seek(SeekFrom::Start(offset + self.partition.header.tmd_off()))
                .context("Seeking to TMD offset")?;
            Some(
                read_arc_slice::<u8, _>(&mut reader, self.partition.header.tmd_size() as usize)
                    .context("Reading TMD")?,
            )
        } else {
            None
        };
        meta.raw_cert_chain = if self.partition.header.cert_chain_size() != 0 {
            reader
                .seek(SeekFrom::Start(offset + self.partition.header.cert_chain_off()))
                .context("Seeking to cert chain offset")?;
            Some(
                read_arc_slice::<u8, _>(
                    &mut reader,
                    self.partition.header.cert_chain_size() as usize,
                )
                .context("Reading cert chain")?,
            )
        } else {
            None
        };
        meta.raw_h3_table = if self.partition.has_hashes {
            reader
                .seek(SeekFrom::Start(offset + self.partition.header.h3_table_off()))
                .context("Seeking to H3 table offset")?;

            Some(read_arc::<[u8; H3_TABLE_SIZE], _>(&mut reader).context("Reading H3 table")?)
        } else {
            None
        };

        self.meta = Some(meta.clone());
        Ok(meta)
    }
}
