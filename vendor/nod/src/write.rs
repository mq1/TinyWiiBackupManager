//! [`DiscWriter`] and associated types.

use std::io;

use bytes::Bytes;

use crate::{
    Error, Result,
    common::{Compression, Format},
    disc,
    read::DiscReader,
};

/// Options for writing a disc image.
#[derive(Default, Debug, Clone)]
pub struct FormatOptions {
    /// The disc format to write.
    pub format: Format,
    /// The compression algorithm to use for the output format, if supported.
    ///
    /// If unsure, use [`Format::default_compression`] to get the default compression for the format.
    pub compression: Compression,
    /// Block size to use.
    ///
    /// If unsure, use [`Format::default_block_size`] to get the default block size for the format.
    pub block_size: u32,
}

impl FormatOptions {
    /// Creates options for the specified format.
    /// Uses the default compression and block size for the format.
    #[inline]
    pub fn new(format: Format) -> FormatOptions {
        FormatOptions {
            format,
            compression: format.default_compression(),
            block_size: format.default_block_size(),
        }
    }
}

/// Options for processing a disc image writer.
#[derive(Default, Debug, Clone)]
pub struct ProcessOptions {
    /// If the output format supports multithreaded processing, this sets the number of threads to
    /// use for processing data. This is particularly useful for formats that compress data or
    /// perform other transformations. The default value of 0 disables multithreading.
    #[cfg(feature = "threading")]
    pub processor_threads: usize,
    /// Enables CRC32 checksum calculation for the disc data.
    ///
    /// If the output format supports it, this will be stored in the disc data. (NKit 2 compatible)
    /// If the "threading" feature is enabled, each digest calculation will run on a separate thread,
    /// unaffected by the processor thread count.
    pub digest_crc32: bool,
    /// Enables MD5 checksum calculation for the disc data. (Slow!)
    ///
    /// If the output format supports it, this will be stored in the disc data. (NKit 2 compatible)
    /// If the "threading" feature is enabled, each digest calculation will run on a separate thread,
    /// unaffected by the processor thread count.
    pub digest_md5: bool,
    /// Enables SHA-1 checksum calculation for the disc data.
    ///
    /// If the output format supports it, this will be stored in the disc data. (NKit 2 compatible)
    /// If the "threading" feature is enabled, each digest calculation will run on a separate thread,
    /// unaffected by the processor thread count.
    pub digest_sha1: bool,
    /// Enables XXH64 checksum calculation for the disc data.
    ///
    /// If the output format supports it, this will be stored in the disc data. (NKit 2 compatible)
    /// If the "threading" feature is enabled, each digest calculation will run on a separate thread,
    /// unaffected by the processor thread count.
    pub digest_xxh64: bool,
    /// The level of scrubbing to perform on the disc image.
    ///
    /// This may reduce the size of the output disc image by removing unnecessary data, but will
    /// also prevent reconstruction of the original disc image. Use with caution.
    ///
    /// If unsure, use `ScrubLevel::None`.
    pub scrub: ScrubLevel,
}

/// The level of scrubbing to perform on the disc image.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubLevel {
    /// Do not scrub any data from the disc image.
    #[default]
    None,
    /// Replace the update partition with zeroes to save space.
    ///
    /// NOTE: This is currently implemented only for WBFS and CISO.
    UpdatePartition,
}

/// A callback for writing disc data.
///
/// The callback should write all data to the output stream before returning, or return an error if
/// writing fails. The second and third arguments are the current bytes processed and the total
/// bytes to process, respectively. For most formats, this has no relation to the written disc size,
/// but can be used to display progress.
pub type DataCallback<'a> = dyn FnMut(Bytes, u64, u64) -> io::Result<()> + 'a;

/// A constructed disc writer.
///
/// This is the primary entry point for writing disc images.
#[derive(Clone)]
#[repr(transparent)]
pub struct DiscWriter(Box<dyn disc::writer::DiscWriter>);

impl DiscWriter {
    /// Creates a new disc writer with the specified format options.
    #[inline]
    pub fn new(disc: DiscReader, options: &FormatOptions) -> Result<DiscWriter> {
        let mut options = options.clone();
        options.compression.validate_level()?;
        let mut reader = disc.into_inner();
        reader.reset();
        let inner = match options.format {
            Format::Iso => {
                if options.compression != Compression::None {
                    return Err(Error::Other("ISO/GCM does not support compression".to_string()));
                }
                Box::new(reader)
            }
            Format::Ciso => crate::io::ciso::DiscWriterCISO::new(reader, &options)?,
            #[cfg(feature = "compress-zlib")]
            Format::Gcz => crate::io::gcz::DiscWriterGCZ::new(reader, &options)?,
            Format::Tgc => crate::io::tgc::DiscWriterTGC::new(reader, &options)?,
            Format::Wbfs => crate::io::wbfs::DiscWriterWBFS::new(reader, &options)?,
            Format::Wia | Format::Rvz => crate::io::wia::DiscWriterWIA::new(reader, &options)?,
            format => return Err(Error::Other(format!("Unsupported write format: {format}"))),
        };
        Ok(DiscWriter(inner))
    }

    /// Processes the disc writer to completion, calling the data callback, in order, for each block
    /// of data to write to the output file. The callback should write all data before returning, or
    /// return an error if writing fails.
    ///
    /// See [`DataCallback`] for more information.
    #[inline]
    pub fn process(
        &self,
        mut data_callback: impl FnMut(Bytes, u64, u64) -> io::Result<()>,
        options: &ProcessOptions,
    ) -> Result<DiscFinalization> {
        self.0.process(&mut data_callback, options)
    }

    /// Returns the progress upper bound for the disc writer. For most formats, this has no
    /// relation to the written disc size, but can be used to display progress.
    #[inline]
    pub fn progress_bound(&self) -> u64 { self.0.progress_bound() }

    /// Returns the weight of the disc writer, which can help determine the number of threads to
    /// dedicate for output processing. This may depend on the format's configuration, such as
    /// whether compression is enabled.
    #[inline]
    pub fn weight(&self) -> DiscWriterWeight { self.0.weight() }
}

/// Data returned by the disc writer after processing.
///
/// If header data is provided, the consumer should seek to the beginning of the output stream and
/// write the header data, overwriting any existing data. Otherwise, the output disc will be
/// invalid.
#[derive(Default, Clone)]
pub struct DiscFinalization {
    /// Header data to write to the beginning of the output stream, if any.
    pub header: Bytes,
    /// The calculated CRC32 checksum of the input disc data, if any.
    pub crc32: Option<u32>,
    /// The calculated MD5 hash of the input disc data, if any.
    pub md5: Option<[u8; 16]>,
    /// The calculated SHA-1 hash of the input disc data, if any.
    pub sha1: Option<[u8; 20]>,
    /// The calculated SHA-256 hash of the input disc data, if any.
    pub xxh64: Option<u64>,
}

/// The weight of a disc writer, which can help determine the number of threads to use for
/// processing.
pub enum DiscWriterWeight {
    /// The writer performs little to no processing of the input data, and is mostly I/O bound.
    /// This means that this writer does not benefit from parallelization, and will ignore the
    /// number of threads specified.
    Light,
    /// The writer performs some processing of the input data, and is somewhat CPU bound. This means
    /// that this writer benefits from parallelization, but not as much as a heavy writer.
    Medium,
    /// The writer performs significant processing of the input data, and is mostly CPU bound. This
    /// means that this writer benefits from parallelization.
    Heavy,
}
