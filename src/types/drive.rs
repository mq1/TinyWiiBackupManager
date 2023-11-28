// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

use anyhow::{anyhow, bail, Result};
use sysinfo::{Disk, DiskExt, System, SystemExt};

use crate::types::game::Game;
use crate::wbfs_file;

const TITLES_URL: &str = "https://www.gametdb.com/titles.txt";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Drive {
    pub name: String,
    pub total_space: String,
    pub available_space: String,
    pub mount_point: PathBuf,
}

impl Drive {
    pub fn list() -> Vec<Self> {
        let mut sys = System::new();
        sys.refresh_disks_list();

        sys.disks()
            .iter()
            .filter(|disk| disk.is_removable())
            .map(Self::from)
            .collect::<Vec<_>>()
    }

    fn get_titles_map(&self) -> Result<HashMap<String, String>> {
        let mut titles = HashMap::new();

        let path = self.mount_point.join("titles.txt");
        if !path.exists() {
            self.download_titles()?;
        }

        let contents = fs::read_to_string(path)?;

        for line in contents.lines() {
            let mut line = line.split('=');
            let id = line
                .next()
                .ok_or_else(|| anyhow!("Invalid titles.txt"))?
                .trim();
            let title = line
                .next()
                .ok_or_else(|| anyhow!("Invalid titles.txt"))?
                .trim();
            titles.insert(id.to_string(), title.to_string());
        }

        Ok(titles)
    }

    fn download_titles(&self) -> Result<()> {
        let resp = ureq::get(TITLES_URL).call()?;

        let path = self.mount_point.join("titles.txt");
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        io::copy(&mut resp.into_reader(), &mut writer)?;

        Ok(())
    }

    pub fn get_games(&self) -> Result<Vec<Game>> {
        let wbfs_folder = self.mount_point.join("wbfs");
        if !wbfs_folder.exists() {
            fs::create_dir_all(&wbfs_folder)?;
            return Ok(Vec::new());
        }

        let titles = self.get_titles_map()?;

        let files = fs::read_dir(wbfs_folder)?;
        let games = files
            .filter_map(|file| {
                let file = file.ok()?;

                // check if file is a directory
                if !file.file_type().unwrap().is_dir() {
                    return None;
                }

                let dir = Game::new(file.path(), &titles).ok()?;

                Some(dir)
            })
            .collect();

        Ok(games)
    }

    pub fn add_game(&self, path: &Path) -> Result<()> {
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap() {
                "iso" => {
                    let dest = self.mount_point.join("wbfs");
                    wbfs_file::conv_to_wbfs_wrapper(path, &dest)?;
                }
                "wbfs" => {
                    let dest = self.mount_point.join("wbfs");
                    wbfs_file::copy_wbfs_file(path, &dest)?;
                }
                _ => bail!("Invalid file extension"),
            }
        }

        Ok(())
    }
}

impl fmt::Display for Drive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}/{} GiB)",
            self.name, self.available_space, self.total_space
        )
    }
}

impl From<&Disk> for Drive {
    fn from(disk: &Disk) -> Self {
        let name = disk.name().to_string_lossy().to_string();
        let total_space_gib = format!("{:.2}", disk.total_space() as f32 / 1073741824.);
        let available_space_gib = format!("{:.2}", disk.available_space() as f32 / 1073741824.);
        let mount_point = disk.mount_point().to_path_buf();

        Drive {
            name,
            total_space: total_space_gib,
            available_space: available_space_gib,
            mount_point,
        }
    }
}
