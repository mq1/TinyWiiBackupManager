// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, TaskType, show_err};
use anyhow::{Result, anyhow};
use slint::{ToSharedString, Weak};
use std::sync::mpsc;
use std::thread;

pub type BoxedTask = Box<dyn FnOnce(&Weak<MainWindow>) -> Result<()> + Send>;

pub struct TaskProcessor(mpsc::Sender<BoxedTask>);

impl TaskProcessor {
    pub fn init(weak: Weak<MainWindow>) -> Result<Self> {
        let (sender, receiver) = mpsc::channel::<BoxedTask>();

        thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                // Execute the task
                if let Err(e) = task(&weak) {
                    show_err(e);
                }

                // Clear the status message
                let _ = weak.upgrade_in_event_loop(|handle| {
                    handle.set_status("".to_shared_string());
                    handle.set_task_type(TaskType::Unknown);
                });
            }
        });

        Ok(Self(sender))
    }

    pub fn spawn(&self, task: BoxedTask) -> Result<()> {
        self.0
            .send(task)
            .map_err(|_| anyhow!("Failed to send task"))
    }
}
