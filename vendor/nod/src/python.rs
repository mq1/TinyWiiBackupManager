use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Read as IoRead, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

use pyo3::{
    exceptions::{PyIOError, PyRuntimeError, PyValueError},
    prelude::*,
    types::PyBytes,
};

use crate::{
    common::{Compression, Format, PartitionKind as NodPartitionKind},
    disc::{DiscHeader as NodDiscHeader, fst::Node},
    read::{
        DiscMeta as NodDiscMeta, DiscOptions, DiscReader as NodDiscReader, PartitionMeta,
        PartitionOptions, PartitionReader,
    },
    write::{DiscWriter as NodDiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};

fn nod_err(e: crate::Error) -> PyErr {
    use std::io::ErrorKind;

    use pyo3::exceptions::PyFileNotFoundError;
    match &e {
        crate::Error::Io(_, io_err) if io_err.kind() == ErrorKind::NotFound => {
            PyFileNotFoundError::new_err(format!("{e}"))
        }
        _ => PyIOError::new_err(format!("{e}")),
    }
}

fn io_err(e: std::io::Error) -> PyErr { PyIOError::new_err(format!("{e}")) }

// ---------------------------------------------------------------------------
// DiscHeader
// ---------------------------------------------------------------------------

#[pyclass(name = "DiscHeader", frozen)]
pub struct PyDiscHeader {
    #[pyo3(get)]
    pub game_id: String,
    #[pyo3(get)]
    pub game_title: String,
    #[pyo3(get)]
    pub disc_num: u8,
    #[pyo3(get)]
    pub disc_version: u8,
    #[pyo3(get)]
    pub audio_streaming: u8,
    #[pyo3(get)]
    pub audio_stream_buf_size: u8,
    #[pyo3(get)]
    pub is_wii: bool,
    #[pyo3(get)]
    pub is_gamecube: bool,
}

#[pymethods]
impl PyDiscHeader {
    fn __repr__(&self) -> String {
        format!(
            "DiscHeader(game_id={:?}, game_title={:?}, is_wii={})",
            self.game_id, self.game_title, self.is_wii
        )
    }
}

fn from_disc_header(h: &NodDiscHeader) -> PyDiscHeader {
    PyDiscHeader {
        game_id: h.game_id_str().to_string(),
        game_title: h.game_title_str().to_string(),
        disc_num: h.disc_num,
        disc_version: h.disc_version,
        audio_streaming: h.audio_streaming,
        audio_stream_buf_size: h.audio_stream_buf_size,
        is_wii: h.is_wii(),
        is_gamecube: h.is_gamecube(),
    }
}

// ---------------------------------------------------------------------------
// DiscMeta
// ---------------------------------------------------------------------------

#[pyclass(name = "DiscMeta", frozen)]
pub struct PyDiscMeta {
    #[pyo3(get)]
    pub format: String,
    #[pyo3(get)]
    pub compression: String,
    #[pyo3(get)]
    pub block_size: Option<u32>,
    #[pyo3(get)]
    pub decrypted: bool,
    #[pyo3(get)]
    pub needs_hash_recovery: bool,
    #[pyo3(get)]
    pub lossless: bool,
    #[pyo3(get)]
    pub disc_size: Option<u64>,
    #[pyo3(get)]
    pub crc32: Option<u32>,
    #[pyo3(get)]
    pub xxh64: Option<u64>,
}

#[pymethods]
impl PyDiscMeta {
    fn __repr__(&self) -> String {
        format!("DiscMeta(format={:?}, compression={:?})", self.format, self.compression)
    }
}

fn from_disc_meta(m: &NodDiscMeta) -> PyDiscMeta {
    PyDiscMeta {
        format: m.format.to_string(),
        compression: m.compression.to_string(),
        block_size: m.block_size,
        decrypted: m.decrypted,
        needs_hash_recovery: m.needs_hash_recovery,
        lossless: m.lossless,
        disc_size: m.disc_size,
        crc32: m.crc32,
        xxh64: m.xxh64,
    }
}

// ---------------------------------------------------------------------------
// PartitionInfo
// ---------------------------------------------------------------------------

#[pyclass(name = "PartitionInfo", frozen)]
pub struct PyPartitionInfo {
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub kind: String,
}

#[pymethods]
impl PyPartitionInfo {
    fn __repr__(&self) -> String {
        format!("PartitionInfo(index={}, kind={:?})", self.index, self.kind)
    }
}

// ---------------------------------------------------------------------------
// FstNode
// ---------------------------------------------------------------------------

/// A single file system entry. Returned by [`Fst.find`] and [`Fst.__iter__`].
/// Pass to [`PartitionReader.read_file`] to read the file contents.
#[pyclass(name = "FstNode", frozen, skip_from_py_object)]
#[derive(Clone)]
pub struct PyFstNode {
    /// The name component of this entry (last path segment).
    #[pyo3(get)]
    pub name: String,
    /// The full path from the partition root, using `/` as separator.
    #[pyo3(get)]
    pub path: String,
    #[pyo3(get)]
    pub is_file: bool,
    #[pyo3(get)]
    pub is_dir: bool,
    /// For files: the byte size of the file.
    /// For directories: the child-end index in the FST.
    #[pyo3(get)]
    pub length: u32,
    /// Index of this node in the FST array.
    #[pyo3(get)]
    pub fst_index: usize,
    // Kept internal; used by PartitionReader.read_file.
    pub(crate) node: Node,
}

#[pymethods]
impl PyFstNode {
    fn __repr__(&self) -> String {
        if self.is_file {
            format!("FstNode(path={:?}, length={})", self.path, self.length)
        } else {
            format!("FstNode(path={:?}, dir=True)", self.path)
        }
    }
}

// ---------------------------------------------------------------------------
// FstIter
// ---------------------------------------------------------------------------

#[pyclass(name = "FstIter")]
pub struct PyFstIter {
    entries: Vec<PyFstNode>,
    index: usize,
}

#[pymethods]
impl PyFstIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> { slf }

    fn __next__(&mut self) -> Option<PyFstNode> {
        if self.index < self.entries.len() {
            let entry = self.entries[self.index].clone();
            self.index += 1;
            Some(entry)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Fst
// ---------------------------------------------------------------------------

#[pyclass(name = "Fst")]
pub struct PyFst {
    raw_fst: Arc<[u8]>,
}

#[pymethods]
impl PyFst {
    /// Find a file or directory by its path (case-insensitive).
    /// Returns `None` if not found.
    fn find(&self, path: &str) -> PyResult<Option<PyFstNode>> {
        let buf: &[u8] = &self.raw_fst;
        let fst = crate::disc::fst::Fst::new(buf)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid FST: {e}")))?;
        Ok(fst.find(path).map(|(idx, node)| {
            let name = fst.get_name(node).unwrap_or_default().into_owned();
            PyFstNode {
                name,
                path: path.trim_matches('/').to_string(),
                is_file: node.is_file(),
                is_dir: node.is_dir(),
                length: node.length(),
                fst_index: idx,
                node,
            }
        }))
    }

    fn __iter__(&self) -> PyResult<PyFstIter> {
        let buf: &[u8] = &self.raw_fst;
        let fst = crate::disc::fst::Fst::new(buf)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid FST: {e}")))?;
        let entries: Vec<PyFstNode> = fst
            .iter()
            .map(|(idx, node, path)| {
                let name = fst.get_name(node).unwrap_or_default().into_owned();
                PyFstNode {
                    name,
                    path,
                    is_file: node.is_file(),
                    is_dir: node.is_dir(),
                    length: node.length(),
                    fst_index: idx,
                    node,
                }
            })
            .collect();
        Ok(PyFstIter { entries, index: 0 })
    }

    fn __repr__(&self) -> String { "Fst(...)".to_string() }
}

// ---------------------------------------------------------------------------
// PartitionMeta
// ---------------------------------------------------------------------------

#[pyclass(name = "PartitionMeta")]
pub struct PyPartitionMeta {
    inner: Arc<PartitionMeta>,
}

#[pymethods]
impl PyPartitionMeta {
    /// Returns the file system table.
    fn fst(&self) -> PyResult<PyFst> { Ok(PyFst { raw_fst: Arc::clone(&self.inner.raw_fst) }) }

    /// Disc and boot header (boot.bin, 0x440 bytes).
    #[getter]
    fn raw_boot<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.inner.raw_boot.as_ref())
    }

    /// Debug and region information (bi2.bin, 0x2000 bytes).
    #[getter]
    fn raw_bi2<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, self.inner.raw_bi2.as_ref())
    }

    /// Apploader binary (apploader.bin).
    #[getter]
    fn raw_apploader<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.raw_apploader)
    }

    /// Main executable binary (main.dol).
    #[getter]
    fn raw_dol<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.raw_dol)
    }

    /// Raw file system table (fst.bin).
    #[getter]
    fn raw_fst<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.raw_fst)
    }

    /// Wii ticket (ticket.bin), or `None` for GameCube discs.
    #[getter]
    fn raw_ticket<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.raw_ticket.as_deref().map(|b| PyBytes::new(py, b))
    }

    /// Wii title metadata (tmd.bin), or `None` for GameCube discs.
    #[getter]
    fn raw_tmd<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.raw_tmd.as_deref().map(|b| PyBytes::new(py, b))
    }

    /// Wii certificate chain (cert.bin), or `None` for GameCube discs.
    #[getter]
    fn raw_cert_chain<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.raw_cert_chain.as_deref().map(|b| PyBytes::new(py, b))
    }

    /// Wii H3 hash table (h3.bin), or `None` for GameCube discs.
    #[getter]
    fn raw_h3_table<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.raw_h3_table.as_deref().map(|b| PyBytes::new(py, b.as_ref()))
    }

    /// Disc header information parsed from boot.bin.
    fn disc_header(&self) -> PyDiscHeader { from_disc_header(self.inner.disc_header()) }

    fn __repr__(&self) -> String { "PartitionMeta(...)".to_string() }
}

