// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::part::Part;
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read, Seek},
};

#[derive(Debug)]
pub struct SplitFileReader {
    parts: VecDeque<Part>,
    total_size: u64,
}

impl SplitFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> io::Result<Self> {
        let files = files.into_iter();
        let mut parts = VecDeque::with_capacity(files.size_hint().0);
        let mut total_size = 0;

        for mut file in files {
            file.rewind()?;
            let part = Part::try_new(file)?;
            total_size += part.size;
            parts.push_back(part);
        }

        Ok(Self { parts, total_size })
    }

    #[inline]
    pub fn total_size(&self) -> u64 {
        self.total_size
    }
}

impl Read for SplitFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        while let Some(part) = self.parts.front_mut() {
            let n = part.file.read(buf)?;
            if n > 0 {
                return Ok(n);
            }
            self.parts.pop_front();
        }

        Ok(0)
    }
}
