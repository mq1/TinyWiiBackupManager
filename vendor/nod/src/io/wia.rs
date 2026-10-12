use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, hash_map::Entry},
    io,
    io::{Seek, SeekFrom},
    mem::size_of,
    sync::Arc,
    time::Instant,
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tracing::{debug, instrument, warn};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, big_endian::*};

use crate::{
    Error, IoResultContext, Result, ResultContext,
    common::{Compression, Format, HashBytes, KeyBytes, MagicBytes},
    disc::{
        BootHeader, DiscHeader, SECTOR_SIZE,
        fst::Fst,
        hashes::GroupHashes,
        reader::DiscReader,
        wii::{HASHES_SIZE, SECTOR_DATA_SIZE},
        writer::{BlockProcessor, BlockResult, DiscWriter, par_process, read_block},
    },
    io::{
        block::{Block, BlockKind, BlockReader, RVZ_MAGIC, WIA_MAGIC},
        nkit::NKitHeader,
    },
    read::{DiscMeta, DiscStream},
    util::{
        Align,
        aes::{aes_cbc_decrypt_b2b, decrypt_sector_data_b2b},
        array_ref, array_ref_mut,
        compress::{Compressor, DecompressionKind},
        digest::{DigestManager, sha1_hash, xxh64_hash},
        lfg::{LaggedFibonacci, SEED_SIZE, SEED_SIZE_BYTES},
        read::{
            read_arc_slice_at, read_at, read_box_slice_at, read_into_arc_slice,
            read_into_box_slice, read_vec_at,
        },
        static_assert,
    },
    write::{DataCallback, DiscFinalization, DiscWriterWeight, FormatOptions, ProcessOptions},
};

const WIA_VERSION: u32 = 0x01000000;
const WIA_VERSION_WRITE_COMPATIBLE: u32 = 0x01000000;
const WIA_VERSION_READ_COMPATIBLE: u32 = 0x00080000;

const RVZ_VERSION: u32 = 0x01000000;
const RVZ_VERSION_WRITE_COMPATIBLE: u32 = 0x00030000;
const RVZ_VERSION_READ_COMPATIBLE: u32 = 0x00030000;

/// This struct is stored at offset 0x0 and is 0x48 bytes long. The wit source code says its format
/// will never be changed.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WIAFileHeader {
    pub magic: MagicBytes,
    /// The WIA format version.
    ///
    /// A short note from the wit source code about how version numbers are encoded:
    ///
    /// ```c
    /// //-----------------------------------------------------
    /// // Format of version number: AABBCCDD = A.BB | A.BB.CC
    /// // If D != 0x00 && D != 0xff => append: 'beta' D
    /// //-----------------------------------------------------
    /// ```
    pub version: U32,
    /// If the reading program supports the version of WIA indicated here, it can read the file.
    ///
    /// [version](Self::version) can be higher than `version_compatible`.
    pub version_compatible: U32,
    /// The size of the [WIADisc] struct.
    pub disc_size: U32,
    /// The SHA-1 hash of the [WIADisc] struct.
    ///
    /// The number of bytes to hash is determined by [disc_size](Self::disc_size).
    pub disc_hash: HashBytes,
    /// The original size of the ISO.
    pub iso_file_size: U64,
    /// The size of this file.
    pub wia_file_size: U64,
    /// The SHA-1 hash of this struct, up to but not including `file_head_hash` itself.
    pub file_head_hash: HashBytes,
}

static_assert!(size_of::<WIAFileHeader>() == 0x48);

impl WIAFileHeader {
    pub fn validate(&self) -> Result<()> {
        // Check magic
        if self.magic != WIA_MAGIC && self.magic != RVZ_MAGIC {
            return Err(Error::DiscFormat(format!("Invalid WIA/RVZ magic: {:#X?}", self.magic)));
        }
        // Check version
        let is_rvz = self.magic == RVZ_MAGIC;
        let version = if is_rvz { RVZ_VERSION } else { WIA_VERSION };
        let version_read_compat =
            if is_rvz { RVZ_VERSION_READ_COMPATIBLE } else { WIA_VERSION_READ_COMPATIBLE };
        if version < self.version_compatible.get() || version_read_compat > self.version.get() {
            return Err(Error::DiscFormat(format!(
                "Unsupported WIA/RVZ version: {:#X}",
                self.version.get()
            )));
        }
        // Check file head hash
        let bytes = self.as_bytes();
        verify_hash(&bytes[..bytes.len() - size_of::<HashBytes>()], &self.file_head_hash)?;
        // Check version compatibility
        if self.version_compatible.get() < 0x30000 {
            return Err(Error::DiscFormat(format!(
                "WIA/RVZ version {:#X} is not supported",
                self.version_compatible
            )));
        }
        Ok(())
    }

    pub fn is_rvz(&self) -> bool { self.magic == RVZ_MAGIC }
}

/// Disc kind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscKind {
    /// GameCube disc
    GameCube,
    /// Wii disc
    Wii,
}

impl From<DiscKind> for u32 {
    fn from(value: DiscKind) -> Self {
        match value {
            DiscKind::GameCube => 1,
            DiscKind::Wii => 2,
        }
    }
}

impl From<DiscKind> for U32 {
    fn from(value: DiscKind) -> Self { u32::from(value).into() }
}

impl TryFrom<u32> for DiscKind {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            1 => Ok(Self::GameCube),
            2 => Ok(Self::Wii),
            v => Err(Error::DiscFormat(format!("Invalid disc type {}", v))),
        }
    }
}

/// Compression type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WIACompression {
    /// No compression.
    None,
    /// (WIA only) See [WIASegment]
    Purge,
    /// BZIP2 compression
    Bzip2,
    /// LZMA compression
    Lzma,
    /// LZMA2 compression
    Lzma2,
    /// (RVZ only) Zstandard compression
    Zstandard,
}

impl From<WIACompression> for u32 {
    fn from(value: WIACompression) -> Self {
        match value {
            WIACompression::None => 0,
            WIACompression::Purge => 1,
            WIACompression::Bzip2 => 2,
            WIACompression::Lzma => 3,
            WIACompression::Lzma2 => 4,
            WIACompression::Zstandard => 5,
        }
    }
}

impl From<WIACompression> for U32 {
    fn from(value: WIACompression) -> Self { u32::from(value).into() }
}

impl TryFrom<u32> for WIACompression {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Purge),
            2 => Ok(Self::Bzip2),
            3 => Ok(Self::Lzma),
            4 => Ok(Self::Lzma2),
            5 => Ok(Self::Zstandard),
            v => Err(Error::DiscFormat(format!("Invalid compression type {}", v))),
        }
    }
}

const DISC_HEAD_SIZE: usize = 0x80;

/// This struct is stored at offset 0x48, immediately after [WIAFileHeader].
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WIADisc {
    /// The disc type. (1 = GameCube, 2 = Wii)
    pub disc_type: U32,
    /// The compression type.
    pub compression: U32,
    /// The compression level used by the compressor.
    ///
    /// The possible values are compressor-specific.
    ///
    /// RVZ only:
    /// > This is signed (instead of unsigned) to support negative compression levels in
    /// > [Zstandard](WIACompression::Zstandard) (RVZ only).
    pub compression_level: I32,
    /// The size of the chunks that data is divided into.
    ///
    /// WIA only:
    /// > Must be a multiple of 2 MiB.
    ///
    /// RVZ only:
    /// > Chunk sizes smaller than 2 MiB are supported. The following applies when using a chunk size
    /// > smaller than 2 MiB:
    /// > - The chunk size must be at least 32 KiB and must be a power of two. (Just like with WIA,
    /// >   sizes larger than 2 MiB do not have to be a power of two, they just have to be an integer
    /// >   multiple of 2 MiB.)
    /// > - For Wii partition data, each chunk contains one [WIAExceptionList] which contains
    /// >   exceptions for that chunk (and no other chunks). Offset 0 refers to the first hash of the
    /// >   current chunk, not the first hash of the full 2 MiB of data.
    pub chunk_size: U32,
    /// The first 0x80 bytes of the disc image.
    pub disc_head: [u8; DISC_HEAD_SIZE],
    /// The number of [WIAPartition] structs.
    pub num_partitions: U32,
    /// The size of one [WIAPartition] struct.
    ///
    /// If this is smaller than the size of [WIAPartition], fill the missing bytes with 0x00.
    pub partition_type_size: U32,
    /// The offset in the file where the [WIAPartition] structs are stored (uncompressed).
    pub partition_offset: U64,
    /// The SHA-1 hash of the [WIAPartition] structs.
    ///
    /// The number of bytes to hash is determined by `num_partitions * partition_type_size`.
    pub partition_hash: HashBytes,
    /// The number of [WIARawData] structs.
    pub num_raw_data: U32,
    /// The offset in the file where the [WIARawData] structs are stored (compressed).
    pub raw_data_offset: U64,
    /// The total compressed size of the [WIARawData] structs.
    pub raw_data_size: U32,
    /// The number of [WIAGroup] structs.
    pub num_groups: U32,
    /// The offset in the file where the [WIAGroup] structs are stored (compressed).
    pub group_offset: U64,
    /// The total compressed size of the [WIAGroup] structs.
    pub group_size: U32,
    /// The number of used bytes in the [compr_data](Self::compr_data) array.
    pub compr_data_len: u8,
    /// Compressor specific data.
    ///
    /// If the compression method is [None](WIACompression::None), [Purge](WIACompression::Purge),
    /// [Bzip2](WIACompression::Bzip2), or [Zstandard](WIACompression::Zstandard) (RVZ only),
    /// [compr_data_len](Self::compr_data_len) is 0. If the compression method is
    /// [Lzma](WIACompression::Lzma) or [Lzma2](WIACompression::Lzma2), the compressor specific data is
    /// stored in the format used by the 7-Zip SDK. It needs to be converted if you are using e.g.
    /// liblzma.
    ///
    /// For [Lzma](WIACompression::Lzma), the data is 5 bytes long. The first byte encodes the `lc`,
    /// `pb`, and `lp` parameters, and the four other bytes encode the dictionary size in little
    /// endian.
    pub compr_data: [u8; 7],
}

