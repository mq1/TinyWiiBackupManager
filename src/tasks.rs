// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, TaskType};
use anyhow::Result;
use slint::{SharedString, ToSharedString, Weak};
use std::sync::mpsc;
use std::thread;

pub type BoxedTask = Box<dyn FnOnce(&Weak<MainWindow>) -> Result<String> + Send>;

pub struct TaskProcessor {
    sender: mpsc::Sender<BoxedTask>,
    weak: Weak<MainWindow>,
}

impl TaskProcessor {
    pub fn init(weak: Weak<MainWindow>, hidden: bool) -> Self {
        let (sender, receiver) = mpsc::channel::<BoxedTask>();

        let weak_clone = weak.clone();
        thread::spawn(move || {
            while let Ok(task) = receiver.recv() {
                // Increment the task count
                if !hidden {
                    let _ = weak_clone.upgrade_in_event_loop(|handle| {
                        handle.set_task_count(handle.get_task_count() + 1);
                    });
                }

                // Execute the task and show the optional message
                let res = task(&weak_clone);
                let _ = weak_clone.upgrade_in_event_loop(|handle| match res {
                    Ok(msg) => handle.invoke_show_info(msg.to_shared_string()),
                    Err(e) => handle.invoke_show_error(e.to_shared_string()),
                });

                // Cleanup
                if !hidden {
                    let _ = weak_clone.upgrade_in_event_loop(move |handle| {
                        handle.set_status(SharedString::new());
                        handle.set_task_type(TaskType::Unknown);
                        handle.set_task_count(handle.get_task_count() - 1);
                    });
                }
            }
        });

        Self { sender, weak }
    }

    pub fn spawn(&self, task: BoxedTask) {
        if let Err(e) = self.sender.send(task)
            && let Some(handle) = self.weak.upgrade()
        {
            handle.invoke_show_error(e.to_shared_string());
        }
    }

    pub fn run_now(&self, task: BoxedTask) {
        let res = task(&self.weak);
        if let Some(handle) = self.weak.upgrade() {
            match res {
                Ok(msg) => handle.invoke_show_info(msg.to_shared_string()),
                Err(e) => handle.invoke_show_error(e.to_shared_string()),
            }
        }
    }
}
