// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game::Game;
use anyhow::{Result, bail};
use crc32fast::Hasher;
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

pub fn perform(game: Game, update_progress: &impl Fn(u8)) -> Result<u32> {
    let mut hasher = Hasher::new();

    if game.is_wii {
        let wbfs = game.path.join(format!("{}.wbfs", game.id));

        if wbfs.exists() {
            hash_file(&mut hasher, &wbfs, update_progress)?;

            let wbf1 = game.path.join(format!("{}.wbf1", game.id));
            if wbf1.exists() {
                hash_file(&mut hasher, &wbf1, update_progress)?;

                let wbf2 = game.path.join(format!("{}.wbf2", game.id));
                if wbf2.exists() {
                    hash_file(&mut hasher, &wbf2, update_progress)?;

                    let wbf3 = game.path.join(format!("{}.wbf3", game.id));
                    if wbf3.exists() {
                        hash_file(&mut hasher, &wbf3, update_progress)?;
                    }
                }
            }
        } else {
            let iso = game.path.join(format!("{}.iso", game.id));

            if iso.exists() {
                hash_file(&mut hasher, &iso, update_progress)?;
            } else {
                let part0iso = game.path.join(format!("{}.part0.iso", game.id));
                let part1iso = game.path.join(format!("{}.part1.iso", game.id));

                if part0iso.exists() && part1iso.exists() {
                    hash_file(&mut hasher, &part0iso, update_progress)?;
                    hash_file(&mut hasher, &part1iso, update_progress)?;
                } else {
                    bail!("No valid Wii game file found for {}", game.id);
                }
            }
        }
    } else {
        let iso = game.path.join("game.iso");

        if iso.exists() {
            hash_file(&mut hasher, &iso, update_progress)?;
        } else {
            let ciso = game.path.join("game.ciso");

            if ciso.exists() {
                hash_file(&mut hasher, &ciso, update_progress)?;
            } else {
                bail!("No valid GameCube game file found for {}", game.id);
            }
        }
    }

    let checksum = hasher.finalize();
    let crc32 = format!("{checksum:08x}");

    let crc32_path = game.path.join(format!("{}.crc32", game.id));
    if !crc32_path.exists() {
        fs::write(crc32_path, &crc32)?;
    }

    Ok(checksum)
}

fn hash_file(hasher: &mut Hasher, path: &Path, update_progress: &impl Fn(u8)) -> Result<()> {
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
            update_progress(current_percentage as u8);
            last_update = Instant::now();
        }
    }
    Ok(())
}
