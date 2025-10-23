// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{convert::get_disc_opts, overflow_reader::get_main_file};
use anyhow::{Result, anyhow};
use nod::{
    disc::DiscHeader,
    read::{DiscMeta, DiscReader},
};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DiscInfo {
    pub header: DiscHeader,
    pub meta: DiscMeta,
}

impl DiscInfo {
    pub fn from_game_dir(game_dir: &Path) -> Result<DiscInfo> {
        let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
        let disc = DiscReader::new(&path, &get_disc_opts())?;

        let header = disc.header().to_owned();
        let meta = disc.meta();

        Ok(Self { header, meta })
    }
}