static_assert!(size_of::<WIADisc>() == 0xDC);

impl WIADisc {
    pub fn validate(&self, is_rvz: bool) -> Result<()> {
        DiscKind::try_from(self.disc_type.get())?;
        WIACompression::try_from(self.compression.get())?;
        let chunk_size = self.chunk_size.get();
        if is_rvz {
            if chunk_size < SECTOR_SIZE as u32 || !chunk_size.is_power_of_two() {
                return Err(Error::DiscFormat(format!(
                    "Invalid RVZ chunk size: {:#X}",
                    chunk_size
                )));
            }
        } else if chunk_size < 0x200000 || chunk_size % 0x200000 != 0 {
            return Err(Error::DiscFormat(format!("Invalid WIA chunk size: {:#X}", chunk_size)));
        }
        if self.partition_type_size.get() != size_of::<WIAPartition>() as u32 {
            return Err(Error::DiscFormat(format!(
                "WIA/RVZ partition type size is {}, expected {}",
                self.partition_type_size.get(),
                size_of::<WIAPartition>()
            )));
        }
        Ok(())
    }

    pub fn compression(&self) -> WIACompression {
        WIACompression::try_from(self.compression.get()).unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WIAPartitionData {
    /// The sector on the disc at which this data starts.
    /// One sector is 32 KiB (or 31 KiB excluding hashes).
    pub first_sector: U32,
    /// The number of sectors on the disc covered by this struct.
    /// One sector is 32 KiB (or 31 KiB excluding hashes).
    pub num_sectors: U32,
    /// The index of the first [WIAGroup] struct that points to the data covered by this struct.
    /// The other [WIAGroup] indices follow sequentially.
    pub group_index: U32,
    /// The number of [WIAGroup] structs used for this data.
    pub num_groups: U32,
}

static_assert!(size_of::<WIAPartitionData>() == 0x10);

impl WIAPartitionData {
    pub fn start_sector(&self) -> u32 { self.first_sector.get() }

    pub fn end_sector(&self) -> u32 { self.first_sector.get() + self.num_sectors.get() }

    pub fn contains_sector(&self, sector: u32) -> bool {
        let start = self.first_sector.get();
        sector >= start && sector < start + self.num_sectors.get()
    }

    pub fn contains_group(&self, group: u32) -> bool {
        let start = self.group_index.get();
        group >= start && group < start + self.num_groups.get()
    }
}

/// This struct is used for keeping track of Wii partition data that on the actual disc is encrypted
/// and hashed. This does not include the unencrypted area at the beginning of partitions that
/// contains the ticket, TMD, certificate chain, and H3 table. So for a typical game partition,
/// `pd[0].first_sector * 0x8000` would be 0x0F820000, not 0x0F800000.
///
/// Wii partition data is stored decrypted and with hashes removed. For each 0x8000 bytes on the
/// disc, 0x7C00 bytes are stored in the WIA file (prior to compression). If the hashes are desired,
/// the reading program must first recalculate the hashes as done when creating a Wii disc image
/// from scratch (see <https://wiibrew.org/wiki/Wii_Disc>), and must then apply the hash exceptions
/// which are stored along with the data (see the [WIAExceptionList] section).
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WIAPartition {
    /// The title key for this partition (128-bit AES), which can be used for re-encrypting the
    /// partition data.
    ///
    /// This key can be used directly, without decrypting it using the Wii common key.
    pub partition_key: KeyBytes,
    /// To quote the wit source code: `segment 0 is small and defined for management data (boot ..
    /// fst). segment 1 takes the remaining data.`
    ///
    /// The point at which wit splits the two segments is the FST end offset rounded up to the next
    /// 2 MiB. Giving the first segment a size which is not a multiple of 2 MiB is likely a bad idea
    /// (unless the second segment has a size of 0).
    pub partition_data: [WIAPartitionData; 2],
}

static_assert!(size_of::<WIAPartition>() == 0x30);

/// This struct is used for keeping track of disc data that is not stored as [WIAPartition].
/// The data is stored as is (other than compression being applied).
///
/// The first [WIARawData] has `raw_data_offset` set to 0x80 and `raw_data_size` set to 0x4FF80,
/// but despite this, it actually contains 0x50000 bytes of data. (However, the first 0x80 bytes
/// should be read from [WIADisc] instead.) This should be handled by rounding the offset down to
/// the previous multiple of 0x8000 (and adding the equivalent amount to the size so that the end
/// offset stays the same), not by special casing the first [WIARawData].
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WIARawData {
    /// The offset on the disc at which this data starts.
    pub raw_data_offset: U64,
    /// The number of bytes on the disc covered by this struct.
    pub raw_data_size: U64,
    /// The index of the first [WIAGroup] struct that points to the data covered by this struct.
    /// The other [WIAGroup] indices follow sequentially.
    pub group_index: U32,
    /// The number of [WIAGroup] structs used for this data.
    pub num_groups: U32,
}

impl WIARawData {
    pub fn start_offset(&self) -> u64 { self.raw_data_offset.get().align_down(SECTOR_SIZE as u64) }

    pub fn start_sector(&self) -> u32 { (self.start_offset() / SECTOR_SIZE as u64) as u32 }

    pub fn end_offset(&self) -> u64 { self.raw_data_offset.get() + self.raw_data_size.get() }

    pub fn end_sector(&self) -> u32 {
        // Round up for unaligned raw data end offsets
        self.end_offset().div_ceil(SECTOR_SIZE as u64) as u32
    }

    pub fn contains_sector(&self, sector: u32) -> bool {
        sector >= self.start_sector() && sector < self.end_sector()
    }

    pub fn contains_group(&self, group: u32) -> bool {
        let start = self.group_index.get();
        group >= start && group < start + self.num_groups.get()
    }
}

/// This struct points directly to the actual disc data, stored compressed.
///
/// The data is interpreted differently depending on whether the [WIAGroup] is referenced by a
/// [WIAPartitionData] or a [WIARawData] (see the [WIAPartition] section for details).
///
/// A [WIAGroup] normally contains chunk_size bytes of decompressed data
/// (or `chunk_size / 0x8000 * 0x7C00` for Wii partition data when not counting hashes), not
/// counting any [WIAExceptionList] structs. However, the last [WIAGroup] of a [WIAPartitionData]
/// or [WIARawData] contains less data than that if `num_sectors * 0x8000` (for [WIAPartitionData])
/// or `raw_data_size` (for [WIARawData]) is not evenly divisible by `chunk_size`.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct WIAGroup {
    /// The offset in the file where the compressed data is stored.
    ///
    /// Stored as a `u32`, divided by 4.
    pub data_offset: U32,
    /// The size of the compressed data, including any [WIAExceptionList] structs. 0 is a special
    /// case meaning that every byte of the decompressed data is 0x00 and the [WIAExceptionList]
    /// structs (if there are supposed to be any) contain 0 exceptions.
    pub data_size: U32,
}

/// Compared to [WIAGroup], [RVZGroup] changes the meaning of the most significant bit of
/// [data_size](Self::data_size) and adds one additional attribute.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(4))]
pub struct RVZGroup {
    /// The offset in the file where the compressed data is stored, divided by 4.
    pub data_offset: U32,
    /// The most significant bit is 1 if the data is compressed using the compression method
    /// indicated in [WIADisc], and 0 if it is not compressed. The lower 31 bits are the size of
    /// the compressed data, including any [WIAExceptionList] structs. The lower 31 bits being 0 is
    /// a special case meaning that every byte of the decompressed and unpacked data is 0x00 and
    /// the [WIAExceptionList] structs (if there are supposed to be any) contain 0 exceptions.
    pub data_size_and_flag: U32,
    /// The size after decompressing but before decoding the RVZ packing.
    /// If this is 0, RVZ packing is not used for this group.
    pub rvz_packed_size: U32,
}

const COMPRESSED_BIT: u32 = 1 << 31;

impl RVZGroup {
    #[inline]
    pub fn data_size(&self) -> u32 { self.data_size_and_flag.get() & !COMPRESSED_BIT }

