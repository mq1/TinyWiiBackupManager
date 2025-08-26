use log::{debug, info};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// The fixed split size for output files: 4 GiB - 32 KiB.
const SPLIT_SIZE: u64 = (4 * 1024 * 1024 * 1024) - (32 * 1024);

/// Manages writing data across multiple split files.
/// Automatically cleans up files on drop if finalize() was not called.
pub struct SplitWriter {
    base_path: PathBuf,
    files: Vec<Option<File>>,
    total_written: u64,
    finalized: bool,
}

impl SplitWriter {
    /// Creates a new `SplitWriter`.
    /// The `base_path` should include the desired extension (e.g., "game.wbfs" or "game.iso").
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
            files: Vec::new(),
            total_written: 0,
            finalized: false,
        }
    }

    /// Generates the filename for a given split index. This is an internal helper.
    /// For WBFS: index 0 -> .wbfs, index 1 -> .wbf1, ...
    /// For others: index 0 -> .ext, index 1 -> .ext.1, ...
    fn get_filename(&self, index: usize) -> PathBuf {
        if index == 0 {
            self.base_path.clone()
        } else {
            // Get the extension from the base path
            let ext = self
                .base_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if ext == "wbfs" {
                // Special case for WBFS: .wbf1, .wbf2, etc.
                let base_without_ext = self.base_path.with_extension("");
                base_without_ext.with_extension(format!("wbf{}", index))
            } else {
                // For ISO and others: .iso.1, .iso.2, etc.
                let mut path = self.base_path.clone();
                let filename = path.file_name().unwrap().to_string_lossy();
                let new_filename = format!("{}.{}", filename, index);
                path.set_file_name(new_filename);
                path
            }
        }
    }

    /// Writes a buffer of data sequentially.
    pub fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        while !buf.is_empty() {
            let split_index = (self.total_written / SPLIT_SIZE) as usize;
            let offset_in_split = self.total_written % SPLIT_SIZE;

            let file = self.get_file(split_index)?;

            let bytes_to_write = (SPLIT_SIZE - offset_in_split).min(buf.len() as u64) as usize;
            file.write_all(&buf[..bytes_to_write])?;

            buf = &buf[bytes_to_write..];
            self.total_written += bytes_to_write as u64;
        }
        Ok(())
    }

    /// Writes a buffer of data at a specific absolute offset.
    pub fn write_all_at(&mut self, offset: u64, buf: &[u8]) -> io::Result<()> {
        let split_index = (offset / SPLIT_SIZE) as usize;
        let offset_in_split = offset % SPLIT_SIZE;

        let file = self.get_file(split_index)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileExt;
            file.write_all_at(buf, offset_in_split)
        }
        #[cfg(not(unix))]
        {
            use std::io::{Read, Seek, SeekFrom};
            file.seek(SeekFrom::Start(offset_in_split))?;
            file.write_all(buf)
        }
    }

    /// Opens (or gets a handle to) the file for a given split index.
    fn get_file(&mut self, index: usize) -> io::Result<&mut File> {
        if index >= self.files.len() {
            self.files.resize_with(index + 1, || None);
        }

        if self.files[index].is_none() {
            let filename = self.get_filename(index);
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(filename)?;
            self.files[index] = Some(file);
        }

        Ok(self.files[index].as_mut().unwrap())
    }

    /// Finalizes the write operation and marks files as complete.
    /// Removes any unused split files that might exist from previous runs.
    pub fn finalize(&mut self) -> io::Result<()> {
        // Flush all open files
        for file in self.files.iter_mut().flatten() {
            file.flush()?;
        }

        // Remove any unused split files from previous runs
        let num_splits_used = self.files.len();
        for i in num_splits_used..3 {
            // Check up to 3 possible split files
            let filename = self.get_filename(i);
            if filename.exists() {
                debug!("Removing unused split file: {}", filename.display());
                fs::remove_file(filename)?;
            } else {
                break; // No more files to check
            }
        }

        self.finalized = true;
        Ok(())
    }

    /// Gets the list of all file paths that have been created.
    fn get_created_files(&self) -> Vec<PathBuf> {
        self.files
            .iter()
            .enumerate()
            .filter_map(|(i, file_opt)| {
                if file_opt.is_some() {
                    Some(self.get_filename(i))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Drop for SplitWriter {
    fn drop(&mut self) {
        // If we weren't finalized, clean up any files we created
        if !self.finalized {
            info!("SplitWriter dropped without finalization - cleaning up files");

            // Get the list of files to remove before clearing the Vec
            let files_to_remove = self.get_created_files();

            // Close all file handles first
            self.files.clear();

            // Remove all created files
            for file_path in files_to_remove {
                if file_path.exists() {
                    debug!("Removing incomplete file: {}", file_path.display());
                    if let Err(e) = fs::remove_file(&file_path) {
                        debug!("Failed to remove file {}: {}", file_path.display(), e);
                    }
                }
            }

            // Try to remove the parent directory if it's empty
            if let Some(parent) = self.base_path.parent()
                && let Ok(mut entries) = fs::read_dir(parent)
                && entries.next().is_none()
            {
                debug!("Removing empty game directory: {}", parent.display());
                let _ = fs::remove_dir(parent);
            }
        }
    }
}
