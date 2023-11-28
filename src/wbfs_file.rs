// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use std::ffi::CString;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::raw::c_char;
use std::path::Path;

use anyhow::{bail, Result};

extern "C" {
    fn conv_to_wbfs(filename: *const c_char, dest_dir: *const c_char);
}

fn get_id_and_title(file: &mut File) -> Result<(String, String)> {
    // read the id
    let mut id = [0u8; 0x6];
    file.read_exact(&mut id)?;
    let id = String::from_utf8(id.to_vec())?;

    // read the title
    file.seek(SeekFrom::Current(0x1a))?;
    let mut title = [0u8; 0x40];
    file.read_exact(&mut title)?;
    let title = String::from_utf8(title.to_vec())?;
    let title = title.trim_matches(char::from(0)).to_string();

    Ok((id, title))
}

pub fn conv_to_wbfs_wrapper(src: &Path, dest: &Path) -> Result<()> {
    let mut file = File::open(src)?;
    let (id, title) = get_id_and_title(&mut file)?;

    let dest_dir = Path::new(dest).join(format!("{} [{}]", title, id));
    if dest_dir.exists() {
        println!("Skipping {}", dest_dir.display());
        return Ok(());
    }

    let src = CString::new(src.to_string_lossy().to_string())?;
    let dest = CString::new(dest.to_string_lossy().to_string())?;

    unsafe {
        conv_to_wbfs(src.as_ptr(), dest.as_ptr());
    };

    Ok(())
}

pub fn copy_wbfs_file(src: &Path, dest: &Path) -> Result<()> {
    let mut file = File::open(src)?;

    // check if the file is a wbfs file
    {
        let mut magic = [0u8; 0x4];
        file.read_exact(&mut magic)?;
        if magic != [0x57, 0x42, 0x46, 0x53] {
            bail!("Invalid wbfs file");
        }
    }

    file.seek(SeekFrom::Start(0x200))?;
    let (id, title) = get_id_and_title(&mut file)?;
    let dest_dir = dest.join(format!("{} [{}]", title, id));

    if dest_dir.exists() {
        println!("Skipping {}", dest_dir.display());
        return Ok(());
    }

    fs::create_dir(&dest_dir)?;
    let dest_file = dest_dir.join(&id).with_extension("wbfs");
    fs::copy(src, dest_file)?;

    // copy eventual wbf[1-3] files
    for i in 1..=3 {
        let extension = format!("wbf{}", i);

        let src = src.with_extension(&extension);
        if src.exists() {
            let dest_file = dest_dir.join(&id).with_extension(&extension);
            std::fs::copy(src, dest_file)?;
        }
    }

    Ok(())
}