    #[inline]
    pub fn is_compressed(&self) -> bool { self.data_size_and_flag.get() & COMPRESSED_BIT != 0 }
}

impl From<&WIAGroup> for RVZGroup {
    fn from(value: &WIAGroup) -> Self {
        Self {
            data_offset: value.data_offset,
            data_size_and_flag: U32::new(value.data_size.get() | COMPRESSED_BIT),
            rvz_packed_size: U32::new(0),
        }
    }
}

impl From<&RVZGroup> for WIAGroup {
    fn from(value: &RVZGroup) -> Self {
        Self { data_offset: value.data_offset, data_size: value.data_size().into() }
    }
}

/// This struct represents a 20-byte difference between the recalculated hash data and the original
/// hash data. (See also [WIAExceptionList])
///
/// When recalculating hashes for a [WIAGroup] with a size which is not evenly divisible by 2 MiB
/// (with the size of the hashes included), the missing bytes should be treated as zeroes for the
/// purpose of hashing. (wit's writing code seems to act as if the reading code does not assume that
/// these missing bytes are zero, but both wit's and Dolphin's reading code treat them as zero.
/// Dolphin's writing code assumes that the reading code treats them as zero.)
///
/// wit's writing code only outputs [WIAException] structs for mismatches in the actual hash
/// data, not in the padding data (which normally only contains zeroes). Dolphin's writing code
/// outputs [WIAException] structs for both hash data and padding data. When Dolphin needs to
/// write [WIAException] structs for a padding area which is 32 bytes long, it writes one which
/// covers the first 20 bytes of the padding area and one which covers the last 20 bytes of the
/// padding area, generating 12 bytes of overlap between the [WIAException] structs.
#[derive(Clone, Debug, PartialEq, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(C, align(2))]
pub struct WIAException {
    /// The offset among the hashes. The offsets 0x0000-0x0400 here map to the offsets 0x0000-0x0400
    /// in the full 2 MiB of data, the offsets 0x0400-0x0800 here map to the offsets 0x8000-0x8400
    /// in the full 2 MiB of data, and so on.
    ///
    /// The offsets start over at 0 for each new [WIAExceptionList].
    pub offset: U16,
    /// The hash that the automatically generated hash at the given offset needs to be replaced
    /// with.
    ///
    /// The replacement should happen after calculating all hashes for the current 2 MiB of data
    /// but before encrypting the hashes.
    pub hash: HashBytes,
}

/// Each [WIAGroup] of Wii partition data contains one or more [WIAExceptionList] structs before
/// the actual data, one for each 2 MiB of data in the [WIAGroup]. The number of [WIAExceptionList]
/// structs per [WIAGroup] is always `chunk_size / 0x200000`, even for a [WIAGroup] which contains
/// less data than normal due to it being at the end of a partition.
///
/// For memory management reasons, programs which read WIA files might place a limit on how many
/// exceptions there can be in a [WIAExceptionList]. Dolphin's reading code has a limit of
/// `52 × 64 = 3328` (unless the compression method is [None](WIACompression::None) or
/// [Purge](WIACompression::Purge), in which case there is no limit), which is enough to cover all
/// hashes and all padding. wit's reading code seems to be written as if `47 × 64 = 3008` is the
/// maximum it needs to be able to handle, which is enough to cover all hashes but not any padding.
/// However, because wit allocates more memory than needed, it seems to be possible to exceed 3008
/// by some amount without problems. It should be safe for writing code to assume that reading code
/// can handle at least 3328 exceptions per [WIAExceptionList].
///
/// Somewhat ironically, there are exceptions to how [WIAExceptionList] structs are handled:
///
/// For the compression method [Purge](WIACompression::Purge), the [WIAExceptionList] structs are
/// stored uncompressed (in other words, before the first [WIASegment]). For
/// [Bzip2](WIACompression::Bzip2), [Lzma](WIACompression::Lzma) and [Lzma2](WIACompression::Lzma2), they are
/// compressed along with the rest of the data.
///
/// For the compression methods [None](WIACompression::None) and [Purge](WIACompression::Purge), if the
/// end offset of the last [WIAExceptionList] is not evenly divisible by 4, padding is inserted
/// after it so that the data afterwards will start at a 4 byte boundary. This padding is not
/// inserted for the other compression methods.
pub type WIAExceptionList = Box<[WIAException]>;

pub struct BlockReaderWIA {
    inner: Box<dyn DiscStream>,
    header: WIAFileHeader,
    disc: WIADisc,
    partitions: Arc<[WIAPartition]>,
    raw_data: Arc<[WIARawData]>,
    groups: Arc<[RVZGroup]>,
    nkit_header: Option<NKitHeader>,
    decompressor: DecompressionKind,
}

impl Clone for BlockReaderWIA {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            header: self.header.clone(),
            disc: self.disc.clone(),
            partitions: self.partitions.clone(),
            raw_data: self.raw_data.clone(),
            groups: self.groups.clone(),
            nkit_header: self.nkit_header.clone(),
            decompressor: self.decompressor.clone(),
        }
    }
}

fn verify_hash(buf: &[u8], expected: &HashBytes) -> Result<()> {
    let out = sha1_hash(buf);
    if out != *expected {
        let mut got_bytes = [0u8; 40];
        let got = base16ct::lower::encode_str(&out, &mut got_bytes).unwrap(); // Safe: fixed buffer size
        let mut expected_bytes = [0u8; 40];
        let expected = base16ct::lower::encode_str(expected, &mut expected_bytes).unwrap(); // Safe: fixed buffer size
        return Err(Error::DiscFormat(format!(
            "WIA/RVZ hash mismatch: {}, expected {}",
            got, expected
        )));
    }
    Ok(())
}

impl BlockReaderWIA {
    pub fn new(mut inner: Box<dyn DiscStream>) -> Result<Box<Self>> {
        // Load & verify file header
        let header: WIAFileHeader =
            read_at(inner.as_mut(), 0).context("Reading WIA/RVZ file header")?;
        header.validate()?;
        let is_rvz = header.is_rvz();
        debug!("Header: {:?}", header);

        // Load & verify disc header
        let mut disc_buf: Vec<u8> = read_vec_at(
            inner.as_mut(),
            header.disc_size.get() as usize,
            size_of::<WIAFileHeader>() as u64,
        )
        .context("Reading WIA/RVZ disc header")?;
        verify_hash(&disc_buf, &header.disc_hash)?;
        disc_buf.resize(size_of::<WIADisc>(), 0);
        let disc = WIADisc::read_from_bytes(disc_buf.as_slice()).unwrap();
        disc.validate(is_rvz)?;
        debug!("Disc: {:?}", disc);

        // Read NKit header if present (after disc header)
        let nkit_header = NKitHeader::try_read_from(
            inner.as_mut(),
            size_of::<WIAFileHeader>() as u64 + header.disc_size.get() as u64,
            disc.chunk_size.get(),
            false,
        );

        // Load & verify partition headers
        let partitions: Arc<[WIAPartition]> = read_arc_slice_at(
            inner.as_mut(),
            disc.num_partitions.get() as usize,
            disc.partition_offset.get(),
        )
        .context("Reading WIA/RVZ partition headers")?;
        verify_hash(partitions.as_ref().as_bytes(), &disc.partition_hash)?;
        debug!("Partitions: {:?}", partitions);

        // Create decompressor
        let decompressor = DecompressionKind::from_wia(&disc)?;

        // Load raw data headers
        let raw_data: Arc<[WIARawData]> = {
            let compressed_data = read_box_slice_at::<u8, _>(
                inner.as_mut(),
                disc.raw_data_size.get() as usize,
                disc.raw_data_offset.get(),
            )
            .context("Reading WIA/RVZ raw data headers")?;
            read_into_arc_slice(disc.num_raw_data.get() as usize, |out| {
                decompressor
                    .decompress(&compressed_data, out)
                    .context("Decompressing WIA/RVZ raw data headers")
                    .map(|_| ())
            })?
        };
        // Validate raw data alignment
        for (idx, rd) in raw_data.iter().enumerate() {
            let start_offset = rd.start_offset();
            let end_offset = rd.end_offset();
            let is_last = idx == raw_data.len() - 1;
            if (start_offset % SECTOR_SIZE as u64) != 0
                // Allow raw data end to be unaligned if it's the last
                || (!is_last && (end_offset % SECTOR_SIZE as u64) != 0)
            {
                return Err(Error::DiscFormat(format!(
                    "WIA/RVZ raw data {} not aligned to sector: {:#X}..{:#X}",
                    idx, start_offset, end_offset
                )));
            }
        }
        debug!("Num raw data: {}", raw_data.len());
        // debug!("Raw data: {:?}", raw_data);

        // Load group headers
        let groups = {
            let compressed_data = read_box_slice_at::<u8, _>(
                inner.as_mut(),
                disc.group_size.get() as usize,
                disc.group_offset.get(),
            )
            .context("Reading WIA/RVZ group headers")?;
            if is_rvz {
                read_into_arc_slice(disc.num_groups.get() as usize, |out| {
                    decompressor
                        .decompress(&compressed_data, out)
                        .context("Decompressing WIA/RVZ group headers")
                        .map(|_| ())
                })?
            } else {
                let wia_groups =
                    read_into_box_slice::<WIAGroup, _>(disc.num_groups.get() as usize, |out| {
                        decompressor
                            .decompress(&compressed_data, out)
                            .context("Decompressing WIA/RVZ group headers")
                            .map(|_| ())
                    })?;
                wia_groups.iter().map(RVZGroup::from).collect()
            }
        };
        debug!("Num groups: {}", groups.len());
        // std::fs::write("groups.txt", format!("Groups: {:#?}", groups)).unwrap();

        Ok(Box::new(Self {
            header,
            disc,
            partitions,
            raw_data,
            groups,
            inner,
            nkit_header,
            decompressor,
        }))
    }
}

fn read_exception_lists(
    bytes: &mut Bytes,
    chunk_size: u32,
    align: bool,
) -> io::Result<Vec<WIAExceptionList>> {
    let initial_remaining = bytes.remaining();
    // One exception list for each 2 MiB of data
    let num_exception_list = (chunk_size as usize).div_ceil(0x200000);
    let mut exception_lists = vec![WIAExceptionList::default(); num_exception_list];
    for exception_list in exception_lists.iter_mut() {
        if bytes.remaining() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Reading WIA/RVZ exception list count",
            ));
        }
        let num_exceptions = bytes.get_u16();
        if bytes.remaining() < num_exceptions as usize * size_of::<WIAException>() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Reading WIA/RVZ exception list",
            ));
        }
        let mut exceptions =
            <[WIAException]>::new_box_zeroed_with_elems(num_exceptions as usize).unwrap();
        bytes.copy_to_slice(exceptions.as_mut_bytes());
        if !exceptions.is_empty() {
            debug!("Exception list: {:?}", exceptions);
        }
        *exception_list = exceptions;
    }
    if align {
        let rem = (initial_remaining - bytes.remaining()) % 4;
        if rem != 0 {
            bytes.advance(4 - rem);
        }
    }
    Ok(exception_lists)
}

struct GroupInfo {
    /// The group index.
    index: u32,
    /// The disc sector at which the group starts.
    sector: u32,
    /// The number of sectors in the group.
    num_sectors: u32,
    /// The size of the group data in bytes. Usually equal to the chunk size (or
    /// `chunk_size / 0x8000 * 0x7C00` for Wii partition data), but can be smaller for the last
    /// group in a partition or raw data.
    size: u32,
    /// The offset within the section where the group data starts. For partition data, this is the
    /// offset (excluding hashes) from the start of the partition. For raw data, this is the offset
    /// from the start of the disc.
    section_offset: u64,
    /// Whether the group is in a partition (raw data otherwise).
    in_partition: bool,
    /// The partition title key (if encrypted).
    partition_key: Option<KeyBytes>,
}

