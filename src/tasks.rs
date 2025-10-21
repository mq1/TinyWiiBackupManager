// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ArchiveFormat, osc::OscApp, titles::Titles, updater::UpdateInfo};
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::thread;

pub type BoxedTask = Box<dyn FnOnce(&Sender<BackgroundMessage>) -> Result<()> + Send>;

pub struct TaskProcessor {
    task_sender: Sender<BoxedTask>,
    pub msg_receiver: Receiver<BackgroundMessage>,
}

impl TaskProcessor {
    pub fn init() -> Self {
        let (task_sender, task_receiver) = unbounded::<BoxedTask>();
        let (msg_sender, msg_receiver) = unbounded::<BackgroundMessage>();

        thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                if let Err(e) = task(&msg_sender) {
                    msg_sender
                        .send(BackgroundMessage::NotifyError(e.to_string()))
                        .expect("Failed to send message");
                }

                // Cleanup
                msg_sender
                    .send(BackgroundMessage::ClearStatus)
                    .expect("Failed to send message");
            }
        });

        Self {
            task_sender,
            msg_receiver,
        }
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(&Sender<BackgroundMessage>) -> Result<()> + Send + 'static,
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
    UpdateStatus(String),
    ClearStatus,
    TriggerRefreshImages,
    TriggerRefreshGames,
    TriggerRefreshHbcApps,
    GotUpdateInfo(UpdateInfo),
    GotTitles(Titles),
    GotOscApps(Vec<OscApp>),
    GotRedumpDb,
    SetArchiveFormat(ArchiveFormat),
}
