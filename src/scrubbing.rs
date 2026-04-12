// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use std::io::{Read, Seek, SeekFrom};

// use this only on wbfs files
pub fn is_worth_scrubbing<R: Read + Seek>(disc_reader: &mut R) -> Result<bool> {
    let mut buf = [0u8; 4];

    // check if the first partition is an update one
    disc_reader.seek(SeekFrom::Start(0x240024))?;
    disc_reader.read(&mut buf)?;
    if buf != [0, 0, 0, 1] {
        return Ok(false);
    }

    // check if the update data is unmapped
    disc_reader.seek(SeekFrom::Start(0x302))?;
    disc_reader.read(&mut buf)?;
    let worth_it = buf != [0, 0, 0, 0];

    Ok(worth_it)
}
