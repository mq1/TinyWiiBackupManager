// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::util::ext::SUPPORTED_INPUT_EXTENSIONS;
use anyhow::{Result, anyhow, bail};
use std::fs;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::NamedTempFile;

/// Returns `true` if we can create a file >4 GiB in this directory
pub fn can_write_over_4gb(path: impl AsRef<Path>) -> bool {
    let result = (|| {
        // Create a temp file in the target directory
        let mut tmp = NamedTempFile::new_in(path)?;

        // Seek to 4 GiB
        tmp.as_file_mut()
            .seek(SeekFrom::Start(4 * 1024 * 1024 * 1024))?;

        // Write a single byte
        tmp.as_file_mut().write_all(&[0])?;

        Ok::<_, std::io::Error>(())
    })();

    result.is_ok()
}

pub fn find_disc(game_dir: impl AsRef<Path>) -> Result<PathBuf> {
    for entry in fs::read_dir(game_dir)?.filter_map(Result::ok) {
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();

        if !file_name.starts_with(".")
            && path.is_file()
            && SUPPORTED_INPUT_EXTENSIONS.contains(&extension)
            && !file_name.ends_with(".part1.iso")
            && !file_name.ends_with(".part2.iso")
            && !file_name.ends_with(".part3.iso")
        {
            return Ok(path);
        }
    }

    bail!("No disc found in game directory")
}

pub fn to_multipart(file_path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let f1 = file_path
        .as_ref()
        .file_name()
        .ok_or(anyhow!("No file name"))?
        .to_string_lossy();

    let parent = file_path.as_ref().parent().ok_or(anyhow!("No parent"))?;

    let mut paths = vec![file_path.as_ref().to_path_buf()];

    if f1.ends_with(".part0.iso") {
        for i in 1..4 {
            let i_file_name = f1.replace(".part0.", &format!(".part{i}."));
            let path = parent.join(i_file_name);
            if path.exists() {
                paths.push(path);
            } else {
                break;
            }
        }
    } else if f1.ends_with(".wbfs") {
        for i in 1..4 {
            let i_file_name = f1.replace(".wbfs", &format!(".wbf{i}"));
            let path = parent.join(i_file_name);
            if path.exists() {
                paths.push(path);
            } else {
                break;
            }
        }
    }

    Ok(paths)
}

/// Converts a dir name (Game Title [123456])
pub fn dir_to_title_id(path: impl AsRef<Path>) -> Result<(String, [u8; 6], String)> {
    let path = path.as_ref();

    if !path.is_dir() {
        bail!("Path must be a directory");
    }

    let dir_name = path
        .file_name()
        .ok_or(anyhow!("Failed to get directory name"))?
        .to_string_lossy();

    // Ignore hidden directories
    if dir_name.starts_with(".") {
        bail!("Directory name starts with '.'");
    }

    let id_start = dir_name
        .find('[')
        .ok_or(anyhow!("Failed to find '[' in directory name"))?;

    let id_end = dir_name
        .find(']')
        .ok_or(anyhow!("Failed to find ']' in directory name"))?;

    let title = dir_name[..id_start].trim().to_string();
    let id_str = &dir_name[id_start + 1..id_end];

    let mut id = [0u8; 6];
    let bytes = id_str.as_bytes();
    let len = bytes.len().min(6);
    id[..len].copy_from_slice(&bytes[..len]);

    Ok((title, id, id_str.to_string()))
}

/// MultiFileReader presents several files as a single, concatenated, read-only stream.
/// - Read: reads across file boundaries seamlessly.
/// - Seek: seeks relative to start/current/end across the logical concatenated file.
///   Seeking beyond end is allowed (like File); subsequent reads return 0.
/// - Clone: clones share the file list and lengths but have independent cursors.
///   Clones do not share OS handles; files are (re)opened lazily as needed.
#[derive(Debug)]
pub struct MultiFileReader {
    // Shared, immutable metadata
    paths: Arc<[PathBuf]>,
    lengths: Arc<[u64]>, // per-file lengths
    starts: Arc<[u64]>,  // starting absolute offsets per file
    total_len: u64,

    // Per-instance state
    global_pos: u64,         // logical cursor (can be > total_len)
    open_idx: Option<usize>, // which file is currently open
    file: Option<File>,      // the actual open handle (position kept in OS)
}

