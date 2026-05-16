// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint::{SharedString, ToSharedString};
use std::{
    fmt::{Display, Write},
    fs::File,
    path::Path,
};
use twbm_core::game_id::GameID;
use zip::ZipArchive;

pub const GIB: f32 = 1024. * 1024. * 1024.;
pub const MIB: f32 = 1024. * 1024.;

pub fn display_list<T>(list: &[T]) -> SharedString
where
    T: Display,
{
    let mut s = SharedString::new();

    let last_i = list.len() - 1;
    for (i, value) in list.iter().enumerate() {
        write!(&mut s, "{value}").unwrap();

        if i != last_i {
            s.push_str(", ");
        }
    }

    s
}

pub fn should_add_game(path: &Path, existing_ids: &[GameID]) -> Option<SharedString> {
    let ext = path.extension()?;

    let meta = if ext.eq_ignore_ascii_case("zip") {
        let mut f = File::open(path).ok()?;
        let mut zip = ZipArchive::new(&mut f).ok()?;
        let mut disc_file = zip.by_index(0).ok()?;
        wii_disc_info::Meta::read(&mut disc_file).ok()?
    } else {
        let mut f = File::open(path).ok()?;
        wii_disc_info::Meta::read(&mut f).ok()?
    };

    let game_id = GameID::new(meta.game_id())?;
    if existing_ids.contains(&game_id) {
        return None;
    }

    Some(path.to_string_lossy().to_shared_string())
}
