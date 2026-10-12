#![allow(clippy::new_ret_no_self)]
#![warn(missing_docs)]
//! Library for reading and writing Nintendo Optical Disc (GameCube and Wii) images.
//!
//! Originally based on the C++ library [nod](https://github.com/AxioDL/nod),
//! but with extended format support and many additional features.
//!
//! Currently supported file formats:
//! - ISO (GCM)
//! - WIA / RVZ
//! - WBFS (+ NKit 2 lossless)
//! - CISO (+ NKit 2 lossless)
//! - NFS (Wii U VC, read-only)
//! - GCZ
//! - TGC
//!
//! # Examples
//!
//! Opening a disc image and reading a file:
//!
//! ```no_run
//! use std::io::Read;
//!
//! use nod::{
//!     common::PartitionKind,
//!     read::{DiscOptions, DiscReader, PartitionOptions},
//! };
//!
//! // Open a disc image and the first data partition.
//! let disc =
//!     DiscReader::new("path/to/file.iso", &DiscOptions::default()).expect("Failed to open disc");
//! let mut partition = disc
//!     .open_partition_kind(PartitionKind::Data, &PartitionOptions::default())
//!     .expect("Failed to open data partition");
//!
//! // Read partition metadata and the file system table.
//! let meta = partition.meta().expect("Failed to read partition metadata");
//! let fst = meta.fst().expect("File system table is invalid");
//!
//! // Find a file by path and read it into a string.
//! if let Some((_, node)) = fst.find("/MP3/Worlds.txt") {
//!     let mut s = String::new();
//!     partition
//!         .open_file(node)
//!         .expect("Failed to open file stream")
//!         .read_to_string(&mut s)
//!         .expect("Failed to read file");
//!     println!("{}", s);
//! }
//! ```
//!
//! Converting a disc image to raw ISO:
//!
//! ```no_run
//! use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
//!
//! let options = DiscOptions {
//!     partition_encryption: PartitionEncryption::Original,
//!     // Use 4 threads to preload data as the disc is read. This can speed up sequential reads,
//!     // especially when the disc image format uses compression.
//!     preloader_threads: 4,
//! };
//! // Open a disc image.
//! let mut disc = DiscReader::new("path/to/file.rvz", &options).expect("Failed to open disc");
//!
//! // Create a new output file.
//! let mut out = std::fs::File::create("output.iso").expect("Failed to create output file");
//! // Read directly from the DiscReader and write to the output file.
//! // NOTE: Any copy method that accepts `Read` and `Write` can be used here,
//! // such as `std::io::copy`. This example utilizes `BufRead` for efficiency,
//! // since `DiscReader` has its own internal buffer.
//! nod::util::buf_copy(&mut disc, &mut out).expect("Failed to write data");
//! ```
//!
//! Converting a disc image to RVZ:
//!
//! ```no_run
//! use std::{
//!     fs::File,
//!     io::{Seek, Write},
//! };
//!
//! use nod::{
//!     common::{Compression, Format},
//!     read::{DiscOptions, DiscReader, PartitionEncryption},
//!     write::{DiscWriter, DiscWriterWeight, FormatOptions, ProcessOptions, ScrubLevel},
//! };
//!
//! let open_options = DiscOptions {
//!     partition_encryption: PartitionEncryption::Original,
//!     // Use 4 threads to preload data as the disc is read. This can speed up sequential reads,
//!     // especially when the disc image format uses compression.
//!     preloader_threads: 4,
//! };
//! // Open a disc image.
//! let disc = DiscReader::new("path/to/file.iso", &open_options).expect("Failed to open disc");
//! // Create a new output file.
//! let mut output_file = File::create("output.rvz").expect("Failed to create output file");
//!
//! let options = FormatOptions {
//!     format: Format::Rvz,
//!     compression: Compression::Zstandard(19),
//!     block_size: Format::Rvz.default_block_size(),
//! };
//! // Create a disc writer with the desired output format.
//! let mut writer = DiscWriter::new(disc, &options).expect("Failed to create writer");
//!
//! // Ideally we'd base this on the actual number of CPUs available.
//! // This is just an example.
//! let num_threads = match writer.weight() {
//!     DiscWriterWeight::Light => 0,
//!     DiscWriterWeight::Medium => 4,
//!     DiscWriterWeight::Heavy => 12,
//! };
//! let process_options = ProcessOptions {
//!     processor_threads: num_threads,
//!     // Enable checksum calculation for the _original_ disc data.
//!     // Digests will be stored in the output file for verification, if supported.
//!     // They will also be returned in the finalization result.
//!     digest_crc32: true,
//!     digest_md5: false, // MD5 is slow, skip it
//!     digest_sha1: true,
//!     digest_xxh64: true,
//!     scrub: ScrubLevel::None,
//! };
//! // Start processing the disc image.
//! let finalization = writer
//!     .process(
//!         |data, _progress, _total| {
//!             output_file.write_all(data.as_ref())?;
//!             // One could display progress here, if desired.
//!             Ok(())
//!         },
//!         &process_options,
//!     )
//!     .expect("Failed to process disc image");
//!
//! // Some disc writers calculate data during processing.
//! // If the finalization returns header data, seek to the beginning of the file and write it.
//! if !finalization.header.is_empty() {
//!     output_file.rewind().expect("Failed to seek");
//!     output_file.write_all(finalization.header.as_ref()).expect("Failed to write header");
//! }
//! output_file.flush().expect("Failed to flush output file");
//!
//! // Display the calculated digests.
//! println!("CRC32: {:08X}", finalization.crc32.unwrap());
//! // ...
//! ```

