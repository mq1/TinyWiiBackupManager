// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, TaskType, show_err};
use anyhow::{Result, anyhow};
use slint::{ToSharedString, Weak};
use std::{sync::OnceLock, thread};

pub type TaskClosure = Box<dyn FnOnce(&Weak<MainWindow>) -> Result<()> + Send + Sync + 'static>;

/// A task processor that runs tasks sequentially using a channel-based queue.
pub static TASK_PROCESSOR: OnceLock<crossbeam_channel::Sender<TaskClosure>> = OnceLock::new();

/// Creates a new TaskProcessor and spawns its background worker thread.
pub fn init(weak: Weak<MainWindow>) -> Result<()> {
    let (task_sender, task_receiver) = crossbeam_channel::unbounded::<TaskClosure>();

    thread::spawn(move || {
        while let Ok(task) = task_receiver.recv() {
            // Execute the task
            task(&weak).err().map(show_err);

            // Clear the status message
            let _ = weak.upgrade_in_event_loop(|handle| {
                handle.set_status("".to_shared_string());
                handle.set_task_type(TaskType::Unknown);
            });
        }
    });

    TASK_PROCESSOR
        .set(task_sender)
        .map_err(|_| anyhow!("Failed to initialize task processor"))
}

pub fn spawn_task(task: TaskClosure) {
    TASK_PROCESSOR
        .get()
        .map(|processor| processor.send(task))
        .ok_or("Failed to spawn task")
        .err()
        .map(show_err);
}

pub fn get_tasks_count() -> i32 {
    TASK_PROCESSOR
        .get()
        .map(|processor| processor.len() as i32 + 1)
        .unwrap_or(0)
}