impl GroupInfo {
    fn from_partition(index: u32, disc: &WIADisc, p: &WIAPartition, pd: &WIAPartitionData) -> Self {
        let sectors_per_chunk = disc.chunk_size.get() / SECTOR_SIZE as u32;
        let rel_group_idx = index - pd.group_index.get();
        // Disc sector at which the group starts
        let sector = pd.start_sector() + rel_group_idx * sectors_per_chunk;
        // Size of the group, limited by the end of the partition data
        let num_sectors = (pd.end_sector() - sector).min(sectors_per_chunk);
        let size = num_sectors * SECTOR_DATA_SIZE as u32;
        // Data offset within partition data (from start of partition)
        let partition_offset =
            (sector - p.partition_data[0].start_sector()) as u64 * SECTOR_DATA_SIZE as u64;
        // Check if the partition is encrypted
        let partition_key = (disc.disc_head[0x61] == 0).then_some(p.partition_key);
        Self {
            index,
            sector,
            num_sectors,
            size,
            section_offset: partition_offset,
            in_partition: true,
            partition_key,
        }
    }

    fn from_raw_data(index: u32, disc: &WIADisc, rd: &WIARawData) -> Self {
        let chunk_size = disc.chunk_size.get();
        let sectors_per_chunk = chunk_size / SECTOR_SIZE as u32;
        let rel_group_idx = index - rd.group_index.get();
        // Disc sector at which the group starts
        let sector = rd.start_sector() + rel_group_idx * sectors_per_chunk;
        // Size of the group, limited by the end of the raw data
        let size =
            (rd.end_offset() - (sector as u64 * SECTOR_SIZE as u64)).min(chunk_size as u64) as u32;
        let num_sectors = size.div_ceil(SECTOR_SIZE as u32);
        // Data offset within disc data
        let partition_offset = sector as u64 * SECTOR_SIZE as u64;
        Self {
            index,
            sector,
            num_sectors,
            size,
            section_offset: partition_offset,
            in_partition: false,
            partition_key: None,
        }
    }
}

fn find_group_info(
    idx: u32,
    disc: &WIADisc,
    partitions: &[WIAPartition],
    raw_data: &[WIARawData],
) -> Option<GroupInfo> {
    partitions
        .iter()
        .find_map(|p| {
            p.partition_data.iter().find_map(|pd| {
                pd.contains_group(idx).then(|| GroupInfo::from_partition(idx, disc, p, pd))
            })
        })
        .or_else(|| {
            raw_data.iter().find_map(|rd| {
                rd.contains_group(idx).then(|| GroupInfo::from_raw_data(idx, disc, rd))
            })
        })
}

fn find_group_info_for_sector(
    sector: u32,
    disc: &WIADisc,
    partitions: &[WIAPartition],
    raw_data: &[WIARawData],
) -> Option<GroupInfo> {
    let sectors_per_chunk = disc.chunk_size.get() / SECTOR_SIZE as u32;
    partitions
        .iter()
        .find_map(|p| {
            p.partition_data.iter().find_map(|pd| {
                pd.contains_sector(sector).then(|| {
                    let rel_group_idx = (sector - pd.start_sector()) / sectors_per_chunk;
                    GroupInfo::from_partition(pd.group_index.get() + rel_group_idx, disc, p, pd)
                })
            })
        })
        .or_else(|| {
            raw_data.iter().find_map(|rd| {
                rd.contains_sector(sector).then(|| {
                    let rel_group_idx = (sector - rd.start_sector()) / sectors_per_chunk;
                    GroupInfo::from_raw_data(rd.group_index.get() + rel_group_idx, disc, rd)
                })
            })
        })
}

impl BlockReader for BlockReaderWIA {
    #[instrument(name = "BlockReaderWIA::read_block", skip_all)]
    fn read_block(&mut self, out: &mut [u8], sector: u32) -> io::Result<Block> {
        let Some(info) =
            find_group_info_for_sector(sector, &self.disc, &self.partitions, &self.raw_data)
        else {
            // Out of bounds
            return Ok(Block::sector(sector, BlockKind::None));
        };
        if info.size as usize > out.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Output buffer too small for WIA/RVZ group data: {} < {}",
                    out.len(),
                    info.size
                ),
            ));
        }

        // Fetch the group
        let Some(group) = self.groups.get(info.index as usize) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Couldn't find WIA/RVZ group index {}", info.index),
            ));
        };

        // Special case for all-zero data
        if group.data_size() == 0 {
            return Ok(Block::sectors(info.sector, info.num_sectors, BlockKind::Zero));
        }

        let group_data_start = group.data_offset.get() as u64 * 4;
        let mut group_data = BytesMut::zeroed(group.data_size() as usize);
        let io_start = Instant::now();
        self.inner.read_exact_at(group_data.as_mut(), group_data_start)?;
        let io_duration = io_start.elapsed();
        let mut group_data = group_data.freeze();

        let chunk_size = self.disc.chunk_size.get();
        let uncompressed_exception_lists =
            matches!(self.disc.compression(), WIACompression::None | WIACompression::Purge)
                || !group.is_compressed();
        let mut exception_lists = vec![];
        if info.in_partition && uncompressed_exception_lists {
            exception_lists = read_exception_lists(&mut group_data, chunk_size, true)
                .io_context("Reading uncompressed exception lists")?;
        }
        let mut decompressed = if group.is_compressed() {
            let max_decompressed_size =
                self.decompressor.get_content_size(group_data.as_ref())?.unwrap_or_else(|| {
                    if info.in_partition && !uncompressed_exception_lists {
                        // Add room for potential hash exceptions. See [WIAExceptionList] for details on the
                        // maximum number of exceptions.
                        chunk_size as usize
                            + (2 + 3328 * size_of::<WIAException>())
                                * (chunk_size as usize).div_ceil(0x200000)
                    } else {
                        chunk_size as usize
                    }
                });
            let mut decompressed = BytesMut::zeroed(max_decompressed_size);
            let len = self
                .decompressor
                .decompress(group_data.as_ref(), decompressed.as_mut())
                .io_context("Decompressing group data")?;
            decompressed.truncate(len);
            decompressed.freeze()
        } else {
            group_data
        };
        if info.in_partition && !uncompressed_exception_lists {
            exception_lists = read_exception_lists(&mut decompressed, chunk_size, false)
                .io_context("Reading compressed exception lists")?;
        }

        if group.rvz_packed_size.get() > 0 {
            // Decode RVZ packed data
            rvz_unpack(&mut decompressed, out, &info).io_context("Unpacking RVZ group data")?;
        } else {
            // Read and decompress data
            if decompressed.remaining() != info.size as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "WIA/RVZ group {} data size mismatch: {} != {}",
                        info.index,
                        decompressed.remaining(),
                        info.size
                    ),
                ));
            }
            decompressed.copy_to_slice(&mut out[..info.size as usize]);
        }
        if !decompressed.is_empty() {
            return Err(io::Error::other("Failed to consume all group data"));
        }

        // Read first 0x80 bytes from disc header
        if info.sector == 0 {
            *array_ref_mut![out, 0, DISC_HEAD_SIZE] = self.disc.disc_head;
        }

        let mut block = if info.in_partition {
            let mut block =
                Block::sectors(info.sector, info.num_sectors, BlockKind::PartDecrypted {
                    hash_block: false,
                });
            block.hash_exceptions = exception_lists.into_boxed_slice();
            block
        } else {
            Block::sectors(info.sector, info.num_sectors, BlockKind::Raw)
        };
        block.io_duration = Some(io_duration);
        Ok(block)
    }

    fn block_size(&self) -> u32 { self.disc.chunk_size.get() }

    fn meta(&self) -> DiscMeta {
        let level = self.disc.compression_level.get();
        let mut result = DiscMeta {
            format: if self.header.is_rvz() { Format::Rvz } else { Format::Wia },
            block_size: Some(self.disc.chunk_size.get()),
            compression: match self.disc.compression() {
                WIACompression::None | WIACompression::Purge => Compression::None,
                WIACompression::Bzip2 => Compression::Bzip2(level as u8),
                WIACompression::Lzma => Compression::Lzma(level as u8),
                WIACompression::Lzma2 => Compression::Lzma2(level as u8),
                WIACompression::Zstandard => Compression::Zstandard(level as i8),
            },
            decrypted: true,
            needs_hash_recovery: true,
            lossless: true,
            disc_size: Some(self.header.iso_file_size.get()),
            ..Default::default()
        };
        if let Some(nkit_header) = &self.nkit_header {
            nkit_header.apply(&mut result);
        }
        result
    }
}

#[instrument(name = "rvz_unpack", skip_all)]
fn rvz_unpack(data: &mut impl Buf, out: &mut [u8], info: &GroupInfo) -> io::Result<()> {
    let mut read = 0;
    let mut lfg = LaggedFibonacci::default();
    while data.remaining() >= 4 {
        let size = data.get_u32();
        let remain = out.len() - read;
        if size & COMPRESSED_BIT != 0 {
            // Junk data
            let size = size & !COMPRESSED_BIT;
            if size as usize > remain {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("RVZ packed junk size too large: {} > {}", size, remain),
                ));
            }
            lfg.init_with_buf(data)?;
            lfg.skip(((info.section_offset + read as u64) % SECTOR_SIZE as u64) as usize);
            lfg.fill(&mut out[read..read + size as usize]);
            read += size as usize;
        } else {
            // Real data
            if size as usize > remain {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("RVZ packed data size too large: {} > {}", size, remain),
                ));
            }
            data.copy_to_slice(&mut out[read..read + size as usize]);
            read += size as usize;
        }
    }
    if read != info.size as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("RVZ packed data size mismatch: {} != {}", read, info.size),
        ));
    }
    Ok(())
}

struct BlockProcessorWIA {
    inner: DiscReader,
    header: WIAFileHeader,
    disc: WIADisc,
    partitions: Arc<[WIAPartition]>,
    raw_data: Arc<[WIARawData]>,
    compressor: Compressor,
    lfg: LaggedFibonacci,
    junk_info: Arc<[JunkInfo]>,
    hash_group: Option<(u32, Arc<GroupHashes>)>,
}

impl Clone for BlockProcessorWIA {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            header: self.header.clone(),
            disc: self.disc.clone(),
            partitions: self.partitions.clone(),
            raw_data: self.raw_data.clone(),
            compressor: self.compressor.clone(),
            lfg: LaggedFibonacci::default(),
            junk_info: self.junk_info.clone(),
            hash_group: self.hash_group.clone(),
        }
    }
}

