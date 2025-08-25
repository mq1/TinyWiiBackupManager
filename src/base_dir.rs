// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::{ConsoleType, Game};
use anyhow::{Context, Result};
use notify::{FsEventWatcher, RecursiveMode, Watcher};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct BaseDir(PathBuf);

impl BaseDir {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let mut base_dir = Self(path);
        base_dir.correct();
        base_dir.create_dirs()?;
        Ok(base_dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn name(&self) -> String {
        // Return the name of the directory
        // file_name works in almost all cases
        // If it's a Windows Drive, it just returns the path

        self.0
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.0.to_string_lossy().to_string())
    }

    fn correct(&mut self) {
        if matches!(
            self.0.file_name().and_then(|name| name.to_str()),
            Some("wbfs" | "games")
        ) && let Some(parent) = self.0.parent()
        {
            self.0 = parent.to_path_buf();
        }
    }

    fn wii_dir(&self) -> PathBuf {
        self.0.join("wbfs")
    }

    fn gc_dir(&self) -> PathBuf {
        self.0.join("games")
    }

    fn create_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(self.wii_dir())?;
        std::fs::create_dir_all(self.gc_dir())?;
        Ok(())
    }

    pub fn get_watcher(&self, callback: impl notify::EventHandler) -> Result<FsEventWatcher> {
        let mut watcher = notify::recommended_watcher(callback)?;

        self.create_dirs()?;
        watcher.watch(&self.wii_dir(), RecursiveMode::NonRecursive)?;
        watcher.watch(&self.gc_dir(), RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }

    /// Scans the "wbfs" and "games" directories and get the list of games and the size of the base directory
    pub fn get_games(&self) -> Result<(Vec<Game>, u64)> {
        let mut games = Vec::new();
        scan_dir(self.wii_dir(), &mut games, ConsoleType::Wii)?;
        scan_dir(self.gc_dir(), &mut games, ConsoleType::GameCube)?;

        // Sort the combined vector
        games.sort_by(|a, b| a.display_title.cmp(&b.display_title));

        // sum the sizes of each game object
        let base_dir_size = games.iter().fold(0, |acc, game| acc + game.size);

        Ok((games, base_dir_size))
    }

    /// Run dot_clean to clean up MacOS ._ files
    #[cfg(target_os = "macos")]
    pub fn run_dot_clean(&self) -> Result<()> {
        std::process::Command::new("dot_clean")
            .arg("-m")
            .arg(&self.0)
            .status()
            .context("Failed to run dot_clean")?;

        Ok(())
    }

    pub fn open(&self) -> Result<()> {
        open::that(&self.0).map_err(anyhow::Error::from)
    }
}

impl fmt::Display for BaseDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// Scans a directory for games
fn scan_dir(dir: impl AsRef<Path>, games: &mut Vec<Game>, console_type: ConsoleType) -> Result<()> {
    let dir = dir.as_ref();

    if !dir.is_dir() {
        return Ok(());
    }

    std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| Game::from_path(path, console_type).ok())
        .for_each(|game| {
            games.push(game);
        });

    Ok(())
}
