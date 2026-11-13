// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

//! RAR and 7z archives, read with an external 7-Zip (`7zz`, `7z` or `7za`). The unrar library's
//! license doesn't fit GPL, so nothing that reads RAR is linked into the app; without 7-Zip these
//! archives are listed by their names only.

use std::{
    env,
    ffi::OsString,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
};

/// A file or folder in an archive
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// the path as 7-Zip names it, which is what it takes back to extract the file
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

impl Entry {
    /// The path with `/` between folders on every system
    pub fn slash_path(&self) -> String {
        self.path.replace('\\', "/")
    }
}

/// Program names of 7-Zip, best first: `7zz` is 7-Zip itself (Homebrew's `sevenzip`, the
/// official Linux and macOS builds), `7z` p7zip or 7-Zip on Linux and Windows, `7za` p7zip's
/// standalone one without RAR (it still reads 7z)
const PROGRAMS: [&str; 3] = ["7zz", "7z", "7za"];

/// Folders to look in besides `PATH`: an app started from the Finder doesn't get the shell's
/// `PATH`, so Homebrew's folders aren't in it
#[cfg(target_os = "macos")]
const EXTRA_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/opt/local/bin"];
#[cfg(windows)]
const EXTRA_DIRS: &[&str] = &[r"C:\Program Files\7-Zip", r"C:\Program Files (x86)\7-Zip"];
#[cfg(not(any(target_os = "macos", windows)))]
const EXTRA_DIRS: &[&str] = &[];

/// The 7-Zip program found on this computer, looked for once
pub fn program() -> Option<&'static Path> {
    static PROGRAM: OnceLock<Option<PathBuf>> = OnceLock::new();
    PROGRAM
        .get_or_init(|| {
            let path = env::var_os("PATH").unwrap_or_default();
            let dirs: Vec<PathBuf> = env::split_paths(&path)
                .chain(EXTRA_DIRS.iter().map(PathBuf::from))
                .collect();
            find_program(&dirs)
        })
        .as_deref()
}

fn find_program(dirs: &[PathBuf]) -> Option<PathBuf> {
    PROGRAMS.iter().find_map(|name| {
        let mut file = OsString::from(name);
        if cfg!(windows) {
            file.push(".exe");
        }
        dirs.iter().map(|dir| dir.join(&file)).find(|p| p.is_file())
    })
}

fn command(program: &Path) -> Command {
    let mut command = Command::new(program);
    // no prompt can wait for an answer (a password): it fails at once instead
    command
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .arg("-sccUTF-8");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// What is in an archive, from its headers (nothing is unpacked). None without 7-Zip, or when the
/// archive can't be read (damaged, or its names are encrypted).
pub fn list(archive: &Path) -> Option<Vec<Entry>> {
    let output = command(program()?)
        .args(["l", "-slt", "-ba", "--"])
        .arg(archive)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_listing(&String::from_utf8_lossy(&output.stdout)))
}

/// The entries of `7z l -slt -ba`: blocks of `Key = value` lines, one block per entry
fn parse_listing(listing: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut current: Option<Entry> = None;

    for line in listing.lines() {
        let Some((key, value)) = line.split_once(" = ").or_else(|| {
            // an empty value: "CRC = " lost its trailing space to nothing, or kept it
            line.strip_suffix(" =").map(|key| (key, ""))
        }) else {
            continue;
        };
        match key {
            "Path" => {
                entries.extend(current.take());
                current = Some(Entry {
                    path: value.to_string(),
                    size: 0,
                    is_dir: false,
                });
            }
            "Size" => {
                if let Some(entry) = &mut current {
                    entry.size = value.parse().unwrap_or(0);
                }
            }
            "Folder" => {
                if let Some(entry) = &mut current {
                    entry.is_dir |= value == "+";
                }
            }
            "Attributes" => {
                if let Some(entry) = &mut current {
                    entry.is_dir |= value.starts_with('D');
                }
            }
            _ => {}
        }
    }
    entries.extend(current);
    entries
}

