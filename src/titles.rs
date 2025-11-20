// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{games::GameID, http};
use anyhow::Result;
use std::{fs, path::Path};

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.txt";

#[derive(Debug, Clone)]
pub struct Titles(Box<[([u8; 6], String)]>);

impl Titles {
    pub fn empty() -> Self {
        Self(Box::new([]))
    }

    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join("titles.txt");

        let contents = if path.exists() {
            fs::read_to_string(path)?
        } else {
            let bytes = http::get(DOWNLOAD_URL)?;
            fs::write(&path, &bytes)?;
            String::from_utf8(bytes)?
        };

        let mut titles = contents
            .lines()
            .filter_map(|line| line.split_once(" = "))
            .map(|(id, title)| (<[u8; 6]>::from_id_str(id), title.to_string()))
            .collect::<Box<[_]>>();

        // We sort it now so we can binary search
        titles.sort_by_key(|(id, _)| *id);

        Ok(Self(titles))
    }

    pub fn get(&self, id: [u8; 6]) -> Option<&str> {
        self.0
            .binary_search_by_key(&id, |(id, _)| *id)
            .ok()
            .map(|i| self.0[i].1.as_str())
    }
}

pub fn spawn_get_titles_task(app: &App) {
    let data_dir = app.data_dir.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus("📓 Loading titles...".to_string()))?;

        let titles = Titles::load(&data_dir)?;
        msg_sender.send(Message::GotTitles(titles))?;
        msg_sender.send(Message::NotifyInfo("📓 Titles loaded".to_string()))?;

        Ok(())
    });
}