// ---------------------------------------------------------------------------
// FileReader
// ---------------------------------------------------------------------------

/// A lazy, seekable binary file reader backed by a disc partition.
///
/// Returned by :meth:`PartitionReader.read_file`. Reads are issued against
/// the source disc on demand — no data is buffered until you call
/// :meth:`read`.  Implements the :class:`io.RawIOBase` interface
/// (``read``, ``seek``, ``tell``, ``readable``, ``seekable``,
/// ``writable``, context manager).
#[pyclass(name = "FileReader")]
pub struct PyFileReader {
    /// Shared reference to the underlying partition (same object as the
    /// `PyPartitionReader` that created us).
    partition: Arc<Mutex<Box<dyn PartitionReader>>>,
    /// Absolute byte offset of the file within the partition stream.
    file_offset: u64,
    /// File size in bytes.
    file_size: u64,
    /// Current read position relative to the start of the file.
    pos: u64,
    closed: bool,
}

#[pymethods]
impl PyFileReader {
    /// Read and return up to *size* bytes. If *size* is ``-1`` or omitted,
    /// reads until end of file.
    #[pyo3(signature = (size = -1))]
    fn read<'py>(&mut self, py: Python<'py>, size: i64) -> PyResult<Bound<'py, PyBytes>> {
        self.check_open()?;
        let remaining = self.file_size.saturating_sub(self.pos);
        let to_read = if size < 0 { remaining } else { (size as u64).min(remaining) } as usize;
        if to_read == 0 {
            return Ok(PyBytes::new(py, &[]));
        }
        let abs_pos = self.file_offset + self.pos;
        let mut buf = vec![0u8; to_read];
        {
            let mut guard = self.partition.lock().unwrap();
            guard.seek(SeekFrom::Start(abs_pos)).map_err(io_err)?;
            guard.read_exact(&mut buf).map_err(io_err)?;
        }
        self.pos += to_read as u64;
        Ok(PyBytes::new(py, &buf))
    }

    /// Seek to *pos* bytes relative to *whence*:
    ///   0 (default) — start of file, 1 — current position, 2 — end of file.
    ///
    /// Returns the new absolute position.
    #[pyo3(signature = (pos, whence = 0))]
    fn seek(&mut self, pos: i64, whence: i32) -> PyResult<u64> {
        self.check_open()?;
        let new_pos: u64 = match whence {
            0 => pos.max(0) as u64,
            1 => self.pos.saturating_add_signed(pos),
            2 => self.file_size.saturating_add_signed(pos),
            w => return Err(PyValueError::new_err(format!("invalid whence value: {w}"))),
        };
        self.pos = new_pos.min(self.file_size);
        Ok(self.pos)
    }

    /// Return the current stream position.
    fn tell(&self) -> PyResult<u64> {
        self.check_open()?;
        Ok(self.pos)
    }

    /// Return the file size in bytes.
    fn size(&self) -> u64 { self.file_size }

    fn readable(&self) -> bool { true }

    fn seekable(&self) -> bool { true }

    fn writable(&self) -> bool { false }

    #[getter]
    fn closed(&self) -> bool { self.closed }

    fn close(&mut self) { self.closed = true; }

    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> { slf }

    #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<Py<PyAny>>,
        _exc_val: Option<Py<PyAny>>,
        _exc_tb: Option<Py<PyAny>>,
    ) {
        self.close();
    }

    fn __repr__(&self) -> String {
        format!("FileReader(size={}, pos={})", self.file_size, self.pos)
    }
}

