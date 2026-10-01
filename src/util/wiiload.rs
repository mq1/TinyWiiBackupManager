// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, bail};
use tempfile::tempfile;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

fn get_app_files(
    archive: &mut ZipArchive<impl std::io::Read + std::io::Seek>,
) -> Result<(String, Vec<String>, Vec<String>)> {
    let Some(app_filename) = archive
        .file_names()
        .find(|f| f.ends_with("boot.dol") || f.ends_with("boot.elf"))
    else {
        bail!("Failed to find app binary");
    };

    eprintln!("App filename: {app_filename}");

    let parent_filename = app_filename[0..app_filename.len() - 8].to_string();
    eprintln!("Parent filename: {parent_filename}");

    let mut app_files = Vec::new();
    let mut excluded_files = Vec::new();
    for filename in archive.file_names() {
        if filename.starts_with(&parent_filename) {
            app_files.push(filename.to_string());
        } else {
            excluded_files.push(filename.to_string());
        }
    }

    Ok((parent_filename, app_files, excluded_files))
}

pub fn rebuild_zip(in_file: std::fs::File) -> Result<(std::fs::File, Vec<String>)> {
    let mut archive = ZipArchive::new(in_file)?;
    let (parent_filename, app_files, excluded_files) = get_app_files(&mut archive)?;

    let Some(app_name) = parent_filename[0..parent_filename.len() - 1]
        .split('/')
        .next_back()
    else {
        bail!("Failed to get app name")
    };
    eprintln!("App name: {app_name}");

    let mut writer = ZipWriter::new(tempfile()?);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    let new_parent_filename = format!("{app_name}/");
    for filename in &app_files {
        let new_filename = filename.replace(&parent_filename, &new_parent_filename);
        writer.start_file(&new_filename, options)?;
        let mut file = archive.by_name(filename)?;
        std::io::copy(&mut file, &mut writer)?;
        eprintln!("Copied {filename} to {new_filename}");
    }

    let out_file = writer.finish()?;

    Ok((out_file, excluded_files))
}
