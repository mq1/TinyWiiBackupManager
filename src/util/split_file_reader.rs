// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::read::DiscStream;
use std::{fs::File, io, sync::Arc};

struct Part {
    file: File,
    size: u64,
}

impl Part {
    #[cfg(unix)]
    #[inline]
    fn read_exact_at(&self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        std::os::unix::fs::FileExt::read_exact_at(&self.file, buf, offset)
    }

    #[cfg(windows)]
    #[inline]
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match std::os::windows::fs::FileExt::seek_read(&self.file, buf, offset) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(n) => {
                    buf = &mut buf[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct SplitFileReader {
    parts: Arc<[Part]>,
    stream_len: u64,
}

impl SplitFileReader {
    pub fn new(files: impl IntoIterator<Item = File>) -> io::Result<Self> {
        let files = files.into_iter();

        let mut parts = Vec::with_capacity(files.size_hint().0);
        let mut stream_len = 0;

        for file in files {
            let size = file.metadata()?.len();
            if size == 0 {
                return Err(io::ErrorKind::InvalidData.into());
            }
            stream_len += size;

            parts.push(Part { file, size });
        }

        Ok(Self {
            parts: parts.into(),
            stream_len,
        })
    }
}

impl DiscStream for SplitFileReader {
    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.stream_len)
    }

    fn read_exact_at(&mut self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
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

            part.read_exact_at(head, offset)?;

            if tail.is_empty() {
                return Ok(());
            }

            buf = tail;
            offset = 0;
        }

        Err(io::ErrorKind::UnexpectedEof.into())
    }
}
