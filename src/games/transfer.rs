// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::archive::ArchiveOperation, message::Message};
use anyhow::Result;
use iced::{Task, futures::TryFutureExt, task::Straw};
use std::{collections::VecDeque, path::PathBuf};

pub enum TransferOperation {
    Convert,
    Archive(ArchiveOperation),
}

impl TransferOperation {
    pub fn display_str(&self) -> &str {
        match self {
            TransferOperation::Convert => todo!(),
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

    pub fn pop_straw(&mut self) -> Option<Task<Message>> {
        self.queue.pop_front().map(|op| {
            let straw = match op {
                TransferOperation::Convert => todo!(),
                TransferOperation::Archive(op) => op.run(),
            };

            Task::sip(straw, Message::UpdateTransferStatus, Message::GenericResult)
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