impl PyFileReader {
    fn check_open(&self) -> PyResult<()> {
        if self.closed {
            Err(PyValueError::new_err("I/O operation on closed file"))
        } else {
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// PartitionReader
// ---------------------------------------------------------------------------

#[pyclass(name = "PartitionReader")]
pub struct PyPartitionReader {
    inner: Arc<Mutex<Box<dyn PartitionReader>>>,
}

#[pymethods]
impl PyPartitionReader {
    /// Returns `True` for Wii partitions, `False` for GameCube.
    fn is_wii(&self) -> bool { self.inner.lock().unwrap().is_wii() }

    /// Reads the partition header and file system metadata.
    fn meta(&self) -> PyResult<PyPartitionMeta> {
        let meta = self.inner.lock().unwrap().meta().map_err(nod_err)?;
        Ok(PyPartitionMeta { inner: Arc::new(meta) })
    }

    /// Opens a file identified by *node* for lazy on-demand reading.
    ///
    /// Returns a :class:`FileReader` — a seekable, readable binary stream
    /// that issues reads against the disc on demand. No data is read until
    /// you call :meth:`FileReader.read`.
    ///
    /// Raises :exc:`IsADirectoryError` if *node* is a directory.
    fn read_file(&self, node: &PyFstNode) -> PyResult<PyFileReader> {
        if !node.is_file {
            return Err(pyo3::exceptions::PyIsADirectoryError::new_err(format!(
                "{:?} is a directory",
                node.path
            )));
        }
        let is_wii = self.inner.lock().unwrap().is_wii();
        let file_offset = node.node.offset(is_wii);
        let file_size = node.length as u64;
        Ok(PyFileReader {
            partition: Arc::clone(&self.inner),
            file_offset,
            file_size,
            pos: 0,
            closed: false,
        })
    }

    fn __repr__(&self) -> String { format!("PartitionReader(is_wii={})", self.is_wii()) }
}

// ---------------------------------------------------------------------------
// DiscReader
// ---------------------------------------------------------------------------

#[pyclass(name = "DiscReader")]
pub struct PyDiscReader {
    inner: Arc<Mutex<NodDiscReader>>,
}

#[pymethods]
impl PyDiscReader {
    /// Open a disc image for reading from *path*.
    ///
    /// Supports ISO, CISO, GCZ, NFS, RVZ, WBFS, WIA, and TGC formats.
    ///
    /// Raises :exc:`FileNotFoundError` if the file does not exist.
    /// Raises :exc:`OSError` if the file cannot be opened or the format is not recognised.
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let reader = NodDiscReader::new(path, &DiscOptions::default()).map_err(nod_err)?;
        Ok(PyDiscReader { inner: Arc::new(Mutex::new(reader)) })
    }

    /// Returns the disc's primary header.
    fn header(&self) -> PyDiscHeader {
        let guard = self.inner.lock().unwrap();
        from_disc_header(guard.header())
    }

    /// Returns extra metadata about the underlying disc file format.
    fn meta(&self) -> PyDiscMeta {
        let guard = self.inner.lock().unwrap();
        from_disc_meta(&guard.meta())
    }

    /// Returns the disc's size in bytes.
    fn disc_size(&self) -> u64 { self.inner.lock().unwrap().disc_size() }

    /// Returns a list of Wii partitions. Empty for GameCube discs.
    fn partitions(&self) -> Vec<PyPartitionInfo> {
        let guard = self.inner.lock().unwrap();
        guard
            .partitions()
            .iter()
            .map(|p| PyPartitionInfo { index: p.index, kind: p.kind.to_string() })
            .collect()
    }

    /// Opens a partition by index.
    /// For GameCube discs, `index` must be 0.
    #[pyo3(signature = (index, validate_hashes=false))]
    fn open_partition(&self, index: usize, validate_hashes: bool) -> PyResult<PyPartitionReader> {
        let options = PartitionOptions { validate_hashes };
        let reader = self.inner.lock().unwrap().open_partition(index, &options).map_err(nod_err)?;
        Ok(PyPartitionReader { inner: Arc::new(Mutex::new(reader)) })
    }

    /// Opens the first partition matching `kind`.
    /// `kind` is a string: `"Data"`, `"Update"`, `"Channel"`.
    /// For GameCube discs, use `"Data"`.
    #[pyo3(signature = (kind="Data", validate_hashes=false))]
    fn open_partition_kind(
        &self,
        kind: &str,
        validate_hashes: bool,
    ) -> PyResult<PyPartitionReader> {
        let partition_kind = match kind {
            "Data" => NodPartitionKind::Data,
            "Update" => NodPartitionKind::Update,
            "Channel" => NodPartitionKind::Channel,
            other => {
                return Err(PyValueError::new_err(format!(
                    "Unknown partition kind {:?}. Expected \"Data\", \"Update\", or \"Channel\".",
                    other
                )));
            }
        };
        let options = PartitionOptions { validate_hashes };
        let reader = self
            .inner
            .lock()
            .unwrap()
            .open_partition_kind(partition_kind, &options)
            .map_err(nod_err)?;
        Ok(PyPartitionReader { inner: Arc::new(Mutex::new(reader)) })
    }

    fn __repr__(&self) -> String {
        let guard = self.inner.lock().unwrap();
        let h = guard.header();
        format!("DiscReader(game_id={:?}, game_title={:?})", h.game_id_str(), h.game_title_str())
    }
}

// ---------------------------------------------------------------------------
// DiscFinalization
// ---------------------------------------------------------------------------

#[pyclass(name = "DiscFinalization", frozen)]
pub struct PyDiscFinalization {
    #[pyo3(get)]
    pub crc32: Option<u32>,
    #[pyo3(get)]
    pub xxh64: Option<u64>,
    pub md5: Option<[u8; 16]>,
    pub sha1: Option<[u8; 20]>,
    pub header: Vec<u8>,
}

#[pymethods]
impl PyDiscFinalization {
    /// MD5 hash of the input disc data, or `None` if not calculated.
    #[getter]
    fn md5<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.md5.as_ref().map(|b| PyBytes::new(py, b.as_ref()))
    }

    /// SHA-1 hash of the input disc data, or `None` if not calculated.
    #[getter]
    fn sha1<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.sha1.as_ref().map(|b| PyBytes::new(py, b.as_ref()))
    }

