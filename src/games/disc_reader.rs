// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::{export::ExportCancellation, zip_game_index};
use anyhow::{Context, Result};
use arrayvec::ArrayVec;
use nod::read::DiscStream;
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    sync::Arc,
};
use tempfile::tempfile;
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

#[cfg(windows)]
trait ReadExactAt {
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> io::Result<()>;
}

#[cfg(windows)]
impl ReadExactAt for File {
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.seek_read(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        if !buf.is_empty() {
            Err(io::ErrorKind::UnexpectedEof.into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedMultiFileReader {
    inner: ArrayVec<(Arc<File>, u64), 4>,
    stream_len: u64,
}

impl SharedMultiFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> Result<Self> {
        let mut inner = ArrayVec::new();
        let mut stream_len = 0;

        for file in files {
            let size = file.metadata()?.len();
            inner.push((Arc::new(file), size));
            stream_len += size;
        }

        Ok(Self { inner, stream_len })
    }
}

impl DiscStream for SharedMultiFileReader {
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        let mut offset = offset;
        let mut buf = buf;

        for (file, len) in self.inner.iter() {
            // If the offset is past the end of the file, skip it
            if offset >= *len {
                offset -= len;
                continue;
            }

            // Limit in u64 before narrowing: a single ISO may exceed usize on x86.
            let n = (*len - offset).min(buf.len() as u64) as usize;
            file.read_exact_at(&mut buf[..n], offset)?;
            buf = &mut buf[n..];
            offset = 0;

            if buf.is_empty() {
                return Ok(());
            }
        }

        if buf.is_empty() {
            Ok(())
        } else {
            Err(io::ErrorKind::UnexpectedEof.into())
        }
    }
}

pub trait DiscCancellation: Clone + Send + Sync {
    fn check(&self) -> io::Result<()>;
}

#[derive(Debug)]
pub(crate) struct DiscCancelled(pub &'static str);

impl std::fmt::Display for DiscCancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} cancelled", self.0)
    }
}
impl std::error::Error for DiscCancelled {}

pub(crate) fn error_contains<E: std::error::Error + 'static>(
    error: &(dyn std::error::Error + 'static),
) -> bool {
    if error.is::<E>() {
        return true;
    }
    if let Some(inner) = error
        .downcast_ref::<io::Error>()
        .and_then(io::Error::get_ref)
    {
        return error_contains::<E>(inner);
    }
    error.source().is_some_and(error_contains::<E>)
}

impl DiscCancellation for ExportCancellation {
    fn check(&self) -> io::Result<()> {
        ExportCancellation::check(self)
    }
}

/// Keeps cancellation active when nod clones a stream for its worker threads.
#[derive(Clone)]
pub struct CancellableDiscStream<C = ExportCancellation> {
    inner: SharedMultiFileReader,
    cancel: C,
}

impl<C: DiscCancellation> CancellableDiscStream<C> {
    pub fn new(inner: SharedMultiFileReader, cancel: C) -> Self {
        Self { inner, cancel }
    }
}

impl<C: DiscCancellation + 'static> DiscStream for CancellableDiscStream<C> {
    fn stream_len(&mut self) -> io::Result<u64> {
        self.cancel.check()?;
        let len = self.inner.stream_len()?;
        self.cancel.check()?;
        Ok(len)
    }

    fn read_exact_at(&mut self, buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        self.cancel.check()?;
        // A format may request a large metadata table or compressed block in
        // one call. Bound each source read so cancellation is checked in between.
        for chunk in buf.chunks_mut(1024 * 1024) {
            self.cancel.check()?;
            self.inner.read_exact_at(chunk, offset)?;
            self.cancel.check()?;
            offset = offset
                .checked_add(chunk.len() as u64)
                .ok_or_else(|| io::Error::other("disc read offset overflow"))?;
        }
        self.cancel.check()
    }
}