#[derive(Default)]
struct BlockMetaWIA {
    is_compressed: bool,
    data_size: u32, // Not aligned
    rvz_packed_size: u32,
    data_hash: u64,
}

/// How the LFG seed for a packed junk area is obtained.
enum JunkSeed {
    /// All-zero data, packed as an all-zeroes seed.
    Zeroed,
    /// Standard junk: seed derived from the disc ID, disc number, and sector.
    Sector(u32),
    /// Seed recovered from the junk data itself (e.g. when the disc header's game ID no longer
    /// matches the ID the junk was generated from).
    Recovered([u32; SEED_SIZE]),
}

#[derive(Clone)]
struct JunkInfo {
    start_sector: u32,
    end_sector: u32,
    file_ends: BTreeSet<u64>,
    disc_id: [u8; 4],
    disc_num: u8,
}

impl JunkInfo {
    fn from_fst(
        start_sector: u32,
        end_sector: u32,
        disc_header: &DiscHeader,
        boot_header: Option<&BootHeader>,
        fst: Option<Fst>,
    ) -> Self {
        let is_wii = disc_header.is_wii();
        let mut file_ends = BTreeSet::new();
        if let Some(boot_header) = boot_header {
            file_ends.insert(boot_header.fst_offset(is_wii) + boot_header.fst_size(is_wii));
        }
        if let Some(fst) = fst {
            for entry in fst.nodes.iter().filter(|n| n.is_file()) {
                file_ends.insert(entry.offset(is_wii) + entry.length() as u64);
            }
        }
        Self {
            start_sector,
            end_sector,
            file_ends,
            disc_id: *array_ref![disc_header.game_id, 0, 4],
            disc_num: disc_header.disc_num,
        }
    }
}

impl BlockProcessor for BlockProcessorWIA {
    type BlockMeta = BlockMetaWIA;

    #[instrument(name = "BlockProcessorWIA::process_block", skip_all)]
    fn process_block(&mut self, group_idx: u32) -> io::Result<BlockResult<Self::BlockMeta>> {
        let info = find_group_info(
            group_idx,
            &self.disc,
            self.partitions.as_ref(),
            self.raw_data.as_ref(),
        )
        .ok_or_else(|| {
            io::Error::other(format!("Couldn't find partition or raw data for group {}", group_idx))
        })?;

        self.inner.seek(SeekFrom::Start(info.sector as u64 * SECTOR_SIZE as u64))?;
        let (_, disc_data) = read_block(&mut self.inner, info.num_sectors as usize * SECTOR_SIZE)?;

        // Decrypt group and calculate hash exceptions
        let is_rvz = self.header.is_rvz();
        let chunk_size = self.disc.chunk_size.get() as u64;
        let (mut group_data, hash_exception_data) = if info.in_partition {
            if info.size % SECTOR_DATA_SIZE as u32 != 0 {
                return Err(io::Error::other("Partition group size not aligned to sector"));
            }

            let mut buf = BytesMut::zeroed(info.size as usize);
            if let Some(key) = info.partition_key {
                // Encrypted partition
                for i in 0..info.num_sectors as usize {
                    decrypt_sector_data_b2b(
                        array_ref![disc_data, i * SECTOR_SIZE, SECTOR_SIZE],
                        array_ref_mut![buf, i * SECTOR_DATA_SIZE, SECTOR_DATA_SIZE],
                        &key,
                    );
                }
            } else {
                // Unencrypted partition
                for i in 0..info.num_sectors as usize {
                    *array_ref_mut![buf, i * SECTOR_DATA_SIZE, SECTOR_DATA_SIZE] =
                        *array_ref![disc_data, i * SECTOR_SIZE + HASHES_SIZE, SECTOR_DATA_SIZE];
                }
            }

            let exceptions = self.hash_exceptions(&disc_data, &info, chunk_size as usize)?;
            (buf.freeze(), exceptions)
        } else {
            (disc_data.clone(), Bytes::new())
        };

        let uncompressed_size =
            (hash_exception_data.len() as u32).align_up(4) + group_data.len() as u32;
        if hash_exception_data.as_ref().iter().all(|&b| b == 0)
            && group_data.as_ref().iter().all(|&b| b == 0)
        {
            // Skip empty group
            return Ok(BlockResult {
                block_idx: group_idx,
                disc_data,
                block_data: Bytes::new(),
                meta: BlockMetaWIA::default(),
            });
        }

        let mut meta = BlockMetaWIA {
            is_compressed: false,
            data_size: uncompressed_size,
            rvz_packed_size: 0,
            data_hash: 0,
        };
        if is_rvz {
            if let Some(packed_data) = self.try_rvz_pack(group_data.as_ref(), &info) {
                meta.data_size =
                    (hash_exception_data.len() as u32).align_up(4) + packed_data.len() as u32;
                meta.rvz_packed_size = packed_data.len() as u32;
                group_data = packed_data;
            }
        }

        // Compress group
        if self.compressor.kind != Compression::None {
            // Compressed data has no alignment between hash exceptions and data or at the end
            let mut buf = BytesMut::with_capacity(hash_exception_data.len() + group_data.len());
            buf.put_slice(hash_exception_data.as_ref());
            buf.put_slice(group_data.as_ref());
            if !is_rvz {
                // WIA cannot fall back to an uncompressed group. At the maximum
                // exception count, exceptions can be larger than the hash data
                // removed from the source, so bound the actual encoded input.
                let bound = compress_bound(self.compressor.kind, buf.len());
                if self.compressor.buffer.capacity() < bound {
                    self.compressor.buffer = Vec::with_capacity(bound);
                }
            }
            if self
                .compressor
                .compress(buf.as_ref())
                .map_err(|e| io::Error::other(format!("Failed to compress group: {}", e)))?
            {
                let compressed_size = self.compressor.buffer.len() as u32;
                // For WIA, we must always store compressed data.
                // For RVZ, only store compressed data if it's smaller than uncompressed.
                if !is_rvz || compressed_size.align_up(4) < meta.data_size {
                    // Align resulting block data to 4 bytes
                    let mut buf = BytesMut::zeroed(compressed_size.align_up(4) as usize);
                    buf[..compressed_size as usize].copy_from_slice(&self.compressor.buffer);
                    meta.is_compressed = true;
                    // Data size does not include end alignment
                    meta.data_size = compressed_size;
                    meta.data_hash = xxh64_hash(buf.as_ref());
                    return Ok(BlockResult {
                        block_idx: group_idx,
                        disc_data,
                        block_data: buf.freeze(),
                        meta,
                    });
                }
            } else if !is_rvz {
                return Err(io::Error::other(format!(
                    "Failed to compress group {}: len {}, capacity {}",
                    group_idx,
                    self.compressor.buffer.len(),
                    self.compressor.buffer.capacity()
                )));
            }
        }

        // Store uncompressed group, aligned to 4 bytes after hash exceptions and at the end
        let mut buf = BytesMut::zeroed(meta.data_size.align_up(4) as usize);
        buf[..hash_exception_data.len()].copy_from_slice(hash_exception_data.as_ref());
        let offset = (hash_exception_data.len() as u32).align_up(4) as usize;
        buf[offset..offset + group_data.len()].copy_from_slice(group_data.as_ref());
        meta.data_hash = xxh64_hash(buf.as_ref());
        Ok(BlockResult { block_idx: group_idx, disc_data, block_data: buf.freeze(), meta })
    }
}

// Store both hash differences and nonzero padding. A 32-byte padding area needs
// two overlapping 20-byte exceptions; 52 entries cover an entire sector without
// crossing its hash area, matching the maximum supported by Dolphin readers.
fn append_hash_exceptions(
    output: &mut BytesMut,
    sector: u16,
    original: &[u8; HASHES_SIZE],
    reconstructed: &[u8; HASHES_SIZE],
) -> u16 {
    let offsets = (0..0x280)
        .step_by(20)
        .chain((0x280..0x320).step_by(20))
        .chain([0x320, 0x32c])
        .chain((0x340..0x3e0).step_by(20))
        .chain([0x3e0, 0x3ec]);
    let mut count = 0;
    for offset in offsets {
        if original[offset..offset + 20] != reconstructed[offset..offset + 20] {
            output.put_u16(sector * HASHES_SIZE as u16 + offset as u16);
            output.put_slice(&original[offset..offset + 20]);
            count += 1;
        }
    }
    count
}

impl BlockProcessorWIA {
    fn hash_exceptions(
        &mut self,
        disc_data: &[u8],
        info: &GroupInfo,
        chunk_size: usize,
    ) -> io::Result<Bytes> {
        let mut output = BytesMut::new();
        // Even a short final chunk must contain every declared exception list.
        for list in 0..chunk_size.div_ceil(0x200000) {
            let count_offset = output.len();
            output.put_u16(0);
            let mut count = 0u16;
            let start = list * 64;
            let end = (start + 64).min(info.num_sectors as usize);
            for local_sector in start..end {
                let abs_sector = info.sector + local_sector as u32;
                let partition_sector =
                    info.section_offset / SECTOR_DATA_SIZE as u64 + local_sector as u64;
                let expected_group_start = abs_sector - (partition_sector % 64) as u32;
                // A short final group must not reuse its hashes in a following
                // partition whose start falls within the same absolute 64 sectors.
                if self
                    .hash_group
                    .as_ref()
                    .is_none_or(|(start_sector, _)| *start_sector != expected_group_start)
                {
                    // H1 and H2 depend on the entire partition-aligned 64-sector
                    // group, including sectors outside small RVZ chunks. Ignore
                    // existing H0 hashes and zero-fill beyond the partition end.
                    let (group, _) = self.inner.load_sector_group(abs_sector, true)?;
                    let hashes = group.group_hashes.clone().ok_or_else(|| {
                        io::Error::other("Missing reconstructed Wii partition hashes")
                    })?;
                    self.hash_group = Some((group.start_sector, hashes));
                }
                let (group_start, hashes) = self.hash_group.as_ref().unwrap();
                let (h0, h1, h2) = hashes.hashes_for_sector((abs_sector - *group_start) as usize);
                let mut reconstructed = [0u8; HASHES_SIZE];
                reconstructed[..0x26c].copy_from_slice(h0.as_bytes());
                reconstructed[0x280..0x320].copy_from_slice(h1.as_bytes());
                reconstructed[0x340..0x3e0].copy_from_slice(h2.as_bytes());
                let source = array_ref![disc_data, local_sector * SECTOR_SIZE, HASHES_SIZE];
                let mut original = [0u8; HASHES_SIZE];
                if let Some(key) = info.partition_key {
                    aes_cbc_decrypt_b2b(&key, &[0u8; 16], source, &mut original);
                } else {
                    original.copy_from_slice(source);
                }
                // Small RVZ chunks use chunk-local offsets; for chunks above
                // 2 MiB, offsets restart for each exception list.
                count += append_hash_exceptions(
                    &mut output,
                    (local_sector - start) as u16,
                    &original,
                    &reconstructed,
                );
            }
            output[count_offset..count_offset + 2].copy_from_slice(&count.to_be_bytes());
        }
        Ok(output.freeze())
    }