// [WIP] Disc image building is incomplete and not yet exposed.
pub(crate) mod build;
pub mod common;
pub mod disc;
pub(crate) mod io;
#[cfg(feature = "python")]
mod python;
pub mod read;
pub mod util;
pub mod write;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn nod_rs(m: &Bound<'_, PyModule>) -> PyResult<()> { python::register(m) }

/// Error types for nod.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error for disc format related issues.
    #[error("disc format error: {0}")]
    DiscFormat(String),
    /// A general I/O error.
    #[error("{0}")]
    Io(String, #[source] std::io::Error),
    /// An unknown error.
    #[error("error: {0}")]
    Other(String),
}

impl From<&str> for Error {
    #[inline]
    fn from(s: &str) -> Error { Error::Other(s.to_string()) }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(e: std::io::Error) -> Error { Error::Io("IO Error".to_string(), e) }
}

impl From<String> for Error {
    #[inline]
    fn from(s: String) -> Error { Error::Other(s) }
}

impl From<zerocopy::AllocError> for Error {
    #[inline]
    fn from(_: zerocopy::AllocError) -> Error {
        Error::Io(
            "allocation failed".to_string(),
            std::io::Error::from(std::io::ErrorKind::OutOfMemory),
        )
    }
}

/// Helper result type for [`Error`].
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Helper trait for adding context to errors.
pub trait ErrorContext {
    /// Adds context to an error.
    fn context(self, context: impl Into<String>) -> Error;
}

impl ErrorContext for std::io::Error {
    #[inline]
    fn context(self, context: impl Into<String>) -> Error { Error::Io(context.into(), self) }
}

/// Helper trait for adding context to result errors.
pub trait ResultContext<T> {
    /// Adds context to a result error.
    fn context(self, context: impl Into<String>) -> Result<T>;

    /// Adds context to a result error using a closure.
    fn with_context<F>(self, f: F) -> Result<T>
    where F: FnOnce() -> String;
}

impl<T, E> ResultContext<T> for Result<T, E>
where E: ErrorContext
{
    #[inline]
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.context(context))
    }

    #[inline]
    fn with_context<F>(self, f: F) -> Result<T>
    where F: FnOnce() -> String {
        self.map_err(|e| e.context(f()))
    }
}

pub(crate) trait IoErrorContext {
    fn io_context(self, context: impl Into<String>) -> std::io::Error;
}

impl IoErrorContext for std::io::Error {
    #[inline]
    fn io_context(self, context: impl Into<String>) -> std::io::Error {
        std::io::Error::new(self.kind(), self.context(context))
    }
}

pub(crate) trait IoResultContext<T> {
    fn io_context(self, context: impl Into<String>) -> std::io::Result<T>;

    fn io_with_context<F>(self, f: F) -> std::io::Result<T>
    where F: FnOnce() -> String;
}

impl<T> IoResultContext<T> for std::io::Result<T> {
    #[inline]
    fn io_context(self, context: impl Into<String>) -> std::io::Result<T> {
        self.map_err(|e| e.io_context(context))
    }

    #[inline]
    fn io_with_context<F>(self, f: F) -> std::io::Result<T>
    where F: FnOnce() -> String {
        self.map_err(|e| e.io_context(f()))
    }
}
