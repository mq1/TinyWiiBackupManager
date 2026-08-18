// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use multi_readers::MultiReader;
use ouroboros::self_referencing;
use std::{
    ffi::OsStr,
    fmt::Write,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use zip::{ZipArchive, read::ZipFileSeek};

#[self_referencing]
struct Zipped {
    inner: ZipArchive<File>,

    #[borrows(mut inner)]
    #[covariant]
    entry: ZipFileSeek<'this, File>,
}

enum Inner {
    Files(MultiReader<File>),
    Zipped(Zipped),
}

pub struct DiscFileReader {
    path: PathBuf,
    position: u64,
    inner: Inner,
}

impl DiscFileReader {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        let ext = path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        if ext == "zip" {
            let file = File::open(path)?;
            let zip = ZipArchive::new(file)?;

            let zipped = ZippedTryBuilder {
                inner: zip,
                entry_builder: |zip| zip.by_index_seek(0),
            }
            .try_build()?;

            Ok(Self {
                path: path.to_path_buf(),
                position: 0,
                inner: Inner::Zipped(zipped),
            })
        } else {
            let mut files = vec![File::open(path)];

            if filename.contains(".part0.iso") {
                let part1_filename = filename.replace(".part0.iso", ".part1.iso");
                let part1_path = path.with_file_name(part1_filename);

                if !part1_path.exists() {
                    return Err(Error::DiscNotFound);
                }

                files.push(File::open(part1_path))
            } else if ext == "wbfs" {
                for i in 1..=4 {
                    let mut wbfx_filename = filename.to_string();
                    let _ = wbfx_filename.pop();
                    write!(&mut wbfx_filename, "{i}").unwrap();

                    let wbfx_path = path.with_file_name(wbfx_filename);
                    if wbfx_path.exists() {
                        files.push(File::open(wbfx_path));
                    }
                }
            }

            let multi = MultiReader::try_new(files)?;

            Ok(Self {
                path: path.to_path_buf(),
                position: 0,
                inner: Inner::Files(multi),
            })
        }
    }
}

impl Read for DiscFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = match &mut self.inner {
            Inner::Files(p) => p.read(buf)?,
            Inner::Zipped(z) => z.with_entry_mut(|e| e.read(buf))?,
        };

        self.position += n as u64;
        Ok(n)
    }
}

impl Seek for DiscFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match &mut self.inner {
            Inner::Files(p) => p.seek(pos)?,
            Inner::Zipped(z) => z.with_entry_mut(|e| e.seek(pos))?,
        };

        self.position = new_pos;
        Ok(new_pos)
    }
}

impl Clone for DiscFileReader {
    fn clone(&self) -> Self {
        let mut new = Self::new(&self.path).unwrap();
        new.seek(SeekFrom::Start(self.position)).unwrap();
        new
    }
}
