// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, games::GameID, http::AGENT, tasks::BackgroundMessage};
use anyhow::Result;
use std::{fs, path::Path};

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.txt";

#[derive(Debug, Clone)]
pub struct Titles(Box<[(GameID, String)]>);

impl Titles {
    pub fn load(data_dir: &Path) -> Result<Self> {
        let path = data_dir.join("titles.txt");

        let contents = if data_dir.exists() {
            fs::read_to_string(path)?
        } else {
            let mut res = AGENT.get(DOWNLOAD_URL).call()?;
            let contents = res.body_mut().read_to_string()?;
            fs::write(path, &contents)?;
            contents
        };

        let mut titles = contents
            .lines()
            .filter_map(|line| line.split_once(" = "))
            .map(|(id, title)| (GameID::from(id), title.to_string()))
            .collect::<Vec<_>>();

        // We sort it now so we can binary search
        titles.sort_by_key(|(id, _)| *id);

        let list = titles.into_boxed_slice();

        Ok(Self(list))
    }

    pub fn get(&self, id: GameID) -> Option<&str> {
        let index = self.0.binary_search_by_key(&id, |(id, _)| *id).ok()?;
        let title = &self.0[index].1;

        Some(title)
    }
}

pub fn spawn_get_titles_task(app: &App) {
    let data_dir = app.data_dir.clone();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "📓 Loading titles...".to_string(),
        ))?;

        let titles = Titles::load(&data_dir)?;
        msg_sender.send(BackgroundMessage::GotTitles(titles))?;
        msg_sender.send(BackgroundMessage::NotifyInfo(
            "📓 Titles loaded".to_string(),
        ))?;

        Ok(())
    });
}
