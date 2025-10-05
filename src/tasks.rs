// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MainWindow, TaskType, show_err};
use anyhow::Result;
use slint::{ToSharedString, Weak};
use std::{sync::OnceLock, thread};

pub type TaskClosure = Box<dyn FnOnce(&Weak<MainWindow>) -> Result<()> + Send + Sync + 'static>;

pub static TASK_PROCESSOR: OnceLock<TaskProcessor> = OnceLock::new();

/// A task processor that runs tasks sequentially using a channel-based queue.
#[derive(Clone)]
pub struct TaskProcessor(crossbeam_channel::Sender<TaskClosure>);

impl TaskProcessor {
    /// Creates a new TaskProcessor and spawns its background worker thread.
    pub fn new(weak: Weak<MainWindow>) -> Self {
        let (task_sender, task_receiver) = crossbeam_channel::unbounded();

        thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                Self::execute_task(task, &weak, task_receiver.len() as i32 + 1);
            }
        });

        Self(task_sender)
    }

    /// Executes a single task with proper cleanup.
    fn execute_task(task: TaskClosure, weak: &Weak<MainWindow>, task_count: i32) {
        // Update task counter
        let _ = weak.upgrade_in_event_loop(move |handle| {
            handle.set_task_count(task_count);
        });

        // Execute the task
        task(weak).err().map(show_err);

        // Clear the status message
        let _ = weak.upgrade_in_event_loop(|handle| {
            handle.set_status("".to_shared_string());
            handle.set_task_type(TaskType::Unknown);
        });
    }

    /// Spawns a new task by sending it to the worker's queue.
    pub fn spawn_task(&self, task: TaskClosure) {
        self.0.send(task).err().map(show_err);
    }
}
