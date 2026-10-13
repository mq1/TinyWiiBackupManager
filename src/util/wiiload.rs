// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, ensure};
use tempfile::tempfile;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

fn get_app_files(
    archive: &mut ZipArchive<impl std::io::Read + std::io::Seek>,
) -> Result<(String, Vec<String>, Vec<String>)> {
    let mut filenames = Vec::with_capacity(archive.len());
    for filename in archive.file_names() {
        let filename = filename?;
        let path = filename.strip_suffix('/').unwrap_or(&filename);
        ensure!(
            !path.is_empty()
                && !path.contains(['\\', '\0'])
                && path.split('/').all(|part| !part.is_empty()
                    && part != "."
                    && part != ".."
                    && !part.contains(':')),
            "Unsafe archive member path: {filename}"
        );
        filenames.push(filename.into_owned());
    }
    let app_filename = filenames
        .iter()
        .find(|filename| matches!(filename.rsplit('/').next(), Some("boot.dol" | "boot.elf")))
        .context("Failed to find app binary")?;

    eprintln!("App filename: {app_filename}");

    let (parent, _) = app_filename.rsplit_once('/').context(
        "The ZIP must contain an app directory with boot.dol or boot.elf; a root-level binary cannot identify the app directory",
    )?;
    let parent_filename = format!("{parent}/");
    eprintln!("Parent filename: {parent_filename}");

    let mut app_files = Vec::with_capacity(archive.len());
    let mut excluded_files = Vec::with_capacity(archive.len());
    for filename in filenames {
        if filename.starts_with(&parent_filename) {
            app_files.push(filename);
        } else {
            excluded_files.push(filename);
        }
    }

    Ok((parent_filename, app_files, excluded_files))
}

pub fn rebuild_zip(in_file: std::fs::File) -> Result<(std::fs::File, Vec<String>)> {
    let mut archive = ZipArchive::new(in_file)?;
    let (parent_filename, app_files, excluded_files) = get_app_files(&mut archive)?;

    let app_name = parent_filename
        .strip_suffix('/')
        .and_then(|parent| parent.rsplit('/').next())
        .context("Failed to get app name")?;

    eprintln!("App name: {app_name}");

    let mut writer = ZipWriter::new(tempfile()?);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    let new_parent_filename = format!("{app_name}/");
    for filename in &app_files {
        let suffix = filename
            .strip_prefix(&parent_filename)
            .context("Invalid app directory prefix")?;
        let new_filename = format!("{new_parent_filename}{suffix}");
        let mut file = archive.by_name(filename)?;
        ensure!(
            file.unix_mode()
                .is_none_or(|mode| mode & 0o170000 != 0o120000),
            "Symbolic links are not supported in app archives"
        );
        if file.is_dir() {
            writer.add_directory(&new_filename, options)?;
            continue;
        }
        writer.start_file(&new_filename, options)?;
        std::io::copy(&mut file, &mut writer)?;
        eprintln!("Copied {filename} to {new_filename}");
    }

    let out_file = writer.finish()?;

    Ok((out_file, excluded_files))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn archive(entries: &[(&str, &[u8])]) -> std::fs::File {
        let mut writer = ZipWriter::new(tempfile().unwrap());
        for (name, bytes) in entries {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap()
    }

    #[test]
    fn archive_root_boot_returns_a_clear_error_without_panicking() {
        let input = archive(&[("boot.dol", b"binary"), ("meta.xml", b"metadata")]);
        let error = rebuild_zip(input).unwrap_err();
        assert!(error.to_string().contains("app directory"), "{error:#}");
    }

    #[test]
    fn rebuilding_zip_only_replaces_the_leading_app_prefix() {
        let input = archive(&[
            ("apps/demo/boot.dol", b"binary"),
            ("apps/demo/sub/apps/demo/icon.png", b"nested asset"),
            ("outside.txt", b"excluded"),
        ]);
        let (output, excluded) = rebuild_zip(input).unwrap();
        assert_eq!(excluded, ["outside.txt"]);
        let mut archive = ZipArchive::new(output).unwrap();
        let mut asset = Vec::new();
        archive
            .by_name("demo/sub/apps/demo/icon.png")
            .unwrap()
            .read_to_end(&mut asset)
            .unwrap();
        assert_eq!(asset, b"nested asset");
    }

    #[test]
    fn rebuilding_zip_rejects_unsafe_member_paths() {
        for unsafe_name in [
            "apps/demo/../outside.txt",
            "apps/demo/sub/../../outside.txt",
            "/apps/demo/absolute.txt",
            "C:/apps/demo/drive.txt",
            "apps/demo/back\\slash.txt",
        ] {
            let input = archive(&[("apps/demo/boot.dol", b"binary"), (unsafe_name, b"unsafe")]);
            assert!(
                rebuild_zip(input).is_err(),
                "accepted unsafe archive member {unsafe_name}"
            );
        }
    }
}
