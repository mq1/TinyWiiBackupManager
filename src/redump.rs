// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, http::AGENT};
use anyhow::Result;
use slint::Weak;
use std::{
    fs::{self, File},
    io::{self, BufWriter, Cursor},
    path::Path,
};
use zip::ZipArchive;

pub fn download_all(data_dir: &Path, weak: &Weak<MainWindow>) -> Result<()> {
    for console in ["wii", "gc"] {
        let path = data_dir.join(format!("redump-{console}.dat"));
        if path.exists() {
            continue;
        }

        let url = format!("http://redump.org/datfile/{console}/");
        let mut res = AGENT.get(&url).call()?;
        let bytes = res.body_mut().read_to_vec()?;
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;
        let mut zipped_file = archive.by_index(0)?;
        let mut file = BufWriter::new(File::create(path)?);
        io::copy(&mut zipped_file, &mut file)?;
    }

    weak.upgrade_in_event_loop(move |handle| {
        handle.set_got_redump_db(true);
    })?;

    Ok(())
}

pub fn is_sha1_known(data_dir: &Path, game_sha1: &str, is_wii: bool) -> Result<bool> {
    let file_name = match is_wii {
        true => "redump-wii.dat",
        false => "redump-gc.dat",
    };

    let path = data_dir.join(file_name);
    let present = fs::read_to_string(path)?.contains(game_sha1);

    Ok(present)
}
