// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::archive::ArchiveOperation, message::Message};
use iced::{Task, futures::TryFutureExt};
use std::{collections::VecDeque, path::PathBuf, sync::Arc};

pub enum TransferOperation {
    Convert(PathBuf),
    Archive(ArchiveOperation),
}

impl TransferOperation {
    pub fn display_str(&self) -> &str {
        match self {
            TransferOperation::Convert(_) => todo!(),
            TransferOperation::Archive(op) => op.display_str(),
        }
    }
}

pub struct TransferQueue {
    queue: VecDeque<TransferOperation>,
    status: String,
}

impl TransferQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            status: String::new(),
        }
    }

    pub fn push(&mut self, operation: TransferOperation) {
        self.queue.push_back(operation);
    }

    pub fn push_multiple(&mut self, mut operations: Vec<TransferOperation>) {
        while let Some(op) = operations.pop() {
            self.queue.push_back(op);
        }
    }

    pub fn pop_task(&mut self) -> Option<Task<Message>> {
        self.queue.pop_front().map(|op| {
            let straw = match op {
                TransferOperation::Convert(_) => todo!(),
                TransferOperation::Archive(op) => op.run(),
            };

            Task::sip(straw, Message::UpdateTransferStatus, |res| {
                Message::GenericResult(res.map_err(Arc::new))
            })
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &TransferOperation> {
        self.queue.iter()
    }

    pub fn cancel(&mut self, i: usize) {
        self.queue.remove(i);
    }

    pub fn has_pending_operations(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
}
