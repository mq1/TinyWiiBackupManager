// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::thread;

pub type BoxedTask = Box<dyn FnOnce(&Sender<Message>) -> Result<()> + Send>;

pub struct TaskProcessor {
    sender: Sender<BoxedTask>,
    receiver: Receiver<BoxedTask>,
}

impl TaskProcessor {
    pub fn new(msg_sender: Sender<Message>) -> Self {
        let (sender, receiver) = unbounded::<BoxedTask>();

        let receiver_clone = receiver.clone();
        thread::spawn(move || {
            while let Ok(task) = receiver_clone.recv() {
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

        Self { sender, receiver }
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(&Sender<Message>) -> Result<()> + Send + 'static,
    {
        self.sender
            .send(Box::new(task))
            .expect("Failed to send task");
    }

    pub fn pending(&self) -> usize {
        self.sender.len()
    }

    pub fn cancel_pending(&mut self) {
        let n = self.receiver.try_iter().count();
        log::info!("Canceled {} pending tasks", n);
    }
}
