// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use std::{
    borrow::Cow,
    io::{Seek, Write},
};
use tempfile::tempfile;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

fn is_app_main_file(filename: &Cow<'_, str>) -> bool {
    filename.ends_with("boot.dol") || filename.ends_with("boot.elf")
}

fn find_app_main_file(
    archive: &mut ZipArchive<impl std::io::Read + std::io::Seek>,
) -> Option<String> {
    archive
        .file_names()
        .filter_map(Result::ok)
        .find(is_app_main_file)
        .map(Cow::into_owned)
}

pub fn rebuild_zip(in_file: std::fs::File) -> Result<(std::fs::File, Vec<String>)> {
    let mut archive = ZipArchive::new(in_file)?;

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    let app_main_file = find_app_main_file(&mut archive).context("Failed to find app main file")?;
    let (app_dir, _) = app_main_file
        .rsplit_once('/')
        .context("Failed to get app dir")?;

    let (_, app_name) = app_dir.rsplit_once('/').context("Failed to get app name")?;

    let mut writer = ZipWriter::new(tempfile()?);
    let mut excluded_files = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let mut f = archive.by_index(i)?;
        let filename = f.name()?;

        if let Some(stripped_filename) = filename.strip_prefix(&app_dir) {
            let new_filename = format!("{app_name}{stripped_filename}");

            if f.is_file() {
                writer.start_file(new_filename, options.clone())?;
                std::io::copy(&mut f, &mut writer)?;
            } else if f.is_dir() {
                writer.add_directory(new_filename, options.clone())?;
            }
        } else {
            excluded_files.push(filename.into_owned());
        }
    }

    let mut out_file = writer.finish()?;
    out_file.flush()?;
    out_file.rewind()?;

    Ok((out_file, excluded_files))
}
