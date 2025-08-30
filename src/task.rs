// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::messages::BackgroundMessage;
use anyhow::{Result, anyhow};
use egui_inbox::UiInboxSender;
use std::sync::mpsc;
use std::thread;

/// The Task is a boxed, sendable closure that takes a UiInboxSender and returns a Result.
type Task = Box<dyn FnOnce(UiInboxSender<BackgroundMessage>) -> Result<()> + Send>;

/// A task processor that runs tasks sequentially using a channel-based queue.
#[derive(Clone)]
pub struct TaskProcessor {
    ui_sender: UiInboxSender<BackgroundMessage>,
    task_sender: mpsc::Sender<Task>,
}

impl TaskProcessor {
    /// Creates a new TaskProcessor and spawns its background worker thread.
    pub fn new(ui_sender: UiInboxSender<BackgroundMessage>) -> Self {
        let (task_sender, task_receiver) = mpsc::channel::<Task>();
        let ui_sender_clone = ui_sender.clone();

        // Spawn a background OS thread to process tasks sequentially.
        thread::spawn(move || {
            // This loop will run until the sender side of the channel is dropped.
            while let Ok(task) = task_receiver.recv() {
                // Set the status message to Some to show the spinner
                let _ = ui_sender_clone.send(BackgroundMessage::UpdateStatus(Some("".to_string())));

                // Execute the task with its own clone of the sender and send any resulting error to the UI.
                if let Err(e) = task(ui_sender_clone.clone()) {
                    let _ = ui_sender_clone.send(e.into());
                }

                // clear the status message
                let _ = ui_sender_clone.send(BackgroundMessage::UpdateStatus(None));
            }
        });

        Self {
            ui_sender,
            task_sender,
        }
    }

    /// Spawns a new task by sending it to the worker's queue.
    /// The provided closure will be called with a clone of the `UiInboxSender`.
    pub fn spawn_task<F>(&self, task_closure: F)
    where
        F: FnOnce(UiInboxSender<BackgroundMessage>) -> Result<()> + Send + 'static,
    {
        // The `task_closure` is boxed and sent to the worker.
        if let Err(e) = self.task_sender.send(Box::new(task_closure)) {
            let e = anyhow!(e.to_string()).context("Failed to queue task");
            let _ = self.ui_sender.send(e.into());
        }
    }
}
