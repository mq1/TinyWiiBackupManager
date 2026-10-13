// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, ensure};
use std::io::{Seek, Write};
use tempfile::tempfile;
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

fn is_app_main_file(filename: &str) -> bool {
    matches!(filename.rsplit('/').next(), Some("boot.dol" | "boot.elf"))
}

fn find_app_main_file(
    archive: &mut ZipArchive<impl std::io::Read + std::io::Seek>,
) -> Result<String> {
    let mut app_main_file = None;
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
        if app_main_file.is_none() && is_app_main_file(&filename) {
            app_main_file = Some(filename.into_owned());
        }
    }
    app_main_file.context("Failed to find app main file")
}

pub fn rebuild_zip(in_file: std::fs::File) -> Result<(std::fs::File, Vec<String>)> {
    let mut archive = ZipArchive::new(in_file)?;

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(9));

    let app_main_file = find_app_main_file(&mut archive)?;
    let (app_dir, _) = app_main_file.rsplit_once('/').context(
        "The ZIP must contain an app directory with boot.dol or boot.elf; a root-level binary cannot identify the app directory",
    )?;
    let app_name = app_dir
        .rsplit('/')
        .next()
        .context("Failed to get app name")?;
    let app_prefix = format!("{app_dir}/");

    let mut writer = ZipWriter::new(tempfile()?);
    let mut excluded_files = Vec::with_capacity(archive.len());

    for i in 0..archive.len() {
        let mut f = archive.by_index(i)?;
        let filename = f.name()?;

        if let Some(stripped_filename) = filename.strip_prefix(&app_prefix) {
            let new_filename = format!("{app_name}/{stripped_filename}");
            ensure!(
                f.unix_mode().is_none_or(|mode| mode & 0o170000 != 0o120000),
                "Symbolic links are not supported in app archives"
            );

            if f.is_file() {
                writer.start_file(new_filename, options)?;
                std::io::copy(&mut f, &mut writer)?;
            } else if f.is_dir() {
                writer.add_directory(new_filename, options)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};

    fn archive(entries: &[(&str, &[u8])]) -> std::fs::File {
        let mut writer = ZipWriter::new(tempfile().unwrap());
        for (name, bytes) in entries {
            if name.ends_with('/') {
                writer
                    .add_directory(*name, SimpleFileOptions::default())
                    .unwrap();
            } else {
                writer
                    .start_file(*name, SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(bytes).unwrap();
            }
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
            ("apps/other/reboot.dol", b"not the app binary"),
            ("apps/demo/boot.dol", b"binary"),
            ("apps/demo/sub/", b""),
            ("apps/demo/sub/apps/demo/icon.png", b"nested asset"),
            ("apps/demo-other/asset.txt", b"sibling directory"),
            ("outside.txt", b"excluded"),
        ]);
        let (output, excluded) = rebuild_zip(input).unwrap();
        assert_eq!(
            excluded,
            [
                "apps/other/reboot.dol",
                "apps/demo-other/asset.txt",
                "outside.txt"
            ]
        );
        let mut archive = ZipArchive::new(output).unwrap();
        assert!(archive.by_name("demo/sub/").unwrap().is_dir());
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

    #[test]
    fn rebuilt_zip_is_ready_to_stream_from_the_beginning() {
        for main_file in ["apps/demo/boot.dol", "demo/boot.elf"] {
            let (mut output, excluded) = rebuild_zip(archive(&[(main_file, b"binary")])).unwrap();
            assert!(excluded.is_empty());
            assert_eq!(output.stream_position().unwrap(), 0);
            let expected_len = output.metadata().unwrap().len();
            let mut bytes = Vec::new();
            assert_eq!(
                std::io::copy(&mut output, &mut bytes).unwrap(),
                expected_len
            );
            let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
            let mut body = Vec::new();
            archive.by_index(0).unwrap().read_to_end(&mut body).unwrap();
            assert_eq!(body, b"binary");
        }
    }

    #[test]
    fn rebuilding_zip_rejects_symbolic_links_in_the_app_directory() {
        let mut writer = ZipWriter::new(tempfile().unwrap());
        writer
            .start_file("apps/demo/boot.dol", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"binary").unwrap();
        writer
            .add_symlink("apps/demo/link", "../outside", SimpleFileOptions::default())
            .unwrap();
        let error = rebuild_zip(writer.finish().unwrap()).unwrap_err();
        assert!(error.to_string().contains("Symbolic links"), "{error:#}");
    }
}
