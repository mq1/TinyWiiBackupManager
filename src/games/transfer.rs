// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{archive::ArchiveOperation, convert_for_wii::ConvertForWiiOperation},
    message::Message,
};
use iced::Task;
use std::collections::VecDeque;

pub enum TransferOperation {
    ConvertForWii(ConvertForWiiOperation),
    Archive(ArchiveOperation),
}

impl TransferOperation {
    pub fn display_str(&self) -> &str {
        match self {
            TransferOperation::ConvertForWii(op) => op.display_str(),
            TransferOperation::Archive(op) => op.display_str(),
        }
    }
}

pub struct TransferQueue {
    queue: VecDeque<TransferOperation>,
}

impl TransferQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
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
        self.queue.pop_front().map(|op| match op {
            TransferOperation::ConvertForWii(op) => Task::sip(
                op.run(),
                Message::UpdateTransferStatus,
                Message::Transferred,
            ),
            TransferOperation::Archive(op) => Task::sip(
                op.run(),
                Message::UpdateTransferStatus,
                Message::Transferred,
            ),
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
}
