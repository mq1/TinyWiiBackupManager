// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs::File, io};

#[derive(Debug)]
pub struct Part {
    pub file: File,
    pub size: u64,
}

impl Part {
    pub fn try_new(file: File) -> io::Result<Self> {
        match file.metadata().map(|m| m.len()) {
            Ok(0) => Err(io::ErrorKind::InvalidData.into()),
            Ok(size) => Ok(Self { file, size }),
            Err(e) => Err(e),
        }
    }
}
