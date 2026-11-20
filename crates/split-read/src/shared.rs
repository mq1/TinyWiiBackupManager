// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{part::Part, read_at::ReadAt};
use std::{fs::File, io, sync::Arc};

#[derive(Debug, Clone)]
pub struct SharedSplitFileReader {
    parts: Arc<[Part]>,
    total_size: u64,
}

impl SharedSplitFileReader {
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
        })
    }

    #[inline]
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    pub fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
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
