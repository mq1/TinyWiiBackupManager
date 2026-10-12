//! [`DiscReader`] and associated types.
use std::{
    io::{self, BufRead, Read, Seek},
    path::Path,
    sync::{Arc, Mutex},
};

use dyn_clone::DynClone;
use zerocopy::FromBytes;

use crate::{
    Result,
    common::{Compression, Format, PartitionInfo, PartitionKind},
    disc,
    disc::{
        ApploaderHeader, BB2_OFFSET, BI2_SIZE, BOOT_SIZE, BootHeader, DebugHeader, DiscHeader,
        DolHeader,
        fst::{Fst, Node},
        wii::{ContentMetadata, H3_TABLE_SIZE, REGION_SIZE, Ticket, TmdHeader},
    },
    io::block,
    util::{WindowedReader, array_ref},
};

/// Wii partition encryption mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum PartitionEncryption {
    /// Partition encryption and hashes are rebuilt to match its original state,
    /// if necessary. This is used for converting or verifying a disc image.
    #[default]
    Original,
    /// Partition data will be encrypted if reading a decrypted disc image.
    /// Modifies the disc header to mark partition data as encrypted.
    ForceEncrypted,
    /// Partition data will be decrypted if reading an encrypted disc image.
    /// Modifies the disc header to mark partition data as decrypted.
    ForceDecrypted,
    /// Partition data will be decrypted if reading an encrypted disc image.
    /// Modifies the disc header to mark partition data as decrypted.
    /// Hashes are removed from the partition data.
    ForceDecryptedNoHashes,
}

/// Options for opening a disc image.
#[derive(Default, Debug, Clone)]
pub struct DiscOptions {
    /// Wii: Partition encryption mode. This affects how partition data appears when
    /// reading directly from [`DiscReader`], and can be used to convert between
    /// encrypted and decrypted disc images.
    pub partition_encryption: PartitionEncryption,
    /// Number of threads to use for preloading data as the disc is read. This
    /// is particularly useful when reading the disc image sequentially, as it
    /// can perform decompression and rebuilding in parallel with the main
    /// read thread. The default value of 0 disables preloading.
    #[cfg(feature = "threading")]
    pub preloader_threads: usize,
}

/// Options for opening a partition.
#[derive(Default, Debug, Clone)]
pub struct PartitionOptions {
    /// Wii: Validate data hashes while reading the partition, if available.
    /// This significantly slows down reading.
    pub validate_hashes: bool,
}

/// Trait for reading disc images.
///
/// Disc images are read in blocks, often in the hundred kilobyte to several megabyte range,
/// making the standard [`Read`] and [`Seek`] traits a poor fit for this use case. This trait
/// provides a simplified interface for reading disc images, with a focus on large, random
/// access reads.
///
/// For multithreading support, an implementation must be [`Send`] and [`Clone`].
/// [`Sync`] is _not_ required: the stream will be cloned if used in multiple threads.
///
/// Rather than implement this trait directly, you'll likely use one of the following
/// [`DiscReader`] functions:
/// - [`DiscReader::new`]: to open a disc image from a file path.
/// - [`DiscReader::new_stream`]: when you can provide a [`Box<dyn DiscStream>`].
/// - [`DiscReader::new_from_cloneable_read`]: when you can provide a [`Read`] + [`Seek`] +
///   [`Clone`] stream.
/// - [`DiscReader::new_from_non_cloneable_read`]: when you can provide a [`Read`] + [`Seek`]
///   stream. (Accesses will be synchronized, limiting multithreaded performance.)
pub trait DiscStream: DynClone + Send {
    /// Reads the exact number of bytes required to fill `buf` from the given offset.
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()>;

    /// Returns the length of the stream in bytes.
    fn stream_len(&mut self) -> io::Result<u64>;
}

dyn_clone::clone_trait_object!(DiscStream);

impl<T> DiscStream for T
where T: AsRef<[u8]> + Send + Clone
{
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let data = self.as_ref();
        let len = data.len() as u64;
        let end = offset + buf.len() as u64;
        if offset >= len || end > len {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        buf.copy_from_slice(&data[offset as usize..end as usize]);
        Ok(())
    }

    fn stream_len(&mut self) -> io::Result<u64> { Ok(self.as_ref().len() as u64) }
}

#[derive(Debug, Clone)]
pub(crate) struct CloneableStream<T>(pub T)
where T: Read + Seek + Clone + Send;

impl<T> CloneableStream<T>
where T: Read + Seek + Clone + Send
{
    pub fn new(stream: T) -> Self { Self(stream) }
}

