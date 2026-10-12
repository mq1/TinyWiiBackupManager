// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use std::io::Read;

fn disc() -> Vec<u8> {
    let mut data = vec![0; 3 * 1024 * 1024 + 0x8000];
    data[..6].copy_from_slice(b"GZZE01");
    data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
    for (i, byte) in data[0x8000..].iter_mut().enumerate() {
        *byte = ((i * 17 + i / 257) % 251) as u8;
    }
    data
}

fn options(format: ExportFormat, fast: bool) -> DiscExportOptions {
    DiscExportOptions::batch(
        format,
        ExportThreads {
            preloader: 0,
            processor: 0,
        },
        fast,
    )
}

#[test]
fn fast_export_preserves_exact_output_in_all_six_formats() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.iso");
    let original = disc();
    std::fs::write(&source, &original).unwrap();
    for format in ExportFormat::ALL {
        let normal = temp.path().join(format!("normal.{}", format.extension()));
        let fast = temp.path().join(format!("fast.{}", format.extension()));
        for (path, enabled) in [(&normal, false), (&fast, true)] {
            export_disc(
                &source,
                path,
                options(format, enabled),
                &ExportCancellation::default(),
                |_| Ok(()),
            )
            .unwrap();
        }
        assert_eq!(
            std::fs::read(&fast).unwrap(),
            std::fs::read(&normal).unwrap(),
            "{format}"
        );
        let reader = DiscReader::new(fast, &DiscOptions::default()).unwrap();
        let mut decoded = Vec::new();
        reader
            .take(original.len() as u64)
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded, original, "{format}");
    }
}

#[test]
fn fast_copy_preserves_validated_iso_wbfs_ciso_and_concatenates_split_parts() {
    let temp = tempfile::tempdir().unwrap();
    let original = disc();
    let iso = temp.path().join("original.iso");
    std::fs::write(&iso, &original).unwrap();
    for format in [ExportFormat::Iso, ExportFormat::Wbfs, ExportFormat::Ciso] {
        let source = temp.path().join(format!("source.{}", format.extension()));
        export_disc(
            &iso,
            &source,
            options(format, false),
            &ExportCancellation::default(),
            |_| Ok(()),
        )
        .unwrap();
        let bytes = std::fs::read(&source).unwrap();
        let source = match format {
            ExportFormat::Iso => {
                let first = temp.path().join("split.part0.iso");
                std::fs::write(&first, &bytes[..0x8000]).unwrap();
                std::fs::write(temp.path().join("split.part1.iso"), &bytes[0x8000..]).unwrap();
                first
            }
            ExportFormat::Wbfs => {
                std::fs::write(&source, &bytes[..0x8000]).unwrap();
                std::fs::write(temp.path().join("source.wbf1"), &bytes[0x8000..]).unwrap();
                source
            }
            _ => source,
        };
        let output = temp.path().join(format!("copied.{}", format.extension()));
        export_disc(
            &source,
            &output,
            options(format, true),
            &ExportCancellation::default(),
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(std::fs::read(output).unwrap(), bytes, "{format}");
    }
}

#[test]
fn fast_copy_uses_actual_format_and_rejects_truncated_wbfs() {
    let temp = tempfile::tempdir().unwrap();
    let original = disc();
    let source = temp.path().join("misnamed.wbfs");
    std::fs::write(&source, &original).unwrap();
    let valid = temp.path().join("valid.wbfs");
    export_disc(
        &source,
        &valid,
        options(ExportFormat::Wbfs, true),
        &ExportCancellation::default(),
        |_| Ok(()),
    )
    .unwrap();
    let mut file = File::open(&valid).unwrap();
    assert_eq!(DiscReader::detect(&mut file).unwrap(), Some(Format::Wbfs));
    drop(file);
    let mut bytes = std::fs::read(&valid).unwrap();
    bytes.pop();
    std::fs::write(&valid, bytes).unwrap();
    for fast in [false, true] {
        let output = temp.path().join(format!("bad-{fast}.wbfs"));
        assert!(
            export_disc(
                &valid,
                &output,
                options(ExportFormat::Wbfs, fast),
                &ExportCancellation::default(),
                |_| Ok(())
            )
            .is_err()
        );
        assert!(!output.exists());
    }
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
}

#[test]
fn fast_copy_and_conversion_cancel_before_publication_and_keep_existing_file() {
    for format in [ExportFormat::Iso, ExportFormat::Rvz] {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.iso");
        let original = disc();
        std::fs::write(&source, &original).unwrap();
        let destination = temp.path().join(format!("existing.{}", format.extension()));
        std::fs::write(&destination, b"previous image").unwrap();
        let cancel = ExportCancellation::default();
        let result = export_disc(
            &source,
            &destination,
            DiscExportOptions {
                destination: DestinationMode::ReplaceConfirmed,
                ..options(format, true)
            },
            &cancel,
            |percentage| {
                if percentage > 0 {
                    cancel.cancel();
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(std::fs::read(destination).unwrap(), b"previous image");
        assert_eq!(std::fs::read(source).unwrap(), original);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    }
}
