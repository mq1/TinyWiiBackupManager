// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use std::{fs, path::PathBuf, sync::OnceLock};

static PROJ: OnceLock<ProjectDirs> = OnceLock::new();

pub fn init() -> Result<()> {
    let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;

    PROJ.set(proj)
        .map_err(|_| anyhow!("Failed to set project dirs"))?;

    Ok(())
}

pub fn data_dir() -> Result<PathBuf> {
    let proj = PROJ.get().ok_or(anyhow!("PROJ not initialized"))?;
    let dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;

    Ok(dir)
}