impl<T> DiscStream for CloneableStream<T>
where T: Read + Seek + Clone + Send
{
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        self.0.seek(io::SeekFrom::Start(offset))?;
        self.0.read_exact(buf)
    }

    fn stream_len(&mut self) -> io::Result<u64> { self.0.seek(io::SeekFrom::End(0)) }
}

#[derive(Debug)]
pub(crate) struct NonCloneableStream<T>(pub Arc<Mutex<T>>)
where T: Read + Seek + Send;

impl<T> Clone for NonCloneableStream<T>
where T: Read + Seek + Send
{
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

impl<T> NonCloneableStream<T>
where T: Read + Seek + Send
{
    pub fn new(stream: T) -> Self { Self(Arc::new(Mutex::new(stream))) }

    fn lock(&self) -> io::Result<std::sync::MutexGuard<'_, T>> {
        self.0.lock().map_err(|_| io::Error::other("NonCloneableStream mutex poisoned"))
    }
}

impl<T> DiscStream for NonCloneableStream<T>
where T: Read + Seek + Send
{
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let mut stream = self.lock()?;
        stream.seek(io::SeekFrom::Start(offset))?;
        stream.read_exact(buf)
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        let mut stream = self.lock()?;
        stream.seek(io::SeekFrom::End(0))
    }
}

/// An open disc image and read stream.
///
/// This is the primary entry point for reading disc images.
#[derive(Clone)]
#[repr(transparent)]
pub struct DiscReader(disc::reader::DiscReader);

impl DiscReader {
    /// Opens a disc image from a file path.
    pub fn new<P: AsRef<Path>>(path: P, options: &DiscOptions) -> Result<DiscReader> {
        let io = block::open(path.as_ref())?;
        let inner = disc::reader::DiscReader::new(io, options)?;
        Ok(DiscReader(inner))
    }

    /// Opens a disc image from a [`DiscStream`]. This allows low-overhead, multithreaded
    /// access to disc images stored in memory, archives, or other non-file sources.
    ///
    /// See [`DiscStream`] for more information.
    pub fn new_stream(stream: Box<dyn DiscStream>, options: &DiscOptions) -> Result<DiscReader> {
        let io = block::new(stream)?;
        let inner = disc::reader::DiscReader::new(io, options)?;
        Ok(DiscReader(inner))
    }

    /// Opens a disc image from a [`Read`] + [`Seek`] stream that can be cloned.
    ///
    /// The stream will be cloned for each thread that reads from it, allowing for multithreaded
    /// access (e.g. for preloading blocks during reading or parallel block processing during
    /// conversion).
    pub fn new_from_cloneable_read<R>(stream: R, options: &DiscOptions) -> Result<DiscReader>
    where R: Read + Seek + Clone + Send + 'static {
        Self::new_stream(Box::new(CloneableStream::new(stream)), options)
    }

    /// Opens a disc image from a [`Read`] + [`Seek`] stream that cannot be cloned.
    ///
    /// Multithreaded accesses will be synchronized, which will limit performance (e.g. for
    /// preloading blocks during reading or parallel block processing during conversion).
    pub fn new_from_non_cloneable_read<R>(stream: R, options: &DiscOptions) -> Result<DiscReader>
    where R: Read + Seek + Send + 'static {
        Self::new_stream(Box::new(NonCloneableStream::new(stream)), options)
    }

    /// Detects the format of a disc image from a read stream.
    #[inline]
    pub fn detect<R>(stream: &mut R) -> io::Result<Option<Format>>
    where R: Read + ?Sized {
        block::detect(stream)
    }

    /// The disc's primary header.
    #[inline]
    pub fn header(&self) -> &DiscHeader { self.0.header() }

    /// The Wii disc's region information.
    ///
    /// **GameCube**: This will return `None`.
    #[inline]
    pub fn region(&self) -> Option<&[u8; REGION_SIZE]> { self.0.region() }

    /// Returns extra metadata included in the disc file format, if any.
    #[inline]
    pub fn meta(&self) -> DiscMeta { self.0.meta() }

    /// The disc's size in bytes, or an estimate if not stored by the format.
    #[inline]
    pub fn disc_size(&self) -> u64 { self.0.disc_size() }

    /// A list of Wii partitions on the disc.
    ///
    /// **GameCube**: This will return an empty slice.
    #[inline]
    pub fn partitions(&self) -> &[PartitionInfo] { self.0.partitions() }

    /// Opens a decrypted partition read stream for the specified partition index.
    ///
    /// **GameCube**: `index` must always be 0.
    #[inline]
    pub fn open_partition(
        &self,
        index: usize,
        options: &PartitionOptions,
    ) -> Result<Box<dyn PartitionReader>> {
        self.0.open_partition(index, options)
    }

    /// Opens a decrypted partition read stream for the first partition matching
    /// the specified kind.
    ///
    /// **GameCube**: `kind` must always be [`PartitionKind::Data`].
    #[inline]
    pub fn open_partition_kind(
        &self,
        kind: PartitionKind,
        options: &PartitionOptions,
    ) -> Result<Box<dyn PartitionReader>> {
        self.0.open_partition_kind(kind, options)
    }

    pub(crate) fn into_inner(self) -> disc::reader::DiscReader { self.0 }
}

