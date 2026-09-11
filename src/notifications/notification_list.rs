// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::notifications::notification::Notification;
use smol::{fs::File, io::AsyncWriteExt, lock::Mutex};
use std::{path::Path, sync::Arc};
use tap::Pipe;

#[derive(Debug, Default)]
pub struct NotificationList {
    inner: Vec<Notification>,
    log: Option<Arc<Mutex<File>>>,
}

impl NotificationList {
    pub fn new(data_dir: &Path) -> Self {
        let log = data_dir
            .join("log.txt")
            .pipe(std::fs::File::create)
            .ok()
            .map(File::from)
            .map(Mutex::new)
            .map(Arc::new);

        NotificationList {
            inner: Vec::new(),
            log,
        }
    }

    pub fn push(&mut self, notification: Notification) {
        println!("{notification}");

        if let Some(log) = &self.log {
            let log = log.clone();
            let s = format!("{notification}\n");

            smol::spawn(async move {
                let _ = log.lock().await.write_all(s.as_bytes()).await;
            })
            .detach();
        }

        self.inner.push(notification);
    }

    pub fn close(&mut self, index: usize) {
        self.inner.remove(index);
    }

    pub fn has_notifications(&self) -> bool {
        !self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.inner.iter()
    }
}
