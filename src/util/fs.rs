// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::util::ext::SUPPORTED_INPUT_EXTENSIONS;
use anyhow::{Result, anyhow, bail};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use walkdir::WalkDir;

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
    for entry in WalkDir::new(game_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if let Some(extension) = path.extension().and_then(|ext| ext.to_str())
            && SUPPORTED_INPUT_EXTENSIONS.contains(&extension)
        {
            return Ok(path.to_path_buf());
        }
    }

    bail!("No supported disc image found in directory");
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
