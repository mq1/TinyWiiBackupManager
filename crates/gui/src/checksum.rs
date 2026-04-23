// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Logic;
use anyhow::{Result, bail};
use crc32fast::Hasher;
use slint::{ToSharedString, Weak};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

fn hash_file(hasher: &mut Hasher, path: &Path, weak: &Weak<Logic<'static>>) -> Result<()> {
    let mut f = File::open(path)?;
    let size = f.metadata()?.len();

    if size == 0 {
        bail!("File is empty");
    }

    let mut progress = 0;
    let mut last_update = Instant::now();
    let mut buf = vec![0; 128 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);

        progress += n as u64;

        if last_update.elapsed() > Duration::from_millis(100) {
            let current_percentage = progress * 100 / size;
            let status = format!("{current_percentage}%");

            let _ = weak.upgrade_in_event_loop(move |logic| {
                logic.invoke_set_crc32_status(status.to_shared_string());
            });

            last_update = Instant::now();
        }
    }
    Ok(())
}

pub fn perform(
    game_dir: impl Into<PathBuf>,
    is_wii: bool,
    game_id: impl AsRef<str>,
    weak: &Weak<Logic<'static>>,
) -> Result<()> {
    let game_dir = game_dir.into();
    let mut hasher = Hasher::new();

    if is_wii {
        let wbfs = game_dir.join(format!("{}.wbfs", game_id.as_ref()));

        if wbfs.exists() {
            hash_file(&mut hasher, &wbfs, weak)?;

            let wbf1 = game_dir.join(format!("{}.wbf1", game_id.as_ref()));
            if wbf1.exists() {
                hash_file(&mut hasher, &wbf1, weak)?;

                let wbf2 = game_dir.join(format!("{}.wbf2", game_id.as_ref()));
                if wbf2.exists() {
                    hash_file(&mut hasher, &wbf2, weak)?;

                    let wbf3 = game_dir.join(format!("{}.wbf3", game_id.as_ref()));
                    if wbf3.exists() {
                        hash_file(&mut hasher, &wbf3, weak)?;
                    }
                }
            }
        } else {
            let iso = game_dir.join(format!("{}.iso", game_id.as_ref()));

            if iso.exists() {
                hash_file(&mut hasher, &iso, weak)?;
            } else {
                let part0iso = game_dir.join(format!("{}.part0.iso", game_id.as_ref()));
                let part1iso = game_dir.join(format!("{}.part1.iso", game_id.as_ref()));

                if part0iso.exists() && part1iso.exists() {
                    hash_file(&mut hasher, &part0iso, weak)?;
                    hash_file(&mut hasher, &part1iso, weak)?;
                } else {
                    bail!("No valid Wii game file found for {}", game_id.as_ref());
                }
            }
        }
    } else {
        let iso = game_dir.join("game.iso");

        if iso.exists() {
            hash_file(&mut hasher, &iso, weak)?;
        } else {
            let ciso = game_dir.join("game.ciso");

            if ciso.exists() {
                hash_file(&mut hasher, &ciso, weak)?;
            } else {
                bail!("No valid GameCube game file found for {}", game_id.as_ref());
            }
        }
    }

    let checksum = hasher.finalize();
    let crc32 = format!("{checksum:08x}");

    let crc32_path = game_dir.join(format!("{}.crc32", game_id.as_ref()));
    if !crc32_path.exists() {
        fs::write(crc32_path, &crc32)?;
    }

    let _ = weak.upgrade_in_event_loop(move |logic| {
        logic.invoke_set_crc32_status(crc32.to_shared_string());
    });

    Ok(())
}
