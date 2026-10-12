use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use crate::{ErrorContext, Result, ResultContext, read::DiscStream};

#[derive(Debug)]
pub struct SplitFileReader {
    files: Vec<Split<PathBuf>>,
    open_file: Option<Split<File>>,
}

#[derive(Debug, Clone)]
struct Split<T> {
    inner: T,
    begin: u64,
    size: u64,
}

impl<T> Split<T> {
    fn contains(&self, pos: u64) -> bool { self.begin <= pos && pos < self.begin + self.size }
}

// .iso.1, .iso.2, etc.
fn split_path_1(input: &Path, index: u32) -> PathBuf {
    let input_str = input.to_str().unwrap_or("[INVALID]");
    let mut out = input_str.to_string();
    out.push('.');
    out.push_str(&index.to_string());
    PathBuf::from(out)
}

// .part1.iso, .part2.iso, etc.
fn split_path_2(input: &Path, index: u32) -> PathBuf {
    let extension = input.extension().and_then(|s| s.to_str()).unwrap_or("iso");
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("[INVALID]");
    let base_stem = stem
        .rsplit_once(".part")
        .filter(|(_, suffix)| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
        .map_or(stem, |(base, _)| base);
    input.with_file_name(format!("{base_stem}.part{index}.{extension}"))
}

// .wbf1, .wbf2, etc.
fn split_path_3(input: &Path, index: u32) -> PathBuf {
    let input_str = input.to_str().unwrap_or("[INVALID]");
    let mut chars = input_str.chars();
    chars.next_back();
    let mut out = chars.as_str().to_string();
    out.push_str(&index.to_string());
    PathBuf::from(out)
}

impl SplitFileReader {
    pub fn empty() -> Self { Self { files: Vec::new(), open_file: Default::default() } }

    pub fn new(path: &Path) -> Result<Self> {
        let mut files = vec![];
        let mut begin = 0;
        match path.metadata() {
            Ok(metadata) => {
                files.push(Split { inner: path.to_path_buf(), begin, size: metadata.len() });
                begin += metadata.len();
            }
            Err(e) => {
                return Err(e.context(format!("Failed to stat file {}", path.display())));
            }
        }
        for path_fn in [split_path_1, split_path_2, split_path_3] {
            let mut index = 1;
            let mut found_split = false;
            loop {
                let path = path_fn(path, index);
                if path == files[0].inner {
                    index += 1;
                    continue;
                }
                if let Ok(metadata) = path.metadata() {
                    files.push(Split { inner: path, begin, size: metadata.len() });
                    begin += metadata.len();
                    index += 1;
                    found_split = true;
                } else {
                    break;
                }
            }
            if found_split {
                break;
            }
        }
        Ok(Self { files, open_file: Default::default() })
    }

    pub fn add(&mut self, path: &Path) -> Result<()> {
        let begin = self.len();
        let metadata =
            path.metadata().context(format!("Failed to stat file {}", path.display()))?;
        self.files.push(Split { inner: path.to_path_buf(), begin, size: metadata.len() });
        Ok(())
    }

    pub fn len(&self) -> u64 { self.files.last().map_or(0, |f| f.begin + f.size) }
}

impl Clone for SplitFileReader {
    fn clone(&self) -> Self { Self { files: self.files.clone(), open_file: Default::default() } }
}

impl DiscStream for SplitFileReader {
    fn read_exact_at(&mut self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            let split = if self.open_file.as_ref().is_none_or(|s| !s.contains(offset)) {
                let split = if let Some(split) = self.files.iter().find(|f| f.contains(offset)) {
                    let file = File::open(&split.inner)?;
                    Split { inner: file, begin: split.begin, size: split.size }
                } else {
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                };
                self.open_file.insert(split)
            } else {
                self.open_file.as_mut().unwrap()
            };
            let file_offset = offset - split.begin;
            let len = (split.size - file_offset).min(buf.len() as u64) as usize;
            let (out, remain) = buf.split_at_mut(len);
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileExt;
                split.inner.read_exact_at(out, file_offset)?;
            }
            #[cfg(not(unix))]
            {
                use std::io::{Read, Seek, SeekFrom};
                split.inner.seek(SeekFrom::Start(file_offset))?;
                split.inner.read_exact(out)?
            }
            buf = remain;
            offset += len as u64;
        }
        Ok(())
    }

    fn stream_len(&mut self) -> io::Result<u64> { Ok(self.len()) }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            path.push(format!("nod-split-test-{}-{nanos}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path { &self.0 }
    }

    impl Drop for TestDir {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn part_zero_input_builds_next_part_paths() {
        let input = Path::new("/tmp/game.part0.iso");

        assert_eq!(split_path_2(input, 1), PathBuf::from("/tmp/game.part1.iso"));
        assert_eq!(split_path_2(input, 10), PathBuf::from("/tmp/game.part10.iso"));
    }

    #[test]
    fn reads_part_zero_sequence_across_multiple_parts() {
        let dir = TestDir::new();
        for index in 0..=10 {
            let path = dir.path().join(format!("game.part{index}.iso"));
            fs::write(path, [index as u8]).unwrap();
        }

        let input = dir.path().join("game.part0.iso");
        let mut reader = SplitFileReader::new(&input).unwrap();
        let mut buf = [0; 11];
        reader.read_exact_at(&mut buf, 0).unwrap();

        assert_eq!(reader.len(), 11);
        assert_eq!(buf, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn numbered_input_is_not_added_twice() {
        let dir = TestDir::new();
        let input = dir.path().join("game.part1.iso");
        fs::write(&input, [1]).unwrap();

        let reader = SplitFileReader::new(&input).unwrap();

        assert_eq!(reader.len(), 1);
        assert_eq!(reader.files.len(), 1);
    }
}