    #[instrument(name = "BlockProcessorWIA::try_rvz_pack", skip_all)]
    fn try_rvz_pack(&mut self, data: &[u8], info: &GroupInfo) -> Option<Bytes> {
        let Some(junk_info) = self
            .junk_info
            .iter()
            .find(|r| info.sector >= r.start_sector && info.sector < r.end_sector)
        else {
            warn!("No junk info found for sector {}", info.sector);
            return None;
        };

        let mut junk_areas = vec![];
        let mut lfg_buf = [0u8; SECTOR_SIZE];
        let mut lfg_sector = u32::MAX;
        let mut offset = info.section_offset;
        let mut data_offset = 0;
        while data_offset < data.len() {
            let sector = (offset / SECTOR_SIZE as u64) as u32;
            let sector_offset = (offset % SECTOR_SIZE as u64) as usize;

            // Initialize LFG for each sector
            if sector != lfg_sector {
                self.lfg.init_with_seed(
                    junk_info.disc_id,
                    junk_info.disc_num,
                    sector as u64 * SECTOR_SIZE as u64,
                );
                self.lfg.fill(&mut lfg_buf);
                lfg_sector = sector;
            }

            // Skip any zeroes
            let zeroes = data[data_offset..].iter().take_while(|&&b| b == 0).count();
            if zeroes > 0 {
                // ...only if they're not LFG zeroes
                let lfg_zeroes = lfg_buf[sector_offset..].iter().take_while(|&&b| b == 0).count();
                if zeroes > lfg_zeroes {
                    // When we have a lot of zeroes, we can pack them as junk data.
                    // We only do this if we're _not_ compressing the data, as the compression
                    // will likely handle this better.
                    if self.compressor.kind == Compression::None && zeroes > SEED_SIZE_BYTES + 4 {
                        debug!("Packing {} zero bytes in group {}", zeroes, info.index);
                        junk_areas.push((data_offset, JunkSeed::Zeroed, zeroes));
                    }

                    offset += zeroes as u64;
                    data_offset += zeroes;
                    continue;
                }
            }

            // Check for junk data
            let len = (SECTOR_SIZE - sector_offset).min(data.len() - data_offset);
            let sector_end = offset + len as u64;
            let num_match = data[data_offset..data_offset + len]
                .iter()
                .zip(&lfg_buf[sector_offset..sector_offset + len])
                .take_while(|(a, b)| a == b)
                .count();
            if num_match > SEED_SIZE_BYTES + 4 {
                debug!("Matched {} junk bytes at offset {:#X}", num_match, offset);
                junk_areas.push((data_offset, JunkSeed::Sector(sector), num_match));
                offset += num_match as u64;
                data_offset += num_match;
            } else {
                // The disc-ID-derived seed didn't match (e.g. the game ID was modified by a
                // patch after the junk was generated). Try to recover the seed from the data
                // itself, like Dolphin does.
                let mut seed = [0u32; SEED_SIZE];
                let num_match = self.lfg.recover_seed(
                    &data[data_offset..data_offset + len],
                    sector_offset,
                    &mut seed,
                );
                if num_match > SEED_SIZE_BYTES + 4 {
                    debug!("Recovered junk seed for {} bytes at offset {:#X}", num_match, offset);
                    junk_areas.push((data_offset, JunkSeed::Recovered(seed), num_match));
                    offset += num_match as u64;
                    data_offset += num_match;
                }
            }

            if offset < sector_end {
                // Jump to the end of the next file, if any exist in this sector
                // Junk data may start after the end of the file
                let next_offset = junk_info
                    .file_ends
                    .range(offset + 1..sector_end)
                    .next()
                    .cloned()
                    // Otherwise, jump to the next sector
                    .unwrap_or(sector_end);

                let skip = (next_offset - offset) as usize;
                offset = next_offset;
                data_offset += skip;
            }
        }

        fn write_raw_data(
            data: &[u8],
            out: &mut [u8],
            offset: usize,
            len: usize,
            data_offset: usize,
        ) {
            *array_ref_mut![out, offset, 4] = (len as u32).to_be_bytes();
            out[offset + 4..offset + 4 + len]
                .copy_from_slice(&data[data_offset..data_offset + len]);
        }

        fn write_junk_data(
            out: &mut [u8],
            offset: usize,
            len: usize,
            seed: &JunkSeed,
            junk_info: &JunkInfo,
        ) {
            let mut seed_out = [0u32; SEED_SIZE];
            match seed {
                // Zeroed data is packed as an all-zeroes LFG seed
                JunkSeed::Zeroed => {}
                JunkSeed::Sector(sector) => {
                    LaggedFibonacci::generate_seed_be(
                        &mut seed_out,
                        junk_info.disc_id,
                        junk_info.disc_num,
                        *sector,
                    );
                }
                JunkSeed::Recovered(seed) => {
                    for (out, x) in seed_out.iter_mut().zip(seed) {
                        *out = x.to_be();
                    }
                }
            }
            *array_ref_mut![out, offset, 4] = (len as u32 | COMPRESSED_BIT).to_be_bytes();
            array_ref_mut![out, offset + 4, SEED_SIZE_BYTES].copy_from_slice(seed_out.as_bytes());
        }

        if !junk_areas.is_empty() {
            let mut packed_data_len = 0;
            let mut last_data_offset = 0;
            for &(data_offset, _, num_match) in &junk_areas {
                if data_offset > last_data_offset {
                    packed_data_len += 4 + data_offset - last_data_offset;
                }
                packed_data_len += 4 + SEED_SIZE_BYTES;
                last_data_offset = data_offset + num_match;
            }
            if last_data_offset < data.len() {
                packed_data_len += 4 + data.len() - last_data_offset;
            }
            let mut packed_data = BytesMut::zeroed(packed_data_len);
            let mut packed_data_offset = 0;
            last_data_offset = 0;
            for (data_offset, seed, len) in &junk_areas {
                let (data_offset, len) = (*data_offset, *len);
                if data_offset > last_data_offset {
                    let len = data_offset - last_data_offset;
                    write_raw_data(
                        data,
                        packed_data.as_mut(),
                        packed_data_offset,
                        len,
                        last_data_offset,
                    );
                    packed_data_offset += 4 + len;
                }
                write_junk_data(packed_data.as_mut(), packed_data_offset, len, seed, junk_info);
                packed_data_offset += 4 + SEED_SIZE_BYTES;
                last_data_offset = data_offset + len;
            }
            if last_data_offset < data.len() {
                let len = data.len() - last_data_offset;
                write_raw_data(
                    data,
                    packed_data.as_mut(),
                    packed_data_offset,
                    len,
                    last_data_offset,
                );
                packed_data_offset += 4 + len;
                last_data_offset += len;
            }
            assert_eq!(packed_data_offset, packed_data_len);
            assert_eq!(last_data_offset, data.len());
            let packed_data = packed_data.freeze();

            // let mut out = BytesMut::zeroed(data.len());
            // rvz_unpack(&mut packed_data.clone(), out.as_mut(), info).unwrap();
            // if out.as_ref() != data {
            //     panic!("Decompressed data mismatch in group {}", info.index);
            // }

            return Some(packed_data);
        }

        None
    }
}

#[derive(Clone)]
pub struct DiscWriterWIA {
    inner: DiscReader,
    header: WIAFileHeader,
    disc: WIADisc,
    partitions: Arc<[WIAPartition]>,
    raw_data: Arc<[WIARawData]>,
    group_count: u32,
    data_start: u32,
    is_rvz: bool,
    compression: Compression,
    initial_header_data: Bytes, // TODO remove
    junk_info: Arc<[JunkInfo]>,
}

#[inline]
fn partition_offset_to_raw(partition_offset: u64) -> u64 {
    (partition_offset / SECTOR_DATA_SIZE as u64) * SECTOR_SIZE as u64
}

pub const RVZ_DEFAULT_CHUNK_SIZE: u32 = 0x20000; // 128 KiB
pub const WIA_DEFAULT_CHUNK_SIZE: u32 = 0x200000; // 2 MiB

// Level 0 will be converted to the default level in [`Compression::validate_level`]
pub const RVZ_DEFAULT_COMPRESSION: Compression = Compression::Zstandard(0);
pub const WIA_DEFAULT_COMPRESSION: Compression = Compression::Lzma(0);

