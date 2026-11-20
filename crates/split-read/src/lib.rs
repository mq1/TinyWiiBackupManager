// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod read_exact_at;

use crate::read_exact_at::ReadExactAt;
use std::{fs::File, io, sync::Arc};

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
    stream_len: u64,
}

impl SplitFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> io::Result<Self> {
        let mut parts = Vec::new();
        let mut stream_len = 0;

        for file in files {
            let part = Part::try_new(file)?;
            stream_len += part.size;
            parts.push(part);
        }

        Ok(Self {
            parts: parts.into(),
            stream_len,
        })
    }

    #[inline]
    pub fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
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
