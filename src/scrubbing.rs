// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::io::{Read, Seek, SeekFrom};

// use this only on wbfs files
pub fn is_worth_scrubbing<R: Read + Seek>(disc_reader: &mut R) -> bool {
    let mut buf = [0u8; 4];

    // check if the first partition is an update one
    if disc_reader.seek(SeekFrom::Start(0x240024)).is_err() {
        return false;
    }
    if disc_reader.read(&mut buf).is_err() {
        return false;
    }
    if buf != [0, 0, 0, 1] {
        return false;
    }

    // check if the update data is unmapped
    if disc_reader.seek(SeekFrom::Start(0x302)).is_err() {
        return false;
    }
    if disc_reader.read(&mut buf).is_err() {
        return false;
    }
    buf != [0, 0, 0, 0]
}
