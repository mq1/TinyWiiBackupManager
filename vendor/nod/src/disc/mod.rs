//! GameCube/Wii disc format types.

use std::{ffi::CStr, str::from_utf8};

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, big_endian::*};

use crate::{common::MagicBytes, util::static_assert};

pub(crate) mod direct;
pub mod fst;
pub(crate) mod gcn;
pub(crate) mod hashes;
pub(crate) mod preloader;
pub(crate) mod reader;
pub mod wii;
pub(crate) mod writer;

/// Size in bytes of a disc sector. (32 KiB)
pub const SECTOR_SIZE: usize = 0x8000;

/// Size in bytes of a Wii partition sector group. (32 KiB * 64, 2 MiB)
pub const SECTOR_GROUP_SIZE: usize = SECTOR_SIZE * 64;

/// Magic bytes for Wii discs. Located at offset 0x18.
pub const WII_MAGIC: MagicBytes = [0x5D, 0x1C, 0x9E, 0xA3];

/// Magic bytes for GameCube discs. Located at offset 0x1C.
pub const GCN_MAGIC: MagicBytes = [0xC2, 0x33, 0x9F, 0x3D];

/// Offset in bytes of the boot block within a disc partition.
pub const BB2_OFFSET: usize = 0x420;

/// Size in bytes of the disc header, debug block and boot block. (boot.bin)
pub const BOOT_SIZE: usize = 0x440;

/// Size in bytes of the DVD Boot Info (debug and region information, bi2.bin)
pub const BI2_SIZE: usize = 0x2000;

/// The size of a single-layer MiniDVD. (1.4 GB)
///
/// GameCube games and some third-party Wii discs (Datel) use this format.
pub const MINI_DVD_SIZE: u64 = 1_459_978_240;

/// The size of a single-layer DVD. (4.7 GB)
///
/// The vast majority of Wii games use this format.
pub const SL_DVD_SIZE: u64 = 4_699_979_776;

/// The size of a dual-layer DVD. (8.5 GB)
///
/// A few larger Wii games use this format.
/// (Super Smash Bros. Brawl, Metroid Prime Trilogy, etc.)
pub const DL_DVD_SIZE: u64 = 8_511_160_320;

/// Shared GameCube & Wii disc header.
///
/// This header is always at the start of the disc image and within each Wii partition.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct DiscHeader {
    /// Game ID (e.g. GM8E01 for Metroid Prime)
    pub game_id: [u8; 6],
    /// Used in multi-disc games
    pub disc_num: u8,
    /// Disc version
    pub disc_version: u8,
    /// Audio streaming enabled
    pub audio_streaming: u8,
    /// Audio streaming buffer size
    pub audio_stream_buf_size: u8,
    /// Padding
    _pad1: [u8; 14],
    /// If this is a Wii disc, this will be 0x5D1C9EA3
    pub wii_magic: MagicBytes,
    /// If this is a GameCube disc, this will be 0xC2339F3D
    pub gcn_magic: MagicBytes,
    /// Game title
    pub game_title: [u8; 64],
    /// If 1, disc omits partition hashes
    pub no_partition_hashes: u8,
    /// If 1, disc omits partition encryption
    pub no_partition_encryption: u8,
    /// Padding
    _pad2: [u8; 926],
}

static_assert!(size_of::<DiscHeader>() == 0x400);

impl DiscHeader {
    /// Game ID as a string.
    #[inline]
    pub fn game_id_str(&self) -> &str { from_utf8(&self.game_id).unwrap_or("[invalid]") }

    /// Game title as a string.
    #[inline]
    pub fn game_title_str(&self) -> &str {
        CStr::from_bytes_until_nul(&self.game_title)
            .ok()
            .and_then(|c| c.to_str().ok())
            .unwrap_or("[invalid]")
    }

    /// Whether this is a GameCube disc.
    #[inline]
    pub fn is_gamecube(&self) -> bool { self.gcn_magic == GCN_MAGIC }

    /// Whether this is a Wii disc.
    #[inline]
    pub fn is_wii(&self) -> bool { self.wii_magic == WII_MAGIC }

    /// Whether the disc has partition data hashes.
    #[inline]
    pub fn has_partition_hashes(&self) -> bool { self.no_partition_hashes == 0 }

    /// Whether the disc has partition data encryption.
    #[inline]
    pub fn has_partition_encryption(&self) -> bool { self.no_partition_encryption == 0 }
}

/// The debug block of a disc partition.
///
/// Located at offset 0x400 (following the disc header) within each partition.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct DebugHeader {
    /// Debug monitor offset
    pub debug_mon_offset: U32,
    /// Debug monitor load address
    pub debug_load_address: U32,
    /// Padding
    _pad1: [u8; 0x18],
}

static_assert!(size_of::<DebugHeader>() == 0x20);

