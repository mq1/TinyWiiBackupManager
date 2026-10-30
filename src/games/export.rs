// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{convert::write_disc_blocking, game::Game},
    tasks::{self, TaskControl, WorkerEvent},
};
use nod::common::Format;
use smol::stream::Stream;
use std::path::PathBuf;

pub fn ext_to_format(ext: &str) -> Option<Format> {
    match ext {
        "iso" | "ISO" => Some(Format::Iso),
        "ciso" | "CISO" => Some(Format::Ciso),
        "gcz" | "CGZ" => Some(Format::Gcz),
        "rvz" | "RVZ" => Some(Format::Rvz),
        "wbfs" | "WBFS" => Some(Format::Wbfs),
        "wia" | "WIA" => Some(Format::Wia),
        "tgc" | "TGC" => Some(Format::Tgc),
        _ => None,
    }
}

/// Exports a game from the drive to a file as a task.
pub fn export_game(
    game: Game,
    out_path: PathBuf,
    control: TaskControl,
) -> impl Stream<Item = WorkerEvent> {
    tasks::spawn_blocking(move |reporter| {
        let disc_path = game.get_disc_path_blocking().ok_or("disc not found")?;

        write_disc_blocking(&disc_path, &out_path, &control, reporter, || {
            format!("Exporting {}", game.title())
        })
        .map(|()| format!("Exported {}", game.title()))
        .map_err(|e| e.to_string())
    })
}
