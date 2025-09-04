// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use crate::util::ext::SUPPORTED_INPUT_EXTENSIONS;
use anyhow::{Result, bail};
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

pub fn find_disc(game: &Game) -> Result<PathBuf> {
    for entry in WalkDir::new(&game.path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if let Some(extension) = path.extension().and_then(|ext| ext.to_str())
            && SUPPORTED_INPUT_EXTENSIONS.contains(&extension)
        {
            return Ok(path.to_path_buf());
        }
    }

    bail!("No supported disc image found in directory");
}