impl DiscWriterWIA {
    pub fn new(inner: DiscReader, options: &FormatOptions) -> Result<Box<dyn DiscWriter>> {
        let is_rvz = options.format == Format::Rvz;
        let chunk_size = options.block_size;

        let disc_header = inner.header();
        let disc_size = inner.disc_size();

        let mut num_partitions = 0;
        let mut num_raw_data = 1;
        let partition_info = inner.partitions();
        for partition in partition_info {
            if !partition.has_hashes {
                continue;
            }
            num_partitions += 1;
            num_raw_data += 2;
        }
        // println!("Num partitions: {}", num_partitions);
        // println!("Num raw data: {}", num_raw_data);

        // Write header
        let header = WIAFileHeader {
            magic: if is_rvz { RVZ_MAGIC } else { WIA_MAGIC },
            version: if is_rvz { RVZ_VERSION } else { WIA_VERSION }.into(),
            version_compatible: if is_rvz {
                RVZ_VERSION_WRITE_COMPATIBLE
            } else {
                WIA_VERSION_WRITE_COMPATIBLE
            }
            .into(),
            disc_size: (size_of::<WIADisc>() as u32).into(),
            disc_hash: Default::default(),
            iso_file_size: disc_size.into(),
            wia_file_size: Default::default(),
            file_head_hash: Default::default(),
        };
        let mut header_data = BytesMut::new();
        header_data.put_slice(header.as_bytes());

        let (compression, level) = match compression_to_wia(options.compression) {
            Some(v) => v,
            None => {
                return Err(Error::Other(format!(
                    "Unsupported compression for WIA/RVZ: {}",
                    options.compression
                )));
            }
        };
        let compr_data = compr_data(options.compression).context("Building compression data")?;
        let mut disc = WIADisc {
            disc_type: if disc_header.is_wii() { DiscKind::Wii } else { DiscKind::GameCube }.into(),
            compression: compression.into(),
            compression_level: level.into(),
            chunk_size: chunk_size.into(),
            disc_head: *array_ref![disc_header.as_bytes(), 0, DISC_HEAD_SIZE],
            num_partitions: num_partitions.into(),
            partition_type_size: (size_of::<WIAPartition>() as u32).into(),
            partition_offset: Default::default(),
            partition_hash: Default::default(),
            num_raw_data: num_raw_data.into(),
            raw_data_offset: Default::default(),
            raw_data_size: Default::default(),
            num_groups: Default::default(),
            group_offset: Default::default(),
            group_size: Default::default(),
            compr_data_len: compr_data.len() as u8,
            compr_data: Default::default(),
        };
        disc.compr_data[..compr_data.len()].copy_from_slice(compr_data.as_ref());
        disc.validate(is_rvz)?;
        header_data.put_slice(disc.as_bytes());

        let nkit_header = NKitHeader {
            version: 2,
            size: Some(disc_size),
            crc32: Some(Default::default()),
            md5: Some(Default::default()),
            sha1: Some(Default::default()),
            xxh64: Some(Default::default()),
            junk_bits: None,
            encrypted: false,
        };
        let mut w = header_data.writer();
        nkit_header.write_to(&mut w).context("Writing NKit header")?;
        let mut header_data = w.into_inner();

        let mut partitions = <[WIAPartition]>::new_box_zeroed_with_elems(num_partitions as usize)?;
        let mut raw_data = <[WIARawData]>::new_box_zeroed_with_elems(num_raw_data as usize)?;
        raw_data[0].raw_data_offset = (DISC_HEAD_SIZE as u64).into();

        let mut raw_data_idx = 0;
        let mut group_idx = 0;
        for (partition, wia_partition) in
            partition_info.iter().filter(|p| p.has_hashes).zip(partitions.iter_mut())
        {
            let partition_start = partition.start_sector as u64 * SECTOR_SIZE as u64;
            let partition_data_start = partition.data_start_sector as u64 * SECTOR_SIZE as u64;
            let partition_end = partition.data_end_sector as u64 * SECTOR_SIZE as u64;

            let boot_header = partition.boot_header();
            let management_data_end =
                partition_offset_to_raw(boot_header.fst_offset(true) + boot_header.fst_size(true))
                    // Align to 2 MiB
                    .align_up(0x200000);
            let management_end_sector = ((partition_data_start + management_data_end)
                .min(partition_end)
                / SECTOR_SIZE as u64) as u32;

            {
                let cur_raw_data = &mut raw_data[raw_data_idx];
                let raw_data_size = partition_start - cur_raw_data.raw_data_offset.get();
                let raw_data_groups = raw_data_size.div_ceil(chunk_size as u64) as u32;
                cur_raw_data.raw_data_size = raw_data_size.into();
                cur_raw_data.group_index = group_idx.into();
                cur_raw_data.num_groups = raw_data_groups.into();
                group_idx += raw_data_groups;
                raw_data_idx += 1;
            }

            {
                let cur_raw_data = &mut raw_data[raw_data_idx];
                let raw_data_size = partition_data_start - partition_start;
                let raw_data_groups = raw_data_size.div_ceil(chunk_size as u64) as u32;
                cur_raw_data.raw_data_offset = partition_start.into();
                cur_raw_data.raw_data_size = raw_data_size.into();
                cur_raw_data.group_index = group_idx.into();
                cur_raw_data.num_groups = raw_data_groups.into();
                group_idx += raw_data_groups;
                raw_data_idx += 1;
            }

            wia_partition.partition_key = partition.key;

            let management_num_sectors = management_end_sector - partition.data_start_sector;
            let management_num_groups = (management_num_sectors as u64 * SECTOR_SIZE as u64)
                .div_ceil(chunk_size as u64) as u32;
            wia_partition.partition_data[0] = WIAPartitionData {
                first_sector: partition.data_start_sector.into(),
                num_sectors: management_num_sectors.into(),
                group_index: group_idx.into(),
                num_groups: management_num_groups.into(),
            };
            group_idx += management_num_groups;

            let data_num_sectors = partition.data_end_sector - management_end_sector;
            let data_num_groups =
                (data_num_sectors as u64 * SECTOR_SIZE as u64).div_ceil(chunk_size as u64) as u32;
            wia_partition.partition_data[1] = WIAPartitionData {
                first_sector: management_end_sector.into(),
                num_sectors: data_num_sectors.into(),
                group_index: group_idx.into(),
                num_groups: data_num_groups.into(),
            };
            group_idx += data_num_groups;

            let next_raw_data = &mut raw_data[raw_data_idx];
            next_raw_data.raw_data_offset = partition_end.into();
        }
        disc.partition_hash = sha1_hash(partitions.as_bytes());

        {
            // Remaining raw data
            let cur_raw_data = &mut raw_data[raw_data_idx];
            let raw_data_size = disc_size - cur_raw_data.raw_data_offset.get();
            let raw_data_groups = raw_data_size.div_ceil(chunk_size as u64) as u32;
            cur_raw_data.raw_data_size = raw_data_size.into();
            cur_raw_data.group_index = group_idx.into();
            cur_raw_data.num_groups = raw_data_groups.into();
            group_idx += raw_data_groups;
        }

        disc.num_groups = group_idx.into();
        let raw_data_size = size_of::<WIARawData>() as u32 * num_raw_data;
        let group_size =
            if is_rvz { size_of::<RVZGroup>() } else { size_of::<WIAGroup>() } as u32 * group_idx;

        header_data.put_slice(partitions.as_bytes());
        header_data.put_bytes(0, raw_data_size as usize);
        header_data.put_bytes(0, group_size as usize);
        // Group data alignment
        let rem = header_data.len() % 4;
        if rem != 0 {
            header_data.put_bytes(0, 4 - rem);
        }

        // println!("Header: {:?}", header);
        // println!("Disc: {:?}", disc);
        // println!("Partitions: {:?}", partitions);
        // println!("Raw data: {:?}", raw_data);

        // Calculate junk info
        let mut junk_info = Vec::<JunkInfo>::with_capacity(partitions.len() + 1);
        // Add partitions first, taking precedence over the raw disc
        for partition in inner.partitions() {
            junk_info.push(JunkInfo::from_fst(
                partition.data_start_sector,
                partition.data_end_sector,
                partition.disc_header(),
                Some(partition.boot_header()),
                partition.fst(),
            ));
        }
        junk_info.push(JunkInfo::from_fst(
            0,
            disc_size.div_ceil(SECTOR_SIZE as u64) as u32,
            disc_header,
            inner.boot_header(),
            inner.fst(),
        ));

        let data_start = header_data.len() as u32;

        Ok(Box::new(Self {
            inner,
            header,
            disc,
            partitions: Arc::from(partitions),
            raw_data: Arc::from(raw_data),
            group_count: group_idx,
            data_start,
            is_rvz,
            compression: options.compression,
            initial_header_data: header_data.freeze(),
            junk_info: Arc::from(junk_info),
        }))
    }
}

