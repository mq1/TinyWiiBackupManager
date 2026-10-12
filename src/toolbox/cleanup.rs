// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::import::make_game_dir_name,
    toolbox::{ToolboxGroup, ToolboxItem},
};
use anyhow::{Context, Result, bail};
use lucide_icons::Icon;
use smol::{
    fs::{self, DirEntry, File},
    stream::{self, Stream, StreamExt},
};
use std::{
    collections::BTreeSet,
    convert::identity,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

async fn scan_dir_for_discs(
    path: &Path,
) -> Result<impl Stream<Item = (PathBuf, wii_disc_info::Meta)>> {
    async fn try_open_entry(
        entry: Result<DirEntry, std::io::Error>,
    ) -> Result<(PathBuf, wii_disc_info::Meta)> {
        let path = entry?.path();
        let mut f = File::open(&path).await?;
        let meta = wii_disc_info::Meta::read_async(&mut f).await?;
        Ok((path, meta))
    }

    fs::read_dir(path)
        .await
        .map_err(anyhow::Error::from)
        .map(|entries| entries.then(try_open_entry).filter_map(Result::ok))
}

fn is_split_iso(path: &Path) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".part0.iso"))
}

fn make_disc_filename(path: &Path, meta: &wii_disc_info::Meta) -> Result<String> {
    let ext = match meta.format() {
        wii_disc_info::Format::Iso => "iso",
        wii_disc_info::Format::Wbfs => "wbfs",
        wii_disc_info::Format::Ciso => "ciso",
        format => bail!(
            "Cannot normalize {}: unsupported format {format}",
            path.display()
        ),
    };

    Ok(if is_split_iso(path) {
        format!("{}.part0.iso", meta.game_id())
    } else if meta.is_wii() {
        format!("{}.{ext}", meta.game_id())
    } else {
        match meta.disc_number() {
            0 => format!("game.{ext}"),
            n => format!("disc{}.{ext}", u16::from(n) + 1),
        }
    })
}

// Plan every part before moving the primary file. Parts belong beside their
// original primary, including when the game already has its own directory.
async fn disc_moves(
    disc_path: &Path,
    meta: &wii_disc_info::Meta,
    new_disc_path: &Path,
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut moves = Vec::new();
    let split_iso = is_split_iso(disc_path);
    if split_iso || meta.format() == wii_disc_info::Format::Wbfs {
        let parent = disc_path.parent().context("Invalid disc parent")?;
        let siblings = fs::read_dir(parent).await?.collect::<Vec<_>>().await;
        let siblings = siblings.into_iter().collect::<io::Result<Vec<_>>>()?;
        let mut missing_part = false;
        for part in 1..=4 {
            let source = if split_iso {
                disc_path
                    .with_extension("")
                    .with_extension(format!("part{part}.iso"))
            } else {
                disc_path.with_extension(format!("wbf{part}"))
            };
            let source_name = source.file_name().context("Invalid part filename")?;
            let matches = siblings
                .iter()
                .filter(|entry| entry.file_name().eq_ignore_ascii_case(source_name))
                .collect::<Vec<_>>();
            if matches.len() > 1 {
                bail!("Ambiguous split part: {}", source.display());
            }
            let Some(entry) = matches.first() else {
                // A forced split can contain only part0 when the image fits
                // in one part. Only a gap before a later existing part is bad.
                missing_part = true;
                continue;
            };
            if missing_part || part == 4 || !entry.file_type().await?.is_file() {
                bail!(
                    "Invalid or unsupported split layout: {}",
                    disc_path.display()
                );
            }
            let target = if split_iso {
                new_disc_path
                    .with_extension("")
                    .with_extension(format!("part{part}.iso"))
            } else {
                new_disc_path.with_extension(format!("wbf{part}"))
            };
            moves.push((entry.path(), target));
        }
    }
    moves.push((disc_path.to_path_buf(), new_disc_path.to_path_buf()));
    Ok(moves)
}

