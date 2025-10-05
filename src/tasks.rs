// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, TaskType, show_err};
use anyhow::{Result, anyhow};
use slint::{ToSharedString, Weak};
use std::{sync::OnceLock, thread};

pub type TaskClosure = dyn FnOnce(&Weak<MainWindow>) -> Result<()> + Send;

/// A task processor that runs tasks sequentially using a channel-based queue.
static TASK_PROCESSOR: OnceLock<crossbeam_channel::Sender<Box<TaskClosure>>> = OnceLock::new();

/// Creates a new TaskProcessor and spawns its background worker thread.
pub fn init(weak: Weak<MainWindow>) -> Result<()> {
    let (task_sender, task_receiver) = crossbeam_channel::unbounded::<Box<TaskClosure>>();

    thread::spawn(move || {
        while let Ok(task) = task_receiver.recv() {
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

    TASK_PROCESSOR
        .set(task_sender)
        .map_err(|_| anyhow!("Failed to initialize TASK_PROCESSOR"))
}

pub fn spawn<F>(task: F) -> Result<()> 
where
    F: FnOnce(&Weak<MainWindow>) -> Result<()> + Send + 'static,
{
    let processor = TASK_PROCESSOR
        .get()
        .ok_or(anyhow!("TASK_PROCESSOR not initialized"))?;

    processor
        .send(Box::new(task))
        .map_err(|_| anyhow!("Failed to spawn task"))
}

pub fn count() -> i32 {
    TASK_PROCESSOR
        .get()
        .map(|processor| processor.len() as i32 + 1)
        .unwrap_or(0)
}
