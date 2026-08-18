// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use either::Either;
use ouroboros::self_referencing;
use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::Path,
};
use zip::{ZipArchive, read::ZipFileSeek};

struct Plain {
    inner: BufReader<File>,
}

#[self_referencing]
struct Zipped {
    inner: ZipArchive<BufReader<File>>,
    #[borrows(mut inner)]
    #[covariant]
    entry: ZipFileSeek<'this, BufReader<File>>,
}

pub struct DiscFileReader {
    inner: Either<Plain, Zipped>,
}

impl DiscFileReader {
    pub fn new(path: &Path) -> Result<Self, Error> {
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let zip = ZipArchive::new(reader)?;

            let zipped = ZippedTryBuilder {
                inner: zip,
                entry_builder: |zip| zip.by_index_seek(0),
            }
            .try_build()?;

            Ok(Self {
                inner: Either::Right(zipped),
            })
        } else {
            let file = File::open(path)?;
            let reader = BufReader::new(file);

            Ok(Self {
                inner: Either::Left(Plain { inner: reader }),
            })
        }
    }
}

impl Read for DiscFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            Either::Left(p) => p.inner.read(buf),
            Either::Right(z) => z.with_entry_mut(|e| e.read(buf)),
        }
    }
}

impl Seek for DiscFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match &mut self.inner {
            Either::Left(p) => p.inner.seek(pos),
            Either::Right(z) => z.with_entry_mut(|e| e.seek(pos)),
        }
    }
}
