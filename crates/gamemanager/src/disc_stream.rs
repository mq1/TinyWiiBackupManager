// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use nod::read::DiscStream;
use split_read::SplitFileReader;
use std::io;

#[derive(Debug, Clone)]
pub struct SplitFileDiscStream(pub SplitFileReader);

impl DiscStream for SplitFileDiscStream {
    #[inline]
    fn read_exact_at(&mut self, buf: &mut [u8], offset: u64) -> io::Result<()> {
        self.0.read_exact_at(buf, offset)
    }

    #[inline]
    fn stream_len(&mut self) -> io::Result<u64> {
        self.0.stream_len()
    }
}
