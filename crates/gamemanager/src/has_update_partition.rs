// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game::Game;
use anyhow::{Context, Result};
use nod::{
    common::PartitionKind,
    read::{DiscOptions, DiscReader},
};

pub fn has_update_partition(game: &Game) -> Result<bool> {
    let disc_path = game.get_disc_path().context("disc not found")?;

    let disc_opts = DiscOptions {
        preloader_threads: 0,
        ..Default::default()
    };

    let disc_reader = DiscReader::new(disc_path, &disc_opts)?;

    let has_update_partition = disc_reader
        .partitions()
        .iter()
        .any(|partition| partition.kind == PartitionKind::Update);

    Ok(has_update_partition)
}
