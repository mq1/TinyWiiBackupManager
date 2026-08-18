// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    games::game_id::GameID,
    util::{self, sha1_list},
};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use sipper::{Straw, sipper};
use size::Size;
use smol::fs;
use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Game {
    pub path: PathBuf,
    pub id: GameID,
    pub title: Cow<'static, str>,
    pub size: Size,
    pub is_wii: bool,
    pub cached_cover_path: PathBuf,
}

impl Game {
    pub async fn try_from_path(
        path: impl Into<PathBuf>,
        is_wii: bool,
        covers_dir: &Path,
    ) -> Result<Self, Error> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
            return Err(Error::NotADir);
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        if dir_name.starts_with('.') {
            return Err(Error::HiddenDir);
        }

        // Extract title and id from the directory name
        let (title_raw, id_raw) = dir_name.split_once('[').ok_or(Error::InvalidFilename)?;

        let Some(id_raw) = id_raw.strip_suffix(']') else {
            return Err(Error::InvalidFilename);
        };

        // Parse the id
        let id = id_raw
            .parse::<GameID>()
            .map_err(|()| Error::InvalidFilename)?;

        // get the pretty title
        let title = twbm_idmap::get_title(id)
            .map_or_else(|| Cow::Owned(title_raw.trim().to_string()), Cow::Borrowed);

        let size = util::misc::get_dir_size(&path).await;

        let cached_cover_path = covers_dir.join(id.as_str()).with_extension("png");

        Ok(Self {
            path,
            id,
            title,
            size,
            is_wii,
            cached_cover_path,
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

    pub fn calc_sha1(&self) -> impl Straw<String, String, Error> + use<> {
        let game = self.clone();

        sipper(async move |mut sender| {
            let (tx, rx) = smol::channel::bounded(1);

            let disc_path = game.get_disc_path().await.ok_or(Error::DiscNotFound)?;

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
                            let status =
                                format!("✓  Hashing {game_title}  {progress_percentage:02}%");
                            let _ = tx.try_send(status);

                            prev_percentage = progress_percentage;
                        }

                        Ok(())
                    },
                    &process_opts,
                )?;

                let sha1 = finalization
                    .sha1
                    .ok_or_else(|| Error::NodOther("No SHA1".into()))?;

                let known_sha1 = sha1_list::is_known(&sha1);

                Ok::<_, Error>(known_sha1)
            });

            while let Ok(msg) = rx.recv().await {
                sender.send(msg).await;
            }

            let known_sha1 = handle.join().expect("Failed to join thread")?;

            if known_sha1 {
                Ok(format!(
                    "Hash match for {}!  -  SHA1 is well known, your dump is perfect",
                    game.title.as_ref()
                ))
            } else {
                Err(Error::HashMismatch(game.title.into_owned()))
            }
        })
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
