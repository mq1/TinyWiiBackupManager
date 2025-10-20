// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use crossbeam_channel::{Sender, unbounded};
use egui_notify::Toasts;
use parking_lot::Mutex;
use std::{sync::Arc, thread};

pub type BoxedTask = Box<dyn FnOnce(Arc<Mutex<String>>, Arc<Mutex<Toasts>>) -> Result<()> + Send>;

pub struct TaskProcessor {
    task_sender: Sender<BoxedTask>,
    pub status: Arc<Mutex<String>>,
    toasts: Arc<Mutex<Toasts>>,
}

impl TaskProcessor {
    pub fn init(toasts: Arc<Mutex<Toasts>>) -> Self {
        let (task_sender, task_receiver) = unbounded::<BoxedTask>();

        let status = Arc::new(Mutex::new(String::new()));

        let status_clone = status.clone();
        let toasts_clone = toasts.clone();
        thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                if let Err(e) = task(status_clone.clone(), toasts_clone.clone()) {
                    toasts_clone.lock().error(e.to_string());
                }

                // Clean up
                status_clone.lock().clear();
            }
        });

        Self {
            task_sender,
            status,
            toasts,
        }
    }

    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(Arc<Mutex<String>>, Arc<Mutex<Toasts>>) -> Result<()> + Send + 'static,
    {
        if let Err(e) = self.task_sender.send(Box::new(task)) {
            self.toasts.lock().error(e.to_string());
        }
    }

    pub fn pending(&self) -> usize {
        self.task_sender.len()
    }
}
