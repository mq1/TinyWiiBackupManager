// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use anyhow::Result;
use crossbeam_channel::{Sender, unbounded};
use std::thread;

pub type BoxedTask = Box<dyn FnOnce(&Sender<Message>) -> Result<()> + Send>;

pub struct TaskProcessor(Sender<BoxedTask>);

impl TaskProcessor {
    pub fn new(msg_sender: Sender<Message>) -> Self {
        let (task_sender, task_receiver) = unbounded::<BoxedTask>();

        thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                if let Err(e) = task(&msg_sender) {
                    msg_sender
                        .send(Message::NotifyError(e))
                        .expect("Failed to send message");
                }

                // Cleanup
                msg_sender
                    .send(Message::ClearStatus)
                    .expect("Failed to send message");
            }
        });

        Self(task_sender)
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(&Sender<Message>) -> Result<()> + Send + 'static,
    {
        self.0.send(Box::new(task)).expect("Failed to send task");
    }

    pub fn pending(&self) -> usize {
        self.0.len()
    }
}
