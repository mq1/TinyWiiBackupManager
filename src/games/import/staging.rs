// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::{CleanupWarning, DiscCancellation, ImportCancellation};
use anyhow::{Context, Result, bail};
use std::{
    fs::{File, OpenOptions},
    io::{self, Seek, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use tempfile::{TempDir, TempPath};

/// Parts live in a private directory on the destination filesystem until every
/// part and the final header have been flushed and synchronized.
pub(super) struct StagedWriter<F> {
    // Files must close before TempDir is dropped, especially on Windows.
    files: Vec<(PathBuf, File)>,
    directory: TempDir,
    filename: F,
    split_size: Option<NonZeroUsize>,
    offset: usize,
    cancel: ImportCancellation,
}

impl<F: Fn(usize) -> String> StagedWriter<F> {
    pub(super) fn new(
        parent: &Path,
        filename: F,
        split_size: Option<NonZeroUsize>,
        cancel: ImportCancellation,
    ) -> Result<Self> {
        cancel.check()?;
        let directory = tempfile::Builder::new()
            .prefix(".twbm-import-")
            .tempdir_in(parent)?;
        let mut writer = Self {
            directory,
            files: Vec::new(),
            filename,
            split_size,
            offset: 0,
            cancel,
        };
        if let Err(error) = writer.next_part() {
            return Err(writer.discard(error.into()));
        }
        Ok(writer)
    }

    fn next_part(&mut self) -> io::Result<()> {
        self.cancel.check()?;
        let path = self
            .directory
            .path()
            .join((self.filename)(self.files.len()));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        self.files.push((path, file));
        self.offset = 0;
        self.cancel.check()
    }

    pub(super) fn discard(mut self, error: anyhow::Error) -> anyhow::Error {
        self.files.clear();
        match self.directory.close() {
            Ok(()) => error,
            Err(cleanup) => error.context(CleanupWarning(format!(
                "Temporary folder cleanup failed: {cleanup}"
            ))),
        }
    }

    pub(super) fn finish(self, header: &[u8]) -> Result<StagedDisc> {
        self.finish_with_check(header, || Ok(()))
    }

    pub(super) fn finish_with_check(
        mut self,
        mut header: &[u8],
        mut checkpoint: impl FnMut() -> Result<()>,
    ) -> Result<StagedDisc> {
        let check = &mut || -> Result<()> {
            checkpoint()?;
            self.cancel.check()?;
            Ok(())
        };
        let result = (|| -> Result<()> {
            check()?;
            // Rewrite the logical beginning, not just the first physical part.
            for (_, file) in &mut self.files {
                check()?;
                let count = header
                    .len()
                    .min(usize::try_from(file.metadata()?.len()).unwrap_or(usize::MAX));
                if count > 0 {
                    file.rewind()?;
                    for chunk in header[..count].chunks(1024 * 1024) {
                        check()?;
                        file.write_all(chunk)?;
                        check()?;
                    }
                    header = &header[count..];
                }
                check()?;
                file.flush()?;
                check()?;
                file.sync_all()?;
                check()?;
            }
            if !header.is_empty() {
                bail!("Final header exceeds the staged disc length");
            }
            Ok(())
        })();
        if let Err(error) = result {
            return Err(self.discard(error));
        }
        let parts = self
            .files
            .into_iter()
            .map(|(path, file)| {
                drop(file);
                path
            })
            .collect();
        Ok(StagedDisc {
            directory: self.directory,
            parts,
        })
    }
}

impl<F: Fn(usize) -> String> Write for StagedWriter<F> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.cancel.check()?;
        if buffer.is_empty() {
            return Ok(0);
        }
        if self
            .split_size
            .is_some_and(|size| self.offset == size.get())
        {
            self.next_part()?;
        }
        let count = self
            .split_size
            .map_or(buffer.len(), |size| {
                buffer.len().min(size.get() - self.offset)
            })
            .min(1024 * 1024);
        let written = self
            .files
            .last_mut()
            .expect("first part exists")
            .1
            .write(&buffer[..count])?;
        if self.split_size.is_some() {
            self.offset += written;
        }
        self.cancel.check()?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        for (_, file) in &mut self.files {
            self.cancel.check()?;
            file.flush()?;
            self.cancel.check()?;
        }
        Ok(())
    }
}

pub(super) struct StagedDisc {
    directory: TempDir,
    parts: Vec<PathBuf>,
}

impl StagedDisc {
    /// The caller serializes commits and checks the complete disc family first.
    /// Publish secondary parts before the primary file that loaders recognize.
    /// If any no-clobber move fails, roll back only files created by this commit.
    pub(super) fn publish(
        self,
        destination: &Path,
        cancel: &ImportCancellation,
    ) -> Result<Option<String>> {
        self.publish_with_check(destination, |_| {
            cancel.check()?;
            Ok(())
        })
    }

    pub(super) fn discard(self, error: anyhow::Error) -> anyhow::Error {
        match self.directory.close() {
            Ok(()) => error,
            Err(cleanup) => error.context(CleanupWarning(format!(
                "Temporary folder cleanup failed: {cleanup}"
            ))),
        }
    }

    pub(super) fn publish_with_check(
        self,
        destination: &Path,
        mut check: impl FnMut(usize) -> Result<()>,
    ) -> Result<Option<String>> {
        if let Err(error) = check(0) {
            return Err(self.discard(error));
        }
        let created = match std::fs::create_dir(destination) {
            Ok(()) => true,
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists && destination.is_dir() => {
                false
            }
            Err(error) => {
                return Err(
                    self.discard(anyhow::Error::new(error).context("Cannot create game folder"))
                );
            }
        };
        let mut published = Vec::new();
        let result: Result<()> = (|| {
            for source in self.parts.iter().skip(1).chain(self.parts.first()) {
                check(published.len())?;
                let target =
                    destination.join(source.file_name().context("Invalid staged filename")?);
                TempPath::try_from_path(source)?
                    .persist_noclobber(&target)
                    .map_err(|error| anyhow::Error::new(error.error))
                    .with_context(|| {
                        format!(
                            "Cannot publish {}; existing files are never replaced",
                            target.display()
                        )
                    })?;
                published.push(target);
            }
            // The primary-file rename commits the complete image. Never turn a
            // late cancellation into failure or remove an already committed image.
            Ok(())
        })();
        let result = match result {
            Ok(()) => Ok(()),
            Err(mut error) => {
                for path in published.into_iter().rev() {
                    if let Err(cleanup) = std::fs::remove_file(&path) {
                        error = error.context(CleanupWarning(format!(
                            "Could not remove incomplete part {}: {cleanup}",
                            path.display()
                        )));
                    }
                }
                if created && let Err(cleanup) = std::fs::remove_dir(destination) {
                    // Never remove another disc's files or recurse through a game folder.
                    if cleanup.kind() != io::ErrorKind::DirectoryNotEmpty {
                        error = error.context(CleanupWarning(format!(
                            "Could not remove empty game folder: {cleanup}"
                        )));
                    }
                }
                Err(error)
            }
        };
        match (result, self.directory.close()) {
            (result, Ok(())) => result.map(|()| None),
            (Ok(()), Err(error)) => Ok(Some(format!(
                "Disc imported, but temporary folder cleanup failed: {error}"
            ))),
            (Err(error), Err(cleanup)) => Err(error).context(CleanupWarning(format!(
                "Temporary folder cleanup failed: {cleanup}"
            ))),
        }
    }
}