impl MultiFileReader {
    /// Build from a list of paths. Files are opened lazily on first read touching them.
    pub fn new<P: AsRef<Path>>(paths: impl IntoIterator<Item = P>) -> io::Result<Self> {
        let paths: Vec<PathBuf> = paths
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();

        let mut lengths = Vec::with_capacity(paths.len());
        for p in &paths {
            let len = std::fs::metadata(p)?.len();
            lengths.push(len);
        }

        let mut starts = Vec::with_capacity(paths.len());
        let mut acc = 0u64;
        for &len in &lengths {
            starts.push(acc);
            acc = acc.checked_add(len).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "total length overflow")
            })?;
        }

        Ok(Self {
            paths: paths.into(),
            lengths: lengths.into(),
            starts: starts.into(),
            total_len: acc,
            global_pos: 0,
            open_idx: None,
            file: None,
        })
    }

    /// Total logical length (sum of all file sizes).
    pub fn len(&self) -> u64 {
        self.total_len
    }

    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    // Find which file contains absolute position `pos` and return (file_index, offset_in_file).
    // Requires pos < total_len.
    fn locate(&self, pos: u64) -> (usize, u64) {
        // starts is sorted. We want the greatest i such that starts[i] <= pos.
        match self.starts.binary_search(&pos) {
            Ok(i) => (i, 0),
            Err(i) => {
                let idx = i - 1;
                (idx, pos - self.starts[idx])
            }
        }
    }

    // Ensure the correct file is open and positioned to `offset` within file `idx`.
    fn open_at(&mut self, idx: usize, offset: u64) -> io::Result<()> {
        if self.open_idx != Some(idx) || self.file.is_none() {
            // Open (or reopen) the needed file
            let mut f = File::open(&self.paths[idx])?;
            f.seek(SeekFrom::Start(offset))?;
            self.file = Some(f);
            self.open_idx = Some(idx);
        } else if let Some(f) = self.file.as_mut() {
            // Reposition if necessary
            f.seek(SeekFrom::Start(offset))?;
        } else {
            // Defensive fallback
            let mut f = File::open(&self.paths[idx])?;
            f.seek(SeekFrom::Start(offset))?;
            self.file = Some(f);
            self.open_idx = Some(idx);
        }
        Ok(())
    }
}

impl Read for MultiFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        if self.global_pos >= self.total_len {
            // Past EOF: read returns 0.
            return Ok(0);
        }

        let mut filled = 0;
        while filled < buf.len() && self.global_pos < self.total_len {
            let (idx, off_in_file) = self.locate(self.global_pos);
            self.open_at(idx, off_in_file)?;

            let file_len = self.lengths[idx];
            let left_in_file = (file_len - off_in_file) as usize;
            let want = left_in_file.min(buf.len() - filled);

            // Safe: we ensured file at correct offset
            let n = if let Some(f) = self.file.as_mut() {
                f.read(&mut buf[filled..filled + want])?
            } else {
                // Shouldn't happen, but be robust
                let mut f = File::open(&self.paths[idx])?;
                f.seek(SeekFrom::Start(off_in_file))?;
                let n = f.read(&mut buf[filled..filled + want])?;
                self.file = Some(f);
                self.open_idx = Some(idx);
                n
            };

            if n == 0 {
                // Unexpected zero read (file shrank?) — advance to next file to avoid infinite loop.
                self.global_pos = self.starts[idx] + self.lengths[idx];
                self.open_idx = None;
                self.file = None;
                continue;
            }

            filled += n;
            self.global_pos += n as u64;
        }

        Ok(filled)
    }
}

impl Seek for MultiFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos_i128 = match pos {
            SeekFrom::Start(s) => s as i128,
            SeekFrom::End(o) => (self.total_len as i128) + (o as i128),
            SeekFrom::Current(o) => (self.global_pos as i128) + (o as i128),
        };

        if new_pos_i128 < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }

        let new_pos_u128 = new_pos_i128 as u128;
        if new_pos_u128 > u64::MAX as u128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek position too large",
            ));
        }

        self.global_pos = new_pos_u128 as u64;

        // Drop any current handle; we will reopen lazily on next read
        self.open_idx = None;
        self.file = None;

        Ok(self.global_pos)
    }
}

impl Clone for MultiFileReader {
    fn clone(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            lengths: self.lengths.clone(),
            starts: self.starts.clone(),
            total_len: self.total_len,
            global_pos: self.global_pos,
            open_idx: None,
            file: None,
        }
    }
}