    /// Header data that must be written to the start of the output file, if non-empty.
    #[getter]
    fn header<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> { PyBytes::new(py, &self.header) }

    fn __repr__(&self) -> String {
        format!("DiscFinalization(crc32={:?}, xxh64={:?})", self.crc32, self.xxh64)
    }
}

// ---------------------------------------------------------------------------
// DiscWriter
// ---------------------------------------------------------------------------

fn parse_format(s: &str) -> PyResult<Format> {
    match s {
        "ISO" => Ok(Format::Iso),
        "CISO" => Ok(Format::Ciso),
        "GCZ" => Ok(Format::Gcz),
        "RVZ" => Ok(Format::Rvz),
        "WBFS" => Ok(Format::Wbfs),
        "WIA" => Ok(Format::Wia),
        "TGC" => Ok(Format::Tgc),
        other => Err(PyValueError::new_err(format!(
            "Unknown format {:?}. Expected one of: ISO, CISO, GCZ, RVZ, WBFS, WIA, TGC.",
            other
        ))),
    }
}

fn parse_compression(s: &str) -> PyResult<Compression> {
    s.parse::<Compression>().map_err(|e| PyValueError::new_err(e))
}

// SAFETY: DiscWriter is only accessed through the Mutex and never shared concurrently.
struct SendDiscWriter(NodDiscWriter);
unsafe impl Send for SendDiscWriter {}
unsafe impl Sync for SendDiscWriter {}