fn same_path(first: &Path, second: &Path) -> bool {
    first == second || (cfg!(windows) && first.as_os_str().eq_ignore_ascii_case(second.as_os_str()))
}

fn preflight_moves(moves: &[(PathBuf, PathBuf)]) -> Result<()> {
    let mut destinations = BTreeSet::new();
    for (source, target) in moves {
        let metadata = std::fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() {
            bail!("Cannot normalize a symbolic link: {}", source.display());
        }
        let key = if cfg!(windows) {
            target.as_os_str().to_ascii_lowercase()
        } else {
            target.as_os_str().to_os_string()
        };
        if !destinations.insert(key) {
            bail!("Multiple files would be moved to {}", target.display());
        }
        if same_path(source, target) {
            continue;
        }
        match std::fs::symlink_metadata(target) {
            Ok(_) => bail!("Destination already exists: {}", target.display()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn move_without_replacing(source: &Path, target: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        // MoveFile refuses an existing destination, including races after the
        // preflight. Unlike std::fs::rename, it never requests replacement.
        let source = source
            .to_str()
            .ok_or_else(|| io::Error::other("Invalid source path"))?;
        let target = target
            .to_str()
            .ok_or_else(|| io::Error::other("Invalid destination path"))?;
        winsafe::MoveFile(source, target).map_err(io::Error::other)
    }
    #[cfg(not(windows))]
    {
        if std::fs::symlink_metadata(source)?.is_dir() {
            // Reserve a new empty destination atomically. POSIX rename may
            // replace this directory owned by us, but cannot replace a
            // nonempty directory if another process added files meanwhile.
            // This also works on filesystems without renameat2/RENAME_EXCL,
            // where tempfile's hard-link fallback cannot move directories.
            std::fs::create_dir(target)?;
            let result = std::fs::rename(source, target);
            if result.is_err() {
                let _ = std::fs::remove_dir(target);
            }
            return result;
        }
        let mut path = tempfile::TempPath::try_from_path(source)?;
        // These are original files, not temporary outputs. A failed move must
        // never delete the source when the path guard is dropped.
        path.disable_cleanup(true);
        path.persist_noclobber(target).map_err(|error| error.error)
    }
}

fn apply_moves(moves: &[(PathBuf, PathBuf)]) -> Result<()> {
    preflight_moves(moves)?;
    let mut completed = Vec::new();
    for (source, target) in moves {
        if same_path(source, target) {
            continue;
        }
        if let Err(error) = move_without_replacing(source, target) {
            let mut error = anyhow::Error::new(error).context(format!(
                "Cannot move {} to {}",
                source.display(),
                target.display()
            ));
            for (source, target) in completed.into_iter().rev() {
                if let Err(rollback) = move_without_replacing(target, source) {
                    error = error.context(format!(
                        "Could not restore {}; its data remains at {}: {rollback}",
                        source.display(),
                        target.display()
                    ));
                }
            }
            return Err(error);
        }
        completed.push((source, target));
    }
    Ok(())
}

async fn adopt_orphaned_discs(games_dir: &Path) -> Result<()> {
    let entries = scan_dir_for_discs(games_dir).await?;

    let results = entries
        .then(|(path, meta)| async move {
            let new_parent_name = make_game_dir_name(meta.game_id(), meta.game_title());
            let new_disc_filename = make_disc_filename(&path, &meta)?;

            let new_disc_parent = games_dir.join(new_parent_name);
            let new_path = new_disc_parent.join(&new_disc_filename);

            let moves = disc_moves(&path, &meta, &new_path).await?;
            smol::unblock(move || {
                preflight_moves(&moves)?;
                let created = match std::fs::create_dir(&new_disc_parent) {
                    Ok(()) => true,
                    Err(error)
                        if error.kind() == io::ErrorKind::AlreadyExists
                            && new_disc_parent.is_dir() =>
                    {
                        false
                    }
                    Err(error) => return Err(error.into()),
                };
                let result = apply_moves(&moves);
                if result.is_err() && created {
                    // Only remove our empty directory; never recurse through a
                    // folder that may contain retained parts or another game.
                    let _ = std::fs::remove_dir(&new_disc_parent);
                }
                result
            })
            .await
        })
        .collect::<Vec<_>>()
        .await;

    let errors = results
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();

    match errors.is_empty() {
        true => Ok(()),
        false => bail!("{errors:?}"),
    }
}

async fn readopt_parented_discs(games_dir: &Path) -> Result<()> {
    let all_dirs = fs::read_dir(games_dir)
        .await?
        .filter_map(Result::ok)
        .then(|entry| async move {
            entry
                .file_type()
                .await
                .is_ok_and(|ft| {
                    ft.is_dir() && !entry.file_name().to_string_lossy().starts_with('.')
                })
                .then_some(entry.path())
        })
        .filter_map(identity);

    let results = all_dirs
        .then(|path| async move {
            let discs = scan_dir_for_discs(&path).await?.collect::<Vec<_>>().await;

            match &discs[..] {
                [_] => {}
                [(_, first), (_, second)]
                    if !first.is_wii()
                        && !second.is_wii()
                        && first.game_id() == second.game_id()
                        && first.disc_number() != second.disc_number() => {}
                _ => {
                    // ignore unknown layouts
                    return Ok(());
                }
            }

            let mut moves = Vec::new();
            for (disc_path, meta) in &discs {
                let new_disc_path = path.join(make_disc_filename(disc_path, meta)?);
                moves.extend(disc_moves(disc_path, meta, &new_disc_path).await?);
            }
            let (_, meta) = &discs[0];
            let new_dir_name = make_game_dir_name(meta.game_id(), meta.game_title());
            let new_path = path.with_file_name(new_dir_name);

            // Include the directory in the same preflight and rollback plan.
            // Renaming it last also preserves covers and other auxiliary files.
            moves.push((path, new_path));
            smol::unblock(move || apply_moves(&moves)).await
        })
        .collect::<Vec<_>>()
        .await;

    let errors = results
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();

    match errors.is_empty() {
        true => Ok(()),
        false => bail!("{errors:?}"),
    }
}

pub const ALL: &[ToolboxGroup] = {
    &[ToolboxGroup {
        label: "Cleanup",
        icon: Icon::BrushCleaning,
        items: &[ToolboxItem {
            label: "Normalize paths (makes the game directories' layouts consistent)",
            run_fn: |ctx| {
                Box::pin(async move {
                    stream::iter([
                        ctx.config.mount_point().join("wbfs"),
                        ctx.config.mount_point().join("games"),
                    ])
                    .then(|path| async move {
                        if fs::metadata(&path).await.is_ok_and(|meta| meta.is_dir()) {
                            adopt_orphaned_discs(&path).await?;
                            readopt_parented_discs(&path).await?;
                        }

                        Ok(())
                    })
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .map(|_| "Paths successfully normalized".to_string())
                })
            },
        }],
    }]
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, io::Cursor};
    use wii_disc_info::{Format, Meta};

    fn disc(format: Format) -> Vec<u8> {
        let mut bytes = vec![0; 0x8000];
        let offset = match format {
            Format::Iso => 0,
            Format::Wbfs => {
                bytes[..4].copy_from_slice(b"WBFS");
                0x200
            }
            Format::Rvz => {
                bytes[..4].copy_from_slice(b"RVZ\x01");
                0x58
            }
            _ => unreachable!(),
        };
        bytes[offset..offset + 6].copy_from_slice(b"SYNX01");
        bytes[offset + 0x18..offset + 0x1c].copy_from_slice(&[0x5d, 0x1c, 0x9e, 0xa3]);
        bytes[offset + 0x20..offset + 0x29].copy_from_slice(b"Normalize");
        bytes
    }

    fn normalized_directory(root: &Path, bytes: &[u8]) -> PathBuf {
        let meta = Meta::read(&mut Cursor::new(bytes)).unwrap();
        root.join(make_game_dir_name(meta.game_id(), meta.game_title()))
    }

    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in std::fs::read_dir(path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    visit(root, &path, files);
                } else {
                    files.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        std::fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        visit(root, root, &mut files);
        files
    }

    fn normalize(root: &Path) -> Result<()> {
        smol::block_on(async {
            adopt_orphaned_discs(root).await?;
            readopt_parented_discs(root).await
        })
    }

    #[test]
    fn normalization_preserves_flat_and_nested_three_part_iso_and_is_idempotent() {
        for nested in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let input = if nested {
                root.path().join("Old name")
            } else {
                root.path().to_path_buf()
            };
            std::fs::create_dir_all(&input).unwrap();
            let primary = disc(Format::Iso);
            std::fs::write(input.join("Original.PART0.ISO"), &primary).unwrap();
            std::fs::write(input.join("Original.PART1.ISO"), b"second part").unwrap();
            std::fs::write(input.join("Original.PART2.ISO"), b"third part").unwrap();
            normalize(root.path()).unwrap();
            let output = normalized_directory(root.path(), &primary);
            assert_eq!(
                std::fs::read(output.join("SYNX01.part0.iso")).unwrap(),
                primary
            );
            assert_eq!(
                std::fs::read(output.join("SYNX01.part1.iso")).unwrap(),
                b"second part"
            );
            assert_eq!(
                std::fs::read(output.join("SYNX01.part2.iso")).unwrap(),
                b"third part"
            );
            let before = snapshot(root.path());
            normalize(root.path()).unwrap();
            assert_eq!(snapshot(root.path()), before);
            assert_eq!(before.len(), 3);
        }
    }

    #[test]
    fn normalization_preserves_flat_and_nested_wbfs_parts() {
        for nested in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let input = if nested {
                root.path().join("Old name")
            } else {
                root.path().to_path_buf()
            };
            std::fs::create_dir_all(&input).unwrap();
            let primary = disc(Format::Wbfs);
            std::fs::write(input.join("Original.WBFS"), &primary).unwrap();
            for part in 1..=3 {
                std::fs::write(input.join(format!("Original.WBF{part}")), [part]).unwrap();
            }
            normalize(root.path()).unwrap();
            let output = normalized_directory(root.path(), &primary);
            assert_eq!(std::fs::read(output.join("SYNX01.wbfs")).unwrap(), primary);
            for part in 1..=3 {
                assert_eq!(
                    std::fs::read(output.join(format!("SYNX01.wbf{part}"))).unwrap(),
                    [part]
                );
            }
            let before = snapshot(root.path());
            normalize(root.path()).unwrap();
            assert_eq!(snapshot(root.path()), before);
        }
    }

    #[test]
    fn secondary_destination_collision_preserves_every_source_and_destination() {
        for nested in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let primary = disc(Format::Iso);
            let output = normalized_directory(root.path(), &primary);
            std::fs::create_dir(&output).unwrap();
            let input = if nested {
                output.clone()
            } else {
                root.path().to_path_buf()
            };
            std::fs::write(input.join("Original.part0.iso"), primary).unwrap();
            std::fs::write(input.join("Original.part1.iso"), b"source part").unwrap();
            std::fs::write(output.join("SYNX01.part1.iso"), b"existing target").unwrap();
            let before = snapshot(root.path());
            assert!(normalize(root.path()).is_err());
            assert_eq!(snapshot(root.path()), before);
        }
    }

    #[test]
    fn unsupported_compressed_disc_is_reported_without_panic_or_changes() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Original.rvz"), disc(Format::Rvz)).unwrap();
        let before = snapshot(root.path());
        assert!(normalize(root.path()).is_err());
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn missing_intermediate_iso_part_is_reported_without_changes() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Original.part0.iso"), disc(Format::Iso)).unwrap();
        std::fs::write(root.path().join("Original.part2.iso"), b"third part").unwrap();
        let before = snapshot(root.path());
        assert!(normalize(root.path()).is_err());
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn normalization_allows_a_forced_split_that_fits_in_one_part() {
        let root = tempfile::tempdir().unwrap();
        let primary = disc(Format::Iso);
        std::fs::write(root.path().join("Original.part0.iso"), &primary).unwrap();
        normalize(root.path()).unwrap();
        let output = normalized_directory(root.path(), &primary);
        assert_eq!(
            std::fs::read(output.join("SYNX01.part0.iso")).unwrap(),
            primary
        );
        let before = snapshot(root.path());
        normalize(root.path()).unwrap();
        assert_eq!(snapshot(root.path()), before);
        assert_eq!(before.len(), 1);
    }

    #[test]
    fn normalization_keeps_both_gc_discs_and_auxiliary_files() {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("Old name");
        std::fs::create_dir(&input).unwrap();
        let mut first = disc(Format::Iso);
        first[..6].copy_from_slice(b"GZZE01");
        first[0x18..0x1c].fill(0);
        first[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
        let mut second = first.clone();
        second[6] = 1;
        std::fs::write(input.join("First.iso"), &first).unwrap();
        std::fs::write(input.join("Second.iso"), &second).unwrap();
        std::fs::write(input.join("notes.txt"), b"keep notes").unwrap();
        normalize(root.path()).unwrap();
        let output = normalized_directory(root.path(), &first);
        assert_eq!(std::fs::read(output.join("game.iso")).unwrap(), first);
        assert_eq!(std::fs::read(output.join("disc2.iso")).unwrap(), second);
        assert_eq!(
            std::fs::read(output.join("notes.txt")).unwrap(),
            b"keep notes"
        );
        let before = snapshot(root.path());
        normalize(root.path()).unwrap();
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn existing_normalized_directory_is_detected_before_renaming_its_sources() {
        let root = tempfile::tempdir().unwrap();
        let primary = disc(Format::Iso);
        let input = root.path().join("Old name");
        let output = normalized_directory(root.path(), &primary);
        std::fs::create_dir(&input).unwrap();
        std::fs::create_dir(&output).unwrap();
        std::fs::write(input.join("Original.iso"), primary).unwrap();
        std::fs::write(output.join("notes.txt"), b"existing game folder").unwrap();
        let before = snapshot(root.path());
        assert!(normalize(root.path()).is_err());
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn failed_later_move_rolls_back_earlier_parts() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("source-one");
        let second = root.path().join("source-two");
        std::fs::write(&first, b"first source").unwrap();
        std::fs::write(&second, b"second source").unwrap();
        let before = snapshot(root.path());
        let moves = [
            (first, root.path().join("target-one")),
            (
                second,
                root.path().join("missing-parent").join("target-two"),
            ),
        ];
        preflight_moves(&moves).unwrap();
        assert!(apply_moves(&moves).is_err());
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn destination_created_after_preflight_is_never_replaced() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let target = root.path().join("target");
        std::fs::write(&source, b"source data").unwrap();
        preflight_moves(&[(source.clone(), target.clone())]).unwrap();
        std::fs::write(&target, b"other writer's data").unwrap();
        let before = snapshot(root.path());
        assert!(move_without_replacing(&source, &target).is_err());
        assert_eq!(snapshot(root.path()), before);
    }

    #[test]
    fn normalization_ignores_hidden_import_staging_directories() {
        let root = tempfile::tempdir().unwrap();
        let staging = root.path().join(".twbm-import-example");
        std::fs::create_dir(&staging).unwrap();
        std::fs::write(staging.join("SYNX01.iso"), disc(Format::Iso)).unwrap();
        let before = snapshot(root.path());
        normalize(root.path()).unwrap();
        assert_eq!(snapshot(root.path()), before);
    }
}