/// The boot block (BB2) of a disc partition.
///
/// Located at offset 0x420 (following the debug block) within each partition.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct BootHeader {
    /// Offset to main DOL (Wii: >> 2)
    pub dol_offset: U32,
    /// Offset to file system table (Wii: >> 2)
    pub fst_offset: U32,
    /// File system size (Wii: >> 2)
    pub fst_size: U32,
    /// File system max size (Wii: >> 2)
    pub fst_max_size: U32,
    /// File system table load address
    pub fst_memory_address: U32,
    /// User data offset
    pub user_offset: U32,
    /// User data size
    pub user_size: U32,
    /// Padding
    _pad2: [u8; 4],
}

static_assert!(size_of::<BootHeader>() == 0x20);
static_assert!(
    size_of::<DiscHeader>() + size_of::<DebugHeader>() + size_of::<BootHeader>() == BOOT_SIZE
);

impl BootHeader {
    /// Offset within the partition to the main DOL.
    #[inline]
    pub fn dol_offset(&self, is_wii: bool) -> u64 {
        if is_wii { self.dol_offset.get() as u64 * 4 } else { self.dol_offset.get() as u64 }
    }

    /// Set the offset within the partition to the main DOL.
    #[inline]
    pub fn set_dol_offset(&mut self, offset: u64, is_wii: bool) {
        if is_wii {
            self.dol_offset.set((offset / 4) as u32);
        } else {
            self.dol_offset.set(offset as u32);
        }
    }

    /// Offset within the partition to the file system table (FST).
    #[inline]
    pub fn fst_offset(&self, is_wii: bool) -> u64 {
        if is_wii { self.fst_offset.get() as u64 * 4 } else { self.fst_offset.get() as u64 }
    }

    /// Set the offset within the partition to the file system table (FST).
    #[inline]
    pub fn set_fst_offset(&mut self, offset: u64, is_wii: bool) {
        if is_wii {
            self.fst_offset.set((offset / 4) as u32);
        } else {
            self.fst_offset.set(offset as u32);
        }
    }

    /// Size of the file system table (FST).
    #[inline]
    pub fn fst_size(&self, is_wii: bool) -> u64 {
        if is_wii { self.fst_size.get() as u64 * 4 } else { self.fst_size.get() as u64 }
    }

    /// Set the size of the file system table (FST).
    #[inline]
    pub fn set_fst_size(&mut self, size: u64, is_wii: bool) {
        if is_wii {
            self.fst_size.set((size / 4) as u32);
        } else {
            self.fst_size.set(size as u32);
        }
    }

    /// Maximum size of the file system table (FST) across multi-disc games.
    #[inline]
    pub fn fst_max_size(&self, is_wii: bool) -> u64 {
        if is_wii { self.fst_max_size.get() as u64 * 4 } else { self.fst_max_size.get() as u64 }
    }

    /// Set the maximum size of the file system table (FST) across multi-disc games.
    #[inline]
    pub fn set_fst_max_size(&mut self, size: u64, is_wii: bool) {
        if is_wii {
            self.fst_max_size.set((size / 4) as u32);
        } else {
            self.fst_max_size.set(size as u32);
        }
    }
}

/// Apploader header.
#[derive(Debug, PartialEq, Clone, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct ApploaderHeader {
    /// Apploader build date
    pub date: [u8; 16],
    /// Entry point
    pub entry_point: U32,
    /// Apploader size
    pub size: U32,
    /// Apploader trailer size
    pub trailer_size: U32,
    /// Padding
    _pad: [u8; 4],
}

impl ApploaderHeader {
    /// Apploader build date as a string.
    #[inline]
    pub fn date_str(&self) -> Option<&str> {
        CStr::from_bytes_until_nul(&self.date).ok().and_then(|c| c.to_str().ok())
    }
}

/// Maximum number of text sections in a DOL.
pub const DOL_MAX_TEXT_SECTIONS: usize = 7;
/// Maximum number of data sections in a DOL.
pub const DOL_MAX_DATA_SECTIONS: usize = 11;

/// Dolphin executable (DOL) header.
#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct DolHeader {
    /// Text section offsets
    pub text_offs: [U32; DOL_MAX_TEXT_SECTIONS],
    /// Data section offsets
    pub data_offs: [U32; DOL_MAX_DATA_SECTIONS],
    /// Text section addresses
    pub text_addrs: [U32; DOL_MAX_TEXT_SECTIONS],
    /// Data section addresses
    pub data_addrs: [U32; DOL_MAX_DATA_SECTIONS],
    /// Text section sizes
    pub text_sizes: [U32; DOL_MAX_TEXT_SECTIONS],
    /// Data section sizes
    pub data_sizes: [U32; DOL_MAX_DATA_SECTIONS],
    /// BSS address
    pub bss_addr: U32,
    /// BSS size
    pub bss_size: U32,
    /// Entry point
    pub entry_point: U32,
    /// Padding
    _pad: [u8; 0x1C],
}

static_assert!(size_of::<DolHeader>() == 0x100);