#[pyclass(name = "DiscWriter")]
pub struct PyDiscWriter {
    inner: Mutex<SendDiscWriter>,
}

#[pymethods]
impl PyDiscWriter {
    /// Creates a new disc writer.
    ///
    /// *disc* is a :class:`DiscReader` opened with :func:`open_disc`.
    ///
    /// *format* is one of ``"ISO"``, ``"CISO"``, ``"GCZ"``, ``"RVZ"``, ``"WBFS"``, ``"WIA"``,
    /// ``"TGC"``.
    ///
    /// *compression* follows the pattern ``"Algorithm"`` or ``"Algorithm:level"``, e.g.
    /// ``"Zstandard:19"``, ``"None"``. Defaults to the format's recommended compression.
    ///
    /// *block_size* defaults to the format's recommended block size (0 = use default).
    #[new]
    #[pyo3(signature = (disc, format, compression=None, block_size=0))]
    fn new(
        disc: &PyDiscReader,
        format: &str,
        compression: Option<&str>,
        block_size: u32,
    ) -> PyResult<Self> {
        let fmt = parse_format(format)?;
        let comp = match compression {
            Some(s) => parse_compression(s)?,
            None => fmt.default_compression(),
        };
        let bs = if block_size == 0 { fmt.default_block_size() } else { block_size };
        let options = FormatOptions { format: fmt, compression: comp, block_size: bs };
        let reader = disc.inner.lock().unwrap().clone();
        let writer = NodDiscWriter::new(reader, &options).map_err(nod_err)?;
        Ok(PyDiscWriter { inner: Mutex::new(SendDiscWriter(writer)) })
    }

    /// Returns the progress upper bound. Can be used to display progress from the *progress*
    /// argument of the callback passed to :meth:`process`.
    fn progress_bound(&self) -> u64 { self.inner.lock().unwrap().0.progress_bound() }

