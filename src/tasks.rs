// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, unbounded};
use parking_lot::Mutex;
use std::{sync::Arc, thread};

use crate::{games::Game, hbc_apps::HbcApp, titles::Titles, updater::UpdateInfo};

pub type BoxedTask =
    Box<dyn FnOnce(&Arc<Mutex<String>>, &Sender<BackgroundMessage>) -> Result<()> + Send>;

pub struct TaskProcessor {
    task_sender: Sender<BoxedTask>,
    pub msg_receiver: Receiver<BackgroundMessage>,
    pub status: Arc<Mutex<String>>,
}

impl TaskProcessor {
    pub fn init() -> Self {
        let (task_sender, task_receiver) = unbounded::<BoxedTask>();
        let (msg_sender, msg_receiver) = unbounded::<BackgroundMessage>();
        let status = Arc::new(Mutex::new(String::new()));

        let status_clone = status.clone();
        thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                if let Err(e) = task(&status_clone, &msg_sender) {
                    msg_sender
                        .send(BackgroundMessage::NotifyError(e.to_string()))
                        .expect("Failed to send message");
                }

                // Clean up
                status_clone.lock().clear();
            }
        });

        Self {
            task_sender,
            msg_receiver,
            status,
        }
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(&Arc<Mutex<String>>, &Sender<BackgroundMessage>) -> Result<()> + Send + 'static,
    {
        self.task_sender
            .send(Box::new(task))
            .expect("Failed to send task");
    }

    pub fn pending(&self) -> usize {
        self.task_sender.len()
    }
}

pub enum BackgroundMessage {
    NotifyInfo(String),
    NotifyError(String),
    TriggerRefreshImages,
    TriggerRefreshTitle,
    TriggerRefreshGames,
    TriggerRefreshHbcApps,
    GotUpdateInfo(UpdateInfo),
    GotTitles(Titles),
    GotGames(Vec<Game>),
    GotHbcApps(Vec<HbcApp>),
}