impl DiscWriter for DiscWriterWIA {
    fn process(
        &self,
        data_callback: &mut DataCallback,
        options: &ProcessOptions,
    ) -> Result<DiscFinalization> {
        let disc_size = self.inner.disc_size();
        data_callback(self.initial_header_data.clone(), 0, disc_size)
            .context("Failed to write WIA/RVZ header")?;

        let chunk_size = self.disc.chunk_size.get();
        let compressor_buf_size = if self.is_rvz {
            // For RVZ, if a group's compressed size is larger than uncompressed, we discard it.
            // This means we can just allocate a buffer for the chunk size.
            chunk_size as usize
        } else {
            // For WIA, we can't mark groups as uncompressed, so we need to compress them all.
            // This means our compression buffer needs to account for worst-case compression.
            compress_bound(self.compression, chunk_size as usize)
        };
        let mut compressor = Compressor::new(self.compression, compressor_buf_size);

        let digest = DigestManager::new(options);
        let mut input_position = 0;
        let mut file_position = self.data_start as u64;
        let mut groups = <[RVZGroup]>::new_box_zeroed_with_elems(self.group_count as usize)?;
        let mut group_hashes = HashMap::<u64, u32>::new();
        let mut reuse_size = 0;
        par_process(
            BlockProcessorWIA {
                inner: self.inner.clone(),
                header: self.header.clone(),
                disc: self.disc.clone(),
                partitions: self.partitions.clone(),
                raw_data: self.raw_data.clone(),
                compressor: compressor.clone(),
                lfg: LaggedFibonacci::default(),
                junk_info: self.junk_info.clone(),
                hash_group: None,
            },
            self.group_count,
            #[cfg(feature = "threading")]
            options.processor_threads,
            |group| -> Result<()> {
                // Empty and reused groups still need a progress/cancellation
                // checkpoint even when no bytes will be written for this group.
                data_callback(Bytes::new(), input_position, disc_size)
                    .context("Checking WIA/RVZ progress")?;
                // Update hashers
                input_position += group.disc_data.len() as u64;
                digest.send(group.disc_data);

                let group_idx = group.block_idx;
                if file_position % 4 != 0 {
                    return Err(Error::Other("File position not aligned to 4".to_string()));
                }
                let data_offset = (file_position / 4) as u32;
                groups[group_idx as usize] = RVZGroup {
                    data_offset: data_offset.into(),
                    data_size_and_flag: (group.meta.data_size
                        | if group.meta.is_compressed { COMPRESSED_BIT } else { 0 })
                    .into(),
                    rvz_packed_size: group.meta.rvz_packed_size.into(),
                };

                // Skip empty group
                if group.meta.data_size == 0 {
                    return Ok(());
                }

                // Reuse group data if possible
                match group_hashes.entry(group.meta.data_hash) {
                    Entry::Occupied(e) => {
                        debug!("Reusing group data offset {} for group {}", e.get(), group_idx);
                        groups[group_idx as usize].data_offset = (*e.get()).into();
                        reuse_size += group.block_data.len();
                        return Ok(());
                    }
                    Entry::Vacant(e) => {
                        e.insert(data_offset);
                    }
                }

                // Write group data
                if group.block_data.len() % 4 != 0 {
                    return Err(Error::Other("Group data size not aligned to 4".to_string()));
                }
                file_position += group.block_data.len() as u64;
                data_callback(group.block_data, input_position, disc_size)
                    .with_context(|| format!("Failed to write group {group_idx}"))?;
                Ok(())
            },
        )?;
        debug!("Saved {} bytes with group data reuse", reuse_size);

        data_callback(Bytes::new(), input_position, disc_size)
            .context("Finalizing WIA/RVZ output")?;

        // Collect hash results
        let digest_results = digest.finish();
        let mut nkit_header = NKitHeader {
            version: 2,
            size: Some(disc_size),
            crc32: None,
            md5: None,
            sha1: None,
            xxh64: None,
            junk_bits: None,
            encrypted: false,
        };
        nkit_header.apply_digests(&digest_results);
        let mut nkit_header_data = Vec::new();
        nkit_header.write_to(&mut nkit_header_data).context("Writing NKit header")?;

        let mut header = self.header.clone();
        let mut disc = self.disc.clone();

        // Compress raw data and groups
        compressor.buffer = Vec::with_capacity(self.data_start as usize);
        if !compressor.compress(self.raw_data.as_bytes()).context("Compressing raw data")? {
            return Err(Error::Other("Failed to compress raw data".to_string()));
        }
        let compressed_raw_data = compressor.buffer.clone();
        // println!(
        //     "Compressed raw data: {} -> {} (max size {})",
        //     self.raw_data.as_bytes().len(),
        //     compressed_raw_data.len(),
        //     self.data_start
        // );
        disc.raw_data_size = (compressed_raw_data.len() as u32).into();
        data_callback(Bytes::new(), input_position, disc_size)
            .context("Finalizing WIA/RVZ raw data")?;

        let groups_data = if self.is_rvz {
            Cow::Borrowed(groups.as_bytes())
        } else {
            let mut groups_buf = Vec::with_capacity(groups.len() * size_of::<WIAGroup>());
            for group in &groups {
                if compressor.kind != Compression::None
                    && !group.is_compressed()
                    && group.data_size() > 0
                {
                    return Err(Error::Other("Uncompressed group in compressed WIA".to_string()));
                }
                if group.rvz_packed_size.get() > 0 {
                    return Err(Error::Other("RVZ packed group in WIA".to_string()));
                }
                groups_buf.extend_from_slice(WIAGroup::from(group).as_bytes());
            }
            Cow::Owned(groups_buf)
        };
        // RVZ can emit a groups table that compresses slightly above the initial
        // header-sized reservation. Ensure the output buffer is large enough for
        // the codec bound before attempting compression.
        let groups_bound = compress_bound(self.compression, groups_data.len());
        if compressor.buffer.capacity() < groups_bound {
            compressor.buffer = Vec::with_capacity(groups_bound);
        }
        if !compressor.compress(groups_data.as_ref()).context("Compressing groups")? {
            return Err(Error::Other("Failed to compress groups".to_string()));
        }
        let compressed_groups = compressor.buffer;
        // println!(
        //     "Compressed groups: {} -> {} (max size {})",
        //     groups_data.len(),
        //     compressed_groups.len(),
        //     self.data_start
        // );
        disc.group_size = (compressed_groups.len() as u32).into();

        // Update header and calculate hashes
        let mut header_offset = size_of::<WIAFileHeader>() as u32
            + size_of::<WIADisc>() as u32
            + nkit_header_data.len() as u32;
        disc.partition_offset = (header_offset as u64).into();
        header_offset += size_of_val(self.partitions.as_ref()) as u32;
        disc.raw_data_offset = (header_offset as u64).into();
        header_offset += compressed_raw_data.len() as u32;
        disc.group_offset = (header_offset as u64).into();
        header_offset += compressed_groups.len() as u32;
        if header_offset > self.data_start {
            return Err(Error::Other("Header offset exceeds max".to_string()));
        }
        header.disc_hash = sha1_hash(disc.as_bytes());
        header.wia_file_size = file_position.into();
        let header_bytes = header.as_bytes();
        header.file_head_hash =
            sha1_hash(&header_bytes[..size_of::<WIAFileHeader>() - size_of::<HashBytes>()]);

        let mut header_data = BytesMut::with_capacity(header_offset as usize);
        header_data.put_slice(header.as_bytes());
        header_data.put_slice(disc.as_bytes());
        header_data.put_slice(&nkit_header_data);
        header_data.put_slice(self.partitions.as_bytes());
        header_data.put_slice(&compressed_raw_data);
        header_data.put_slice(&compressed_groups);
        if header_data.len() as u32 != header_offset {
            return Err(Error::Other("Header offset mismatch".to_string()));
        }

        let mut finalization =
            DiscFinalization { header: header_data.freeze(), ..Default::default() };
        finalization.apply_digests(&digest_results);
        Ok(finalization)
    }

    fn progress_bound(&self) -> u64 { self.inner.disc_size() }

    fn weight(&self) -> DiscWriterWeight {
        if self.disc.compression() == WIACompression::None {
            DiscWriterWeight::Medium
        } else {
            DiscWriterWeight::Heavy
        }
    }
}

fn compression_to_wia(compression: Compression) -> Option<(WIACompression, i32)> {
    match compression {
        Compression::None => Some((WIACompression::None, 0)),
        Compression::Bzip2(level) => Some((WIACompression::Bzip2, level as i32)),
        Compression::Lzma(level) => Some((WIACompression::Lzma, level as i32)),
        Compression::Lzma2(level) => Some((WIACompression::Lzma2, level as i32)),
        Compression::Zstandard(level) => Some((WIACompression::Zstandard, level as i32)),
        _ => None,
    }
}

fn compr_data(compression: Compression) -> io::Result<Box<[u8]>> {
    match compression {
        #[cfg(feature = "compress-lzma")]
        Compression::Lzma(level) => {
            Ok(Box::new(crate::util::compress::lzma_api::lzma_props_encode_preset(level as u32)?))
        }
        #[cfg(feature = "compress-lzma")]
        Compression::Lzma2(level) => {
            Ok(Box::new(crate::util::compress::lzma_api::lzma2_props_encode_preset(level as u32)?))
        }
        _ => Ok(Box::default()),
    }
}

fn compress_bound(compression: Compression, size: usize) -> usize {
    match compression {
        Compression::None => size,
        Compression::Bzip2(_) => {
            // 1.25 * size
            size.div_ceil(4) + size
        }
        Compression::Lzma(_) => {
            // 1.1 * size + 64 KiB
            size.div_ceil(10) + size + 64000
        }
        Compression::Lzma2(_) => {
            // 1.001 * size + 1 KiB
            size.div_ceil(1000) + size + 1000
        }
        #[cfg(feature = "compress-zstd")]
        Compression::Zstandard(_) => crate::util::compress::zstd_api::compress_bound(size),
        _ => unimplemented!("CompressionKind::compress_bound {:?}", compression),
    }
}

#[cfg(test)]
mod hash_exception_tests {
    use super::*;

    fn restore(encoded: &[u8], sector: u16, hash_data: &mut [u8; HASHES_SIZE]) {
        for exception in encoded.chunks_exact(22) {
            let offset =
                u16::from_be_bytes([exception[0], exception[1]]) - sector * HASHES_SIZE as u16;
            hash_data[offset as usize..offset as usize + 20].copy_from_slice(&exception[2..]);
        }
    }

    #[test]
    fn exceptions_preserve_each_hash_and_padding_byte() {
        for changed_offset in 0..HASHES_SIZE {
            let reconstructed = [0u8; HASHES_SIZE];
            let mut original = reconstructed;
            original[changed_offset] = 0xa5;
            let mut encoded = BytesMut::new();
            let count = append_hash_exceptions(&mut encoded, 0, &original, &reconstructed);
            assert!((1..=2).contains(&count));
            assert_eq!(encoded.len(), count as usize * 22);
            let mut restored = reconstructed;
            restore(&encoded, 0, &mut restored);
            assert_eq!(restored, original, "lost byte at {changed_offset:#x}");
        }
    }

    #[test]
    fn maximum_exceptions_cover_last_sector_including_zero_hashes() {
        let reconstructed = [0xa5u8; HASHES_SIZE];
        let original = [0u8; HASHES_SIZE];
        let mut encoded = BytesMut::new();
        let count = append_hash_exceptions(&mut encoded, 63, &original, &reconstructed);
        assert_eq!(count, 52);
        assert_eq!(encoded.len(), 52 * 22);
        let mut restored = reconstructed;
        restore(&encoded, 63, &mut restored);
        assert_eq!(restored, original);
    }

    #[test]
    fn unchanged_hash_area_needs_no_exceptions() {
        let original = [0xa5u8; HASHES_SIZE];
        let mut encoded = BytesMut::new();
        assert_eq!(append_hash_exceptions(&mut encoded, 7, &original, &original), 0);
        assert!(encoded.is_empty());
    }
}
