// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, TaskType, show_err};
use anyhow::Result;
use rfd::{MessageDialog, MessageLevel};
use slint::{SharedString, Weak};
use std::sync::mpsc;
use std::thread;

pub type BoxedTask = Box<dyn FnOnce(&Weak<MainWindow>) -> Result<String> + Send>;

pub struct TaskProcessor(mpsc::Sender<BoxedTask>);

impl TaskProcessor {
    pub fn init(weak: Weak<MainWindow>) -> Self {
        let (sender, receiver) = mpsc::channel::<BoxedTask>();

        thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                // Increment the task count
                let _ = weak.upgrade_in_event_loop(|handle| {
                    handle.set_task_count(handle.get_task_count() + 1);
                });

                // Execute the task and show the optional message
                match task(&weak) {
                    Ok(msg) => {
                        if !msg.is_empty() {
                            let _ = MessageDialog::new()
                                .set_level(MessageLevel::Info)
                                .set_title("Info")
                                .set_description(msg)
                                .show();
                        }
                    }
                    Err(e) => {
                        show_err(e);
                    }
                }

                // Cleanup
                let _ = weak.upgrade_in_event_loop(|handle| {
                    handle.set_status(SharedString::new());
                    handle.set_task_type(TaskType::Unknown);
                    handle.set_task_count(handle.get_task_count() - 1);
                });
            }
        });

        Self(sender)
    }

    pub fn spawn(&self, task: BoxedTask) {
        if let Err(e) = self.0.send(task) {
            show_err(e)
        }
    }
}