    /// Processes the disc and writes the result to *output_path*.
    ///
    /// An optional *callback* ``(progress: int, total: int) -> None`` is called after each
    /// chunk is written and can be used to display progress.
    ///
    /// Returns a :class:`DiscFinalization` with any calculated checksums.
    #[pyo3(signature = (
        output_path,
        *,
        callback=None,
        digest_crc32=false,
        digest_md5=false,
        digest_sha1=false,
        digest_xxh64=false,
        scrub_update_partition=false,
    ))]
    fn process(
        &self,
        py: Python<'_>,
        output_path: &str,
        callback: Option<Py<PyAny>>,
        digest_crc32: bool,
        digest_md5: bool,
        digest_sha1: bool,
        digest_xxh64: bool,
        scrub_update_partition: bool,
    ) -> PyResult<PyDiscFinalization> {
        let options = ProcessOptions {
            #[cfg(feature = "threading")]
            processor_threads: 0,
            digest_crc32,
            digest_md5,
            digest_sha1,
            digest_xxh64,
            scrub: if scrub_update_partition {
                ScrubLevel::UpdatePartition
            } else {
                ScrubLevel::None
            },
        };

        let file = File::create(output_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to create {output_path}: {e}")))?;
        let file = Arc::new(Mutex::new(BufWriter::new(file)));
        let file_write = Arc::clone(&file);

        let result = py.detach(|| {
            self.inner.lock().unwrap().0.process(
                |data, progress, total| {
                    file_write.lock().unwrap().write_all(&data)?;
                    if let Some(callback) = &callback {
                        let callback_result: std::io::Result<()> = Python::attach(|inner_py| {
                            callback.call1(inner_py, (progress, total)).map_err(|err| {
                                std::io::Error::new(std::io::ErrorKind::Other, format!("{err}"))
                            })?;
                            Ok(())
                        });
                        callback_result?;
                    }
                    Ok(())
                },
                &options,
            )
        });

        // Invoke Python progress callback (simplified: call once at end if provided)
        // Full per-chunk callback would require re-acquiring the GIL inside the loop,
        // which py.allow_threads does not allow. A future enhancement could use channels.

        let fin = result.map_err(nod_err)?;

        // Write header to beginning of file if required
        if !fin.header.is_empty() {
            let mut guard = file.lock().unwrap();
            guard.flush().map_err(|e| PyIOError::new_err(format!("{e}")))?;
            let inner = guard.get_mut();
            inner.seek(SeekFrom::Start(0)).map_err(|e| PyIOError::new_err(format!("{e}")))?;
            inner.write_all(&fin.header).map_err(|e| PyIOError::new_err(format!("{e}")))?;
        } else {
            file.lock().unwrap().flush().map_err(|e| PyIOError::new_err(format!("{e}")))?;
        }

        Ok(PyDiscFinalization {
            crc32: fin.crc32,
            xxh64: fin.xxh64,
            md5: fin.md5,
            sha1: fin.sha1,
            header: fin.header.to_vec(),
        })
    }

    fn __repr__(&self) -> String {
        format!("DiscWriter(progress_bound={})", self.inner.lock().unwrap().0.progress_bound())
    }
}

// ---------------------------------------------------------------------------
// DiscPatcher
// ---------------------------------------------------------------------------

/// File data provider used by the streaming DiscReader produced by DiscPatcher::build.
#[derive(Clone)]
struct PatcherCallback {
    files: HashMap<String, Arc<[u8]>>,
}

impl crate::build::gc::FileCallback for PatcherCallback {
    fn read_file(&mut self, out: &mut [u8], name: &str, offset: u64) -> io::Result<()> {
        let data = self.files.get(name).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("DiscPatcher: file not found: {name}"))
        })?;
        let start = offset as usize;
        let end = start + out.len();
        out.copy_from_slice(data.get(start..end).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "DiscPatcher: {name}: read {start}..{end} out of bounds (size {})",
                    data.len()
                ),
            )
        })?);
        Ok(())
    }
}

/// Patches or extends a GameCube disc by adding or replacing files.
///
/// The result of :meth:`build` is a :class:`DiscReader` that can be passed
/// directly to :class:`DiscWriter` for conversion to any supported output format.
///
/// Example::
///
///     disc = nod.DiscReader("original.iso")
///     patcher = nod.DiscPatcher(disc)
///     with open("new_audio.dsp", "rb") as f:
///         patcher.add_file("files/audio/bgm.dsp", f.read())
///     patched = patcher.build()
///     nod.DiscWriter(patched, "ISO").process("patched.iso")
#[pyclass(name = "DiscPatcher")]
pub struct PyDiscPatcher {
    disc: Arc<Mutex<NodDiscReader>>,
    overrides: HashMap<String, Arc<[u8]>>,
    header_overrides: crate::build::gc::PartitionOverrides,
    dol_override: Option<Arc<[u8]>>,
}

#[pymethods]
impl PyDiscPatcher {
    /// Create a patcher for *disc*.
    ///
    /// Raises :exc:`ValueError` if *disc* is a Wii disc.
    #[new]
    fn new(disc: &PyDiscReader) -> PyResult<Self> {
        {
            let guard = disc.inner.lock().unwrap();
            if guard.header().is_wii() {
                return Err(PyValueError::new_err(
                    "DiscPatcher only supports GameCube discs, not Wii",
                ));
            }
        }
        Ok(Self {
            disc: Arc::clone(&disc.inner),
            overrides: HashMap::new(),
            header_overrides: crate::build::gc::PartitionOverrides::default(),
            dol_override: None,
        })
    }

    /// Add a new file or replace an existing file in the disc.
    ///
    /// *path* is the FST path (e.g. ``"files/audio/bgm.dsp"``). Leading
    /// slashes are stripped. Calling this a second time with the same path
    /// replaces the previous override.
    ///
    /// Raises :exc:`ValueError` if *path* starts with ``"sys/"``.
    fn add_file(&mut self, path: &str, data: &[u8]) -> PyResult<()> {
        let path = path.trim_start_matches('/').to_string();
        if path.starts_with("sys/") {
            return Err(PyValueError::new_err("Cannot override system files (sys/) via add_file"));
        }
        self.overrides.insert(path, Arc::from(data));
        Ok(())
    }