/// The data of the files `members` in the archive (paths as [`list`] gave them), one after the
/// other in the archive's order; None when 7-Zip fails or the data is longer than `max_len`
pub fn read(archive: &Path, members: &[&str], max_len: u64) -> Option<Vec<u8>> {
    if members.is_empty() {
        return Some(Vec::new());
    }
    // -spd: a name is a name, even with * or ? in it
    let mut child = command(program()?)
        .args(["e", "-so", "-spd", "--"])
        .arg(archive)
        .args(members)
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;

    let mut data = Vec::new();
    let read = child
        .stdout
        .take()?
        .take(max_len + 1)
        .read_to_end(&mut data);
    if read.is_err() || data.len() as u64 > max_len {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    child.wait().ok()?.success().then_some(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn a_listing_is_read_into_entries() {
        let listing = "Path = 0005000010106001\n\
                       Folder = +\n\
                       Size = 0\n\
                       \n\
                       Path = 0005000010106001/meta/meta.xml\n\
                       Folder = -\n\
                       Size = 9526\n\
                       CRC = 3610A686\n\
                       \n\
                       Path = a\n\
                       Size = 0\n\
                       Attributes = D drwxr-xr-x\n\
                       CRC =\n\
                       \n\
                       Path = Pokemon GS Chronicles 2.7.1.gba\n\
                       Size = 33554432\n\
                       Attributes = A -rw-r--r--\n";
        let entries = parse_listing(listing);
        let summary: Vec<(&str, u64, bool)> = entries
            .iter()
            .map(|e| (e.path.as_str(), e.size, e.is_dir))
            .collect();
        assert_eq!(
            summary,
            [
                ("0005000010106001", 0, true),
                ("0005000010106001/meta/meta.xml", 9526, false),
                ("a", 0, true),
                ("Pokemon GS Chronicles 2.7.1.gba", 33_554_432, false),
            ]
        );
    }

    #[test]
    fn the_first_7zip_program_in_the_folders_is_found() {
        let dir = tempfile::tempdir().unwrap();
        let (first, second) = (dir.path().join("a"), dir.path().join("b"));
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        let exe = if cfg!(windows) { ".exe" } else { "" };
        fs::write(second.join(format!("7zz{exe}")), b"").unwrap();
        fs::write(first.join(format!("7za{exe}")), b"").unwrap();

        // 7zz is better than 7za, wherever it is
        assert_eq!(
            find_program(&[first.clone(), second.clone()]),
            Some(second.join(format!("7zz{exe}")))
        );
        assert_eq!(
            find_program(std::slice::from_ref(&first)),
            Some(first.join(format!("7za{exe}")))
        );
        assert_eq!(find_program(&[dir.path().to_path_buf()]), None);
    }

    /// Packs and reads an archive with the 7-Zip on this computer; skipped without one
    #[test]
    fn files_are_listed_and_read_with_7zip() {
        let Some(program) = program() else {
            eprintln!("no 7-Zip here: skipped");
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(src.join("title/meta")).unwrap();
        fs::write(src.join("title/meta/meta.xml"), b"<menu/>").unwrap();
        // a name 7-Zip would take as a wildcard
        fs::write(src.join("[weird]*.gba"), b"ROM").unwrap();
        let archive = dir.path().join("test.7z");
        let packed = Command::new(program)
            .current_dir(&src)
            .args(["a", "-bd"])
            .arg(&archive)
            .arg(".")
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(packed.success());

        let entries = list(&archive).unwrap();
        let files: Vec<String> = entries
            .iter()
            .filter(|e| !e.is_dir)
            .map(Entry::slash_path)
            .collect();
        assert!(files.contains(&"title/meta/meta.xml".to_string()));
        assert!(files.contains(&"[weird]*.gba".to_string()));

        let rom = entries.iter().find(|e| e.path.ends_with(".gba")).unwrap();
        assert_eq!(read(&archive, &[&rom.path], 100).unwrap(), b"ROM");
        assert_eq!(read(&archive, &[&rom.path], 2), None);
        assert_eq!(list(&dir.path().join("missing.7z")), None);
    }
}
