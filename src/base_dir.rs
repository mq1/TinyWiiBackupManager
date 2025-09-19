// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::{ConsoleType, Game};
use anyhow::{Context, Result, bail};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};
use ureq::http::StatusCode;
use zip::ZipArchive;
use zip::result::ZipResult;
use crate::AGENT;

#[derive(Clone, Serialize, Deserialize)]
pub struct BaseDir(PathBuf);

impl BaseDir {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let mut base_dir = Self(path);
        base_dir.correct();
        Ok(base_dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn name(&self) -> String {
        // Return the name of the directory
        // file_name works in almost all cases
        // If it's a Windows Drive, it just returns the path

        if let Some(file_name) = self.0.file_name() {
            file_name.to_string_lossy().to_string()
        } else {
            self.0.to_string_lossy().to_string()
        }
    }

    fn correct(&mut self) {
        if let Some(file_name) = self.0.file_name()
            && matches!(file_name.to_str(), Some("wbfs" | "games"))
            && let Some(parent) = self.0.parent()
        {
            self.0 = parent.to_path_buf();
        }
    }

    pub fn exists(&self) -> bool {
        self.0.exists()
    }

    pub fn apps_dir(&self) -> PathBuf {
        self.0.join("apps")
    }

    pub fn wii_dir(&self) -> PathBuf {
        self.0.join("wbfs")
    }

    pub fn gc_dir(&self) -> PathBuf {
        self.0.join("games")
    }

    pub fn usbloadergx_dir(&self) -> PathBuf {
        self.0.join("apps").join("usbloader_gx")
    }

    pub fn wiiflow_dir(&self) -> PathBuf {
        self.0.join("wiiflow")
    }

    pub fn cover_dir(&self) -> PathBuf {
        self.usbloadergx_dir().join("images")
    }

    pub fn wiiflow_cover_dir(&self) -> PathBuf {
        self.wiiflow_dir().join("boxcovers")
    }

    pub fn get_watcher(&self, callback: impl Fn() + Send + 'static) -> Result<RecommendedWatcher> {
        let handler = move |res| {
            if let Ok(notify::Event {
                kind:
                    notify::EventKind::Modify(_)
                    | notify::EventKind::Create(_)
                    | notify::EventKind::Remove(_),
                ..
            }) = res
            {
                callback();
            }
        };

        let mut watcher = notify::recommended_watcher(handler)?;

        fs::create_dir_all(self.wii_dir())?;
        watcher.watch(&self.wii_dir(), RecursiveMode::NonRecursive)?;

        fs::create_dir_all(self.gc_dir())?;
        watcher.watch(&self.gc_dir(), RecursiveMode::NonRecursive)?;

        fs::create_dir_all(self.apps_dir())?;
        watcher.watch(&self.apps_dir(), RecursiveMode::NonRecursive)?;

        Ok(watcher)
    }

    /// Scans the "wbfs" and "games" directories and get the list of games and the size of the base directory
    pub fn get_games(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        scan_dir(self.wii_dir(), &mut games, ConsoleType::Wii)?;
        scan_dir(self.gc_dir(), &mut games, ConsoleType::GameCube)?;

        // Sort the combined vector
        games.sort_by(|a, b| a.display_title.cmp(&b.display_title));

        Ok(games)
    }

    /// Run dot_clean to clean up MacOS ._ files
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

    pub fn download_file(
        &self,
        url: &str,
        rel_dir: impl AsRef<Path>,
        filename: &str,
    ) -> Result<bool> {
        let dir = self.0.join(rel_dir);
        let file_path = dir.join(filename);

        if file_path.exists() {
            return Ok(false);
        }

        let response = AGENT.get(url).call()?;

        if response.status() != StatusCode::OK {
            bail!("Failed to download file: {}", response.status());
        }

        let (_, body) = response.into_parts();
        let mut reader = body.into_reader();

        fs::create_dir_all(&dir)?;
        let mut file = File::create(&file_path)?;
        io::copy(&mut reader, &mut file)?;

        Ok(true)
    }

    /// we check if in the zip there is an "apps" directory
    /// if so, we extract it to the base directory
    /// otherwise, we extract the zip to the apps directory
    fn extract_app(&self, archive: &mut ZipArchive<impl io::Read + io::Seek>) -> ZipResult<()> {
        if archive.file_names().any(|n| n.starts_with("apps/")) {
            archive.extract(&self.0)
        } else {
            archive.extract(&self.0.join("apps"))
        }
    }

    pub fn add_zip(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;
        self.extract_app(&mut archive)?;

        Ok(())
    }

    pub fn add_zip_from_url(&self, url: &str) -> Result<()> {
        let mut response = AGENT.get(url).call()?;

        let buffer = response.body_mut().read_to_vec()?;
        let cursor = io::Cursor::new(buffer);
        let mut archive = ZipArchive::new(cursor)?;
        self.extract_app(&mut archive)?;

        Ok(())
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

    fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| Game::from_path(path, console_type).ok())
        .for_each(|game| {
            games.push(game);
        });

    Ok(())
}