impl BufRead for DiscReader {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> { self.0.fill_buf() }

    #[inline]
    fn consume(&mut self, amt: usize) { self.0.consume(amt) }
}

impl Read for DiscReader {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.0.read(buf) }
}

impl Seek for DiscReader {
    #[inline]
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> { self.0.seek(pos) }
}

/// Extra metadata about the underlying disc file format.
#[derive(Debug, Clone, Default)]
pub struct DiscMeta {
    /// The disc file format.
    pub format: Format,
    /// The format's compression algorithm.
    pub compression: Compression,
    /// If the format uses blocks, the block size in bytes.
    pub block_size: Option<u32>,
    /// Whether Wii partitions are stored decrypted in the format.
    pub decrypted: bool,
    /// Whether the format omits Wii partition data hashes.
    pub needs_hash_recovery: bool,
    /// Whether the format supports recovering the original disc data losslessly.
    pub lossless: bool,
    /// The original disc's size in bytes, if stored by the format.
    pub disc_size: Option<u64>,
    /// The original disc's CRC32 hash, if stored by the format.
    pub crc32: Option<u32>,
    /// The original disc's MD5 hash, if stored by the format.
    pub md5: Option<[u8; 16]>,
    /// The original disc's SHA-1 hash, if stored by the format.
    pub sha1: Option<[u8; 20]>,
    /// The original disc's XXH64 hash, if stored by the format.
    pub xxh64: Option<u64>,
}

/// An open disc partition.
pub trait PartitionReader: DynClone + BufRead + Seek + Send {
    /// Whether this is a Wii partition. (GameCube otherwise)
    fn is_wii(&self) -> bool;

    /// Reads the partition header and file system table.
    fn meta(&mut self) -> Result<PartitionMeta>;
}

/// A file reader borrowing a [`PartitionReader`].
pub type FileReader<'a> = WindowedReader<&'a mut dyn PartitionReader>;

/// A file reader owning a [`PartitionReader`].
pub type OwnedFileReader = WindowedReader<Box<dyn PartitionReader>>;

impl dyn PartitionReader + '_ {
    /// Seeks the partition stream to the specified file system node
    /// and returns a windowed stream.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```no_run
    /// use std::io::Read;
    ///
    /// use nod::{
    ///     common::PartitionKind,
    ///     read::{DiscOptions, DiscReader, PartitionOptions},
    /// };
    ///
    /// fn main() -> nod::Result<()> {
    ///     let disc = DiscReader::new("path/to/file.iso", &DiscOptions::default())?;
    ///     let mut partition =
    ///         disc.open_partition_kind(PartitionKind::Data, &PartitionOptions::default())?;
    ///     let meta = partition.meta()?;
    ///     let fst = meta.fst()?;
    ///     if let Some((_, node)) = fst.find("/MP3/Worlds.txt") {
    ///         let mut s = String::new();
    ///         partition
    ///             .open_file(node)
    ///             .expect("Failed to open file stream")
    ///             .read_to_string(&mut s)
    ///             .expect("Failed to read file");
    ///         println!("{}", s);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn open_file(&mut self, node: Node) -> io::Result<FileReader<'_>> {
        if !node.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Node is not a file".to_string(),
            ));
        }
        let is_wii = self.is_wii();
        FileReader::new(self, node.offset(is_wii), node.length() as u64)
    }
}

