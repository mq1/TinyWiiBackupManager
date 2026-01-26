// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        archive::ArchiveOperation, checksum::ChecksumOperation,
        convert_for_wii::ConvertForWiiOperation, strip::StripOperation,
    },
    message::Message,
};
use iced::Task;
use std::collections::VecDeque;

pub enum TransferOperation {
    ConvertForWii(ConvertForWiiOperation),
    Archive(ArchiveOperation),
    Strip(StripOperation),
    Checksum(ChecksumOperation),
}

impl TransferOperation {
    pub fn display_str(&self) -> &str {
        match self {
            TransferOperation::ConvertForWii(op) => op.display_str(),
            TransferOperation::Archive(op) => op.display_str(),
            TransferOperation::Strip(op) => op.display_str(),
            TransferOperation::Checksum(op) => op.display_str(),
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
            TransferOperation::Strip(op) => Task::sip(
                op.run(),
                Message::UpdateTransferStatus,
                Message::Transferred,
            ),
            TransferOperation::Checksum(op) => Task::sip(
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

    pub fn cancel_all(&mut self) {
        self.queue.clear();
    }

    pub fn has_pending_operations(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}