fn copy_cancellable<C: DiscCancellation>(
    reader: &mut impl Read,
    writer: &mut impl Write,
    cancel: &C,
) -> io::Result<()> {
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        cancel.check()?;
        let read = reader.read(&mut buffer)?;
        cancel.check()?;
        if read == 0 {
            return Ok(());
        }
        writer.write_all(&buffer[..read])?;
        cancel.check()?;
    }
}

pub fn retrieve_multi_disc_files(path: &Path) -> Result<ArrayVec<File, 4>> {
    retrieve_multi_disc_files_inner::<ExportCancellation>(path, None)
}

pub fn retrieve_multi_disc_files_cancellable<C: DiscCancellation>(
    path: &Path,
    cancel: &C,
) -> Result<ArrayVec<File, 4>> {
    retrieve_multi_disc_files_inner(path, Some(cancel))
}

fn retrieve_multi_disc_files_inner<C: DiscCancellation>(
    path: &Path,
    cancel: Option<&C>,
) -> Result<ArrayVec<File, 4>> {
    let check = || cancel.map_or(Ok(()), DiscCancellation::check);
    let open = |path: &Path| -> io::Result<File> {
        check()?;
        let file = File::open(path)?;
        check()?;
        Ok(file)
    };
    check()?;
    let mut files = ArrayVec::new();

    let ext = path.extension().context("invalid extension")?;

    if ext.eq_ignore_ascii_case("zip") {
        let mut zip = ZipArchive::new(open(path)?)?;
        check()?;
        let index = zip_game_index(&mut zip, check)?;
        let mut entry = zip.by_index(index)?;
        check()?;

        let mut tmp = tempfile()?;
        check()?;
        if let Some(cancel) = cancel {
            copy_cancellable(&mut entry, &mut tmp, cancel)?;
        } else {
            io::copy(&mut entry, &mut tmp)?;
        }
        check()?;
        tmp.flush()?;
        check()?;

        files.push(tmp);
    } else {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("invalid filename")?;

        files.push(open(path)?);

        if let Some(filename) = filename.strip_suffix(".part0.iso") {
            for i in 1..=3 {
                check()?;
                let part_path = path.with_file_name(format!("{filename}.part{i}.iso"));
                if i > 1 && !part_path.exists() {
                    break;
                }
                files.push(open(&part_path)?);
            }
        } else if let Some(filename) = filename.strip_suffix(".PART0.ISO") {
            for i in 1..=3 {
                check()?;
                let part_path = path.with_file_name(format!("{filename}.PART{i}.ISO"));
                if i > 1 && !part_path.exists() {
                    break;
                }
                files.push(open(&part_path)?);
            }
        } else if let Some(filename) = filename.strip_suffix(".wbfs") {
            for i in 1..=3 {
                check()?;
                let wbfx_path = path.with_file_name(format!("{filename}.wbf{i}"));
                if !wbfx_path.exists() {
                    break;
                }
                files.push(open(&wbfx_path)?);
            }
        } else if let Some(filename) = filename.strip_suffix(".WBFS") {
            for i in 1..=3 {
                check()?;
                let wbfx_path = path.with_file_name(format!("{filename}.WBF{i}"));
                if !wbfx_path.exists() {
                    break;
                }
                files.push(open(&wbfx_path)?);
            }
        }
    }

    check()?;
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use zip::{ZipWriter, write::SimpleFileOptions};

    #[test]
    fn split_iso_reads_two_and_three_parts() {
        for uppercase in [false, true] {
            for parts in 2..=3 {
                let dir = tempfile::tempdir().unwrap();
                let mut expected = Vec::new();
                let mut primary = None;
                for part in 0..parts {
                    let mut name = format!("game.part{part}.iso");
                    if uppercase {
                        name.make_ascii_uppercase();
                    }
                    let path = dir.path().join(name);
                    let data = vec![part as u8 + 1; 17];
                    std::fs::write(&path, &data).unwrap();
                    expected.extend(data);
                    primary.get_or_insert(path);
                }
                let files = retrieve_multi_disc_files(&primary.unwrap()).unwrap();
                assert_eq!(files.len(), parts);
                let mut reader = SharedMultiFileReader::new(files).unwrap();
                let mut actual = vec![0; expected.len()];
                reader.read_exact_at(&mut actual, 0).unwrap();
                assert_eq!(actual, expected);
            }
        }
    }

    fn zip_data(data: &[u8]) -> Vec<u8> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive
            .start_file("game.iso", SimpleFileOptions::default())
            .unwrap();
        archive.write_all(data).unwrap();
        archive.finish().unwrap().into_inner()
    }

    #[test]
    fn cancellable_stream_reads_split_files_and_shares_cancellation_with_clones() {
        let mut first = tempfile().unwrap();
        let mut second = tempfile().unwrap();
        first.write_all(&vec![0x42; 600_000]).unwrap();
        second.write_all(&vec![0x91; 600_000]).unwrap();
        let cancel = ExportCancellation::default();
        let mut stream = CancellableDiscStream::new(
            SharedMultiFileReader::new([first, second]).unwrap(),
            cancel.clone(),
        );
        assert_eq!(stream.stream_len().unwrap(), 1_200_000);
        let mut data = vec![0; 1_200_000];
        stream.read_exact_at(&mut data, 0).unwrap();
        assert!(data[..600_000].iter().all(|&byte| byte == 0x42));
        assert!(data[600_000..].iter().all(|&byte| byte == 0x91));

        let mut clone = stream.clone();
        cancel.cancel();
        assert_eq!(
            clone.read_exact_at(&mut [0; 4], 0).unwrap_err().kind(),
            io::ErrorKind::Other
        );
        assert_eq!(
            stream.stream_len().unwrap_err().kind(),
            io::ErrorKind::Other
        );
        assert!(stream.read_exact_at(&mut [], 0).is_err());
    }

    #[test]
    fn cancelled_retrieval_stops_before_opening_source() {
        let cancel = ExportCancellation::default();
        cancel.cancel();
        let error =
            retrieve_multi_disc_files_cancellable(Path::new("missing.zip"), &cancel).unwrap_err();
        assert_eq!(
            error.downcast_ref::<io::Error>().unwrap().kind(),
            io::ErrorKind::Other
        );
    }

    #[test]
    fn cancellable_zip_extraction_preserves_source_and_contents() {
        let data = vec![0x65; 1_100_000];
        let archive = zip_data(&data);
        let mut source = tempfile::Builder::new().suffix(".zip").tempfile().unwrap();
        source.write_all(&archive).unwrap();
        let files =
            retrieve_multi_disc_files_cancellable(source.path(), &ExportCancellation::default())
                .unwrap();
        let mut stream = SharedMultiFileReader::new(files).unwrap();
        assert_eq!(stream.stream_len().unwrap(), data.len() as u64);
        let mut extracted = vec![0; data.len()];
        stream.read_exact_at(&mut extracted, 0).unwrap();
        assert_eq!(extracted, data);
        assert_eq!(std::fs::read(source.path()).unwrap(), archive);
    }

    #[test]
    fn cancellation_during_zip_read_stops_before_writing_extracted_chunk() {
        struct CancelAfterRead<R> {
            reader: R,
            cancel: ExportCancellation,
        }
        impl<R: Read> Read for CancelAfterRead<R> {
            fn read(&mut self, data: &mut [u8]) -> io::Result<usize> {
                let read = self.reader.read(data)?;
                self.cancel.cancel();
                Ok(read)
            }
        }

        let mut archive = ZipArchive::new(Cursor::new(zip_data(&vec![7; 1_100_000]))).unwrap();
        let cancel = ExportCancellation::default();
        let mut reader = CancelAfterRead {
            reader: archive.by_index(0).unwrap(),
            cancel: cancel.clone(),
        };
        let mut output = Vec::new();
        let error = copy_cancellable(&mut reader, &mut output, &cancel).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(output.is_empty());
    }
}