impl dyn PartitionReader {
    /// Consumes the partition instance and returns a windowed stream.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io::Read;
    ///
    /// use nod::{
    ///     common::PartitionKind,
    ///     read::{DiscOptions, DiscReader, OwnedFileReader, PartitionOptions},
    /// };
    ///
    /// fn main() -> nod::Result<()> {
    ///     let disc = DiscReader::new("path/to/file.iso", &DiscOptions::default())?;
    ///     let mut partition =
    ///         disc.open_partition_kind(PartitionKind::Data, &PartitionOptions::default())?;
    ///     let meta = partition.meta()?;
    ///     let fst = meta.fst()?;
    ///     if let Some((_, node)) = fst.find("/disc.tgc") {
    ///         let file: OwnedFileReader = partition
    ///             .into_open_file(node) // Get an OwnedFileStream
    ///             .expect("Failed to open file stream");
    ///         // Open the inner disc image using the owned stream
    ///         let inner_disc = DiscReader::new_from_cloneable_read(file, &DiscOptions::default())
    ///             .expect("Failed to open inner disc");
    ///         // ...
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn into_open_file(self: Box<Self>, node: Node) -> io::Result<OwnedFileReader> {
        if !node.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Node is not a file".to_string(),
            ));
        }
        let is_wii = self.is_wii();
        OwnedFileReader::new(self, node.offset(is_wii), node.length() as u64)
    }
}

dyn_clone::clone_trait_object!(PartitionReader);

/// Extra disc partition data. (DOL, FST, etc.)
#[derive(Clone, Debug)]
pub struct PartitionMeta {
    /// Disc and boot header (boot.bin)
    pub raw_boot: Arc<[u8; BOOT_SIZE]>,
    /// Debug and region information (bi2.bin)
    pub raw_bi2: Arc<[u8; BI2_SIZE]>,
    /// Apploader (apploader.bin)
    pub raw_apploader: Arc<[u8]>,
    /// Main binary (main.dol)
    pub raw_dol: Arc<[u8]>,
    /// File system table (fst.bin)
    pub raw_fst: Arc<[u8]>,
    /// Ticket (ticket.bin, Wii only)
    pub raw_ticket: Option<Arc<[u8]>>,
    /// TMD (tmd.bin, Wii only)
    pub raw_tmd: Option<Arc<[u8]>>,
    /// Certificate chain (cert.bin, Wii only)
    pub raw_cert_chain: Option<Arc<[u8]>>,
    /// H3 hash table (h3.bin, Wii only)
    pub raw_h3_table: Option<Arc<[u8; H3_TABLE_SIZE]>>,
}

impl PartitionMeta {
    /// A view into the disc header.
    #[inline]
    pub fn disc_header(&self) -> &DiscHeader {
        DiscHeader::ref_from_bytes(array_ref![self.raw_boot, 0, size_of::<DiscHeader>()])
            .expect("Invalid disc header alignment")
    }

    /// A view into the debug header.
    #[inline]
    pub fn debug_header(&self) -> &DebugHeader {
        DebugHeader::ref_from_bytes(array_ref![
            self.raw_boot,
            size_of::<DiscHeader>(),
            size_of::<DebugHeader>()
        ])
        .expect("Invalid debug header alignment")
    }

    /// A view into the boot header.
    #[inline]
    pub fn boot_header(&self) -> &BootHeader {
        BootHeader::ref_from_bytes(array_ref![self.raw_boot, BB2_OFFSET, size_of::<BootHeader>()])
            .expect("Invalid boot header alignment")
    }

    /// A view into the apploader header.
    #[inline]
    pub fn apploader_header(&self) -> &ApploaderHeader {
        ApploaderHeader::ref_from_prefix(&self.raw_apploader)
            .expect("Invalid apploader alignment")
            .0
    }

    /// A view into the file system table (FST).
    #[inline]
    pub fn fst(&self) -> Result<Fst<'_>, &'static str> { Fst::new(&self.raw_fst) }

    /// A view into the DOL header.
    #[inline]
    pub fn dol_header(&self) -> &DolHeader {
        DolHeader::ref_from_prefix(&self.raw_dol).expect("Invalid DOL alignment").0
    }

    /// A view into the ticket. (Wii only)
    #[inline]
    pub fn ticket(&self) -> Option<&Ticket> {
        let raw_ticket = self.raw_ticket.as_deref()?;
        Some(Ticket::ref_from_bytes(raw_ticket).expect("Invalid ticket alignment"))
    }

    /// A view into the TMD. (Wii only)
    #[inline]
    pub fn tmd_header(&self) -> Option<&TmdHeader> {
        let raw_tmd = self.raw_tmd.as_deref()?;
        Some(TmdHeader::ref_from_prefix(raw_tmd).expect("Invalid TMD alignment").0)
    }

    /// A view into the TMD content metadata. (Wii only)
    #[inline]
    pub fn content_metadata(&self) -> Option<&[ContentMetadata]> {
        let raw_cmd = &self.raw_tmd.as_deref()?[size_of::<TmdHeader>()..];
        Some(<[ContentMetadata]>::ref_from_bytes(raw_cmd).expect("Invalid CMD alignment"))
    }
}
