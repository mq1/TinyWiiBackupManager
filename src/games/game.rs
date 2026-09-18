// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::{fs::get_dir_size, sha1_list};
use anyhow::{Context, Result, bail};
use iced::{
    advanced::image::{Allocation, allocate},
    task::{Straw, sipper},
    widget::image,
};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use size::Size;
use smol::fs;
use smol_str::{SmolStr, StrExt, ToSmolStr, format_smolstr};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub struct Game {
    path: PathBuf,
    id: GameID,
    title: SmolStr,
    size: u64,
    size_str: SmolStr,
    is_wii: bool,
    cover: Option<Allocation>,
    search_term_lowercase: SmolStr,
}

impl Game {
    pub async fn try_from_path(path: impl Into<PathBuf>, is_wii: bool) -> Result<Self> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
            bail!("{} is not a directory", path.display());
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("invalid filename")?;

        if dir_name.starts_with('.') {
            bail!("hidden directory");
        }

        // Extract title and id from the directory name
        let (title_raw, id_raw) = dir_name.split_once('[').context("invalid filename")?;

        let id_raw = id_raw.strip_suffix(']').context("invalid filename")?;

        // Parse the id
        let id = id_raw.parse::<GameID>().context("invalid game id")?;

        // get the pretty title
        let title = twbm_idmap::get_title(id)
            .map_or_else(|| SmolStr::new(title_raw.trim()), SmolStr::new_static);

        let size = get_dir_size(&path).await;
        let size_str = Size::from_bytes(size).to_smolstr();

        let search_term_lowercase = format_smolstr!("{}\0{}", title, id).to_lowercase_smolstr();

        Ok(Self {
            path,
            id,
            title,
            size,
            size_str,
            is_wii,
            cover: None,
            search_term_lowercase,
        })
    }

    pub async fn get_disc_path(&self) -> Option<PathBuf> {
        let wii_wbfs = format!("{}.wbfs", self.id);
        let wii_iso = format!("{}.iso", self.id);
        let wii_part0_iso = format!("{}.part0.iso", self.id);

        let possible_filenames = [
            wii_wbfs.as_str(),
            wii_iso.as_str(),
            wii_part0_iso.as_str(),
            "game.iso",
            "game.ciso",
        ];

        for filename in possible_filenames {
            let path = self.path.join(filename);

            if fs::metadata(&path).await.is_ok_and(|meta| meta.is_file()) {
                return Some(path);
            }
        }

        None
    }

    pub fn calc_sha1(&self) -> impl Straw<SmolStr, SmolStr, anyhow::Error> + use<> {
        let game = self.clone();

        sipper(async move |mut sender| {
            let (tx, rx) = smol::channel::bounded(1);

            let disc_path = game.get_disc_path().await.context("disc not found")?;

            let game_title = game.title.to_string();
            let handle = std::thread::spawn(move || {
                let disc = DiscReader::new(&disc_path, &DiscOptions::default())?;

                let process_opts = ProcessOptions {
                    digest_sha1: true,
                    ..Default::default()
                };

                let writer = DiscWriter::new(disc, &FormatOptions::default())?;

                let mut prev_percentage = 100;
                let finalization = writer.process(
                    |_, progress, total| {
                        let progress_percentage = progress * 100 / total;

                        if progress_percentage != prev_percentage {
                            let status = format_smolstr!(
                                "✓  Hashing {game_title}  {progress_percentage:02}%"
                            );
                            let _ = tx.try_send(status);

                            prev_percentage = progress_percentage;
                        }

                        Ok(())
                    },
                    &process_opts,
                )?;

                let sha1 = finalization.sha1.context("Failed to calculate SHA1")?;

                let known_sha1 = sha1_list::is_known(&sha1);

                Ok::<_, anyhow::Error>(known_sha1)
            });

            while let Ok(msg) = rx.recv().await {
                sender.send(msg).await;
            }

            let known_sha1 = handle.join().expect("Failed to join thread")?;

            if known_sha1 {
                Ok(format_smolstr!(
                    "Hash match for {}!  -  SHA1 is well known, your dump is perfect",
                    game.title
                ))
            } else {
                bail!("Hash mismatch for {}", game.title)
            }
        })
    }

    pub fn load_cover_blocking(&mut self, data_dir: &Path) {
        let cover_path = data_dir
            .join("covers")
            .join(self.id.as_str())
            .with_extension("png");

        self.cover = std::fs::read(cover_path)
            .ok()
            .map(image::Handle::from_bytes)
            .map(|handle| unsafe { allocate(&handle, (176, 248).into()) })
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn size_str(&self) -> &str {
        &self.size_str
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn id(&self) -> GameID {
        self.id
    }

    pub fn id_str(&self) -> &str {
        self.id.as_str()
    }

    pub fn is_wii(&self) -> bool {
        self.is_wii
    }

    pub fn is_ngc(&self) -> bool {
        !self.is_wii
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cover(&self) -> Option<&image::Handle> {
        self.cover.as_ref().map(|cover| cover.handle())
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