    /// Replace the main executable (DOL) in the patched disc.
    ///
    /// *data* must be a valid DOL binary. Calling this a second time
    /// replaces the previous override.
    fn set_dol(&mut self, data: &[u8]) { self.dol_override = Some(Arc::from(data)); }

    /// Override disc header fields in the patched disc.
    ///
    /// All parameters are optional; only those provided are changed.
    ///
    /// - *game_id*: six-character ASCII game ID, e.g. ``"GM8E01"``
    /// - *game_title*: game title string
    /// - *disc_num*: disc number (0-based)
    /// - *disc_version*: disc revision number
    /// - *audio_streaming*: audio streaming flag (``True``/``False``)
    /// - *audio_stream_buf_size*: audio stream buffer size
    #[pyo3(signature = (
        *,
        game_id=None,
        game_title=None,
        disc_num=None,
        disc_version=None,
        audio_streaming=None,
        audio_stream_buf_size=None,
    ))]
    fn set_header(
        &mut self,
        game_id: Option<&str>,
        game_title: Option<&str>,
        disc_num: Option<u8>,
        disc_version: Option<u8>,
        audio_streaming: Option<bool>,
        audio_stream_buf_size: Option<u8>,
    ) -> PyResult<()> {
        if let Some(id) = game_id {
            if id.len() != 6 {
                return Err(PyValueError::new_err(format!(
                    "game_id must be exactly 6 characters, got {}",
                    id.len()
                )));
            }
            let mut arr = [0u8; 6];
            arr.copy_from_slice(id.as_bytes());
            self.header_overrides.game_id = Some(arr);
        }
        if let Some(title) = game_title {
            self.header_overrides.game_title = Some(title.to_string());
        }
        if let Some(v) = disc_num {
            self.header_overrides.disc_num = Some(v);
        }
        if let Some(v) = disc_version {
            self.header_overrides.disc_version = Some(v);
        }
        if let Some(v) = audio_streaming {
            self.header_overrides.audio_streaming = Some(v);
        }
        if let Some(v) = audio_stream_buf_size {
            self.header_overrides.audio_stream_buf_size = Some(v);
        }
        Ok(())
    }

    /// Build a new :class:`DiscReader` with all patches applied.
    ///
    /// Reads all files from the source disc, applies any overrides added via
    /// :meth:`add_file`, and returns a :class:`DiscReader` ready for
    /// :class:`DiscWriter`. Non-overridden files are read from the source disc
    /// into memory at this point.
    ///
    /// Raises :exc:`OSError` if the source disc cannot be read.
    /// Raises :exc:`RuntimeError` if the disc layout is invalid.
    fn build(&self) -> PyResult<PyDiscReader> {
        use crate::{
            build::gc::{FileInfo, GCPartitionBuilder},
            disc::{BB2_OFFSET, BI2_SIZE, BOOT_SIZE, fst::Fst},
        };

        // Open the data partition (index 0 on GameCube).
        let (meta, mut partition) = {
            let guard = self.disc.lock().unwrap();
            let mut part =
                guard.open_partition(0, &PartitionOptions::default()).map_err(nod_err)?;
            let meta = part.meta().map_err(nod_err)?;
            (meta, part)
        };

        let mut builder = GCPartitionBuilder::new(false, self.header_overrides.clone());
        // Maps FST path / sys-file name → file bytes for the streaming callback.
        let mut file_map: HashMap<String, Arc<[u8]>> = HashMap::new();

        // ---- System files ------------------------------------------------
        // boot.bin: disc header + boot header.  locate_sys_files() reads this
        // via the sys_file_callback to populate disc_header / boot_header.
        //
        // Zero out all layout offsets/sizes in BootHeader so the builder
        // recalculates them fresh.  This avoids overlaps (e.g. Kirby Air Ride
        // where the original DOL offset overlaps the apploader) and handles
        // games where the DOL is stored as a user-data FST entry (Metroid
        // Prime 2) rather than in the system area.
        let mut boot_bytes = meta.raw_boot.as_ref().to_vec();
        // BootHeader starts at BB2_OFFSET (0x420).  Field layout (all U32 BE):
        //   +0  dol_offset
        //   +4  fst_offset
        //   +8  fst_size
        //  +12  fst_max_size
        //  +16  fst_memory_address  (RAM load address – leave as-is)
        //  +20  user_offset
        //  +24  user_size
        for field_off in [0usize, 4, 8, 12, 20, 24] {
            let start = BB2_OFFSET + field_off;
            boot_bytes[start..start + 4].fill(0);
        }
        builder
            .add_file(FileInfo {
                name: "sys/boot.bin".to_string(),
                size: boot_bytes.len() as u64,
                offset: Some(0),
                alignment: None,
            })
            .map_err(nod_err)?;
        file_map.insert("sys/boot.bin".to_string(), Arc::from(boot_bytes.as_slice()));

        // bi2.bin: debug / region info.
        builder
            .add_file(FileInfo {
                name: "sys/bi2.bin".to_string(),
                size: meta.raw_bi2.len() as u64,
                offset: Some(BOOT_SIZE as u64),
                alignment: None,
            })
            .map_err(nod_err)?;
        file_map.insert("sys/bi2.bin".to_string(), Arc::from(meta.raw_bi2.as_ref() as &[u8]));

        // apploader.img: content provided at stream time via PatcherCallback.
        let apploader_offset = (BOOT_SIZE + BI2_SIZE) as u64;
        builder
            .add_file(FileInfo {
                name: "sys/apploader.img".to_string(),
                size: meta.raw_apploader.len() as u64,
                offset: Some(apploader_offset),
                alignment: None,
            })
            .map_err(nod_err)?;
        file_map.insert("sys/apploader.img".to_string(), Arc::from(meta.raw_apploader.as_ref()));

        // main.dol: always placed after the apploader by the builder because
        // we zeroed dol_offset above.
        let dol_data: Arc<[u8]> =
            self.dol_override.clone().unwrap_or_else(|| Arc::from(meta.raw_dol.as_ref()));
        builder
            .add_file(FileInfo {
                name: "sys/main.dol".to_string(),
                size: dol_data.len() as u64,
                offset: None,
                alignment: Some(128),
            })
            .map_err(nod_err)?;
        file_map.insert("sys/main.dol".to_string(), dol_data);

        // ---- User files from the original FST ----------------------------
        let fst = Fst::new(&meta.raw_fst)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid FST: {e}")))?;

        for (_, node, path) in fst.iter() {
            if !node.is_file() {
                continue;
            }
            let data: Arc<[u8]> = if let Some(ov) = self.overrides.get(&path) {
                ov.clone()
            } else {
                // Read file from the source partition.  Some discs have FST
                // entries whose data lives in a junk region (LFG-generated
                // padding).  Reads of those entries typically succeed and
                // return the junk bytes; if they fail for any reason we fall
                // back to zeros so the file at least appears in the FST.
                let size = node.length() as usize;
                let data = partition
                    .open_file(node)
                    .ok()
                    .and_then(|mut f| {
                        let mut buf = Vec::with_capacity(size);
                        f.read_to_end(&mut buf).ok().map(|_| buf)
                    })
                    .unwrap_or_else(|| vec![0u8; size]);
                Arc::from(data)
            };
            builder
                .add_file(FileInfo {
                    name: path.clone(),
                    size: data.len() as u64,
                    offset: None,
                    alignment: None,
                })
                .map_err(nod_err)?;
            file_map.insert(path, data);
        }

        // ---- New files from overrides not present in the original FST ----
        for (path, data) in &self.overrides {
            if !file_map.contains_key(path.as_str()) {
                builder
                    .add_file(FileInfo {
                        name: path.clone(),
                        size: data.len() as u64,
                        offset: None,
                        alignment: None,
                    })
                    .map_err(nod_err)?;
                file_map.insert(path.clone(), data.clone());
            }
        }

        // ---- Build layout ------------------------------------------------
        // sys_file_callback is called during build() for boot.bin and bi2.bin.
        let sys_files = file_map.clone();
        let partition_writer = builder
            .build(|w, name| {
                let data = sys_files.get(name).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("DiscPatcher build: file not found: {name}"),
                    )
                })?;
                w.write_all(data)
            })
            .map_err(nod_err)?;

        let callback = PatcherCallback { files: file_map };
        let stream = partition_writer.into_cloneable_stream(callback).map_err(nod_err)?;
        let reader = NodDiscReader::new_stream(stream, &DiscOptions::default()).map_err(nod_err)?;

        Ok(PyDiscReader { inner: Arc::new(Mutex::new(reader)) })
    }

    fn __repr__(&self) -> String { format!("DiscPatcher(overrides={})", self.overrides.len()) }
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDiscReader>()?;
    m.add_class::<PyDiscHeader>()?;
    m.add_class::<PyDiscMeta>()?;
    m.add_class::<PyPartitionInfo>()?;
    m.add_class::<PyFileReader>()?;
    m.add_class::<PyPartitionReader>()?;
    m.add_class::<PyPartitionMeta>()?;
    m.add_class::<PyFst>()?;
    m.add_class::<PyFstNode>()?;
    m.add_class::<PyFstIter>()?;
    m.add_class::<PyDiscWriter>()?;
    m.add_class::<PyDiscFinalization>()?;
    m.add_class::<PyDiscPatcher>()?;
    Ok(())
}
