// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod read_at;

use crate::read_at::ReadAt;
use std::{
    fs::File,
    io::{self, Read},
    sync::Arc,
};

#[derive(Debug)]
struct Part {
    file: File,
    size: u64,
}

impl Part {
    fn try_new(file: File) -> io::Result<Self> {
        match file.metadata()?.len() {
            0 => Err(io::ErrorKind::InvalidData.into()),
            size => Ok(Self { file, size }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SplitFileReader {
    parts: Arc<[Part]>,
    total_size: u64,
    pos: u64,
}

impl SplitFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> io::Result<Self> {
        let files = files.into_iter();
        let mut parts = Vec::with_capacity(files.size_hint().0);
        let mut total_size = 0;

        for file in files {
            let part = Part::try_new(file)?;
            total_size += part.size;
            parts.push(part);
        }

        Ok(Self {
            parts: parts.into(),
            total_size,
            pos: 0,
        })
    }

    #[inline]
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn read_at(&mut self, buf: &mut [u8], mut offset: u64) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        for part in self.parts.iter() {
            if offset >= part.size {
                offset -= part.size;
                continue;
            }

            return part.file.read_at(buf, offset);
        }

        Err(io::ErrorKind::UnexpectedEof.into())
    }

    pub fn read_exact_at(&mut self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        if buf.is_empty() {
            return Ok(());
        }

        for part in self.parts.iter() {
            if offset >= part.size {
                offset -= part.size;
                continue;
            }

            let n = (part.size - offset).min(buf.len() as u64) as usize;
            let (head, tail) = buf.split_at_mut(n);

            part.file.read_exact_at(head, offset)?;

            if tail.is_empty() {
                return Ok(());
            }

            buf = tail;
            offset = 0;
        }

        Err(io::ErrorKind::UnexpectedEof.into())
    }
}

impl Read for SplitFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.read_at(buf, self.pos)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.read_exact_at(buf, self.pos)?;
        self.pos += buf.len() as u64;
        Ok(())
    }
}
