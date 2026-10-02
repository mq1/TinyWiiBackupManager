// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use strum::EnumCount;
use strum_macros::EnumCount;

#[derive(Debug, Clone)]
pub enum LongOperationState {
    Idle,
    Progress(String),
    Finished(String),
    Errored(String),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumCount)]
pub enum LongOperationKind {
    Import,
    Convert,
    Export,
    Scrub,
    Hash,
}

pub struct LongOperations {
    operations: [(LongOperationState, LongOperationKind); LongOperationKind::COUNT],
}

impl LongOperations {
    pub fn new() -> Self {
        Self {
            operations: [
                (LongOperationState::Idle, LongOperationKind::Import),
                (LongOperationState::Idle, LongOperationKind::Convert),
                (LongOperationState::Idle, LongOperationKind::Export),
                (LongOperationState::Idle, LongOperationKind::Scrub),
                (LongOperationState::Idle, LongOperationKind::Hash),
            ],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(LongOperationState, LongOperationKind)> {
        self.operations.iter()
    }

    pub fn set(&mut self, kind: LongOperationKind, state: LongOperationState) {
        for (op, op_kind) in &mut self.operations {
            if *op_kind == kind {
                *op = state;
                break;
            }
        }
    }

    pub fn is_idle(&self, kind: LongOperationKind) -> bool {
        self.operations
            .iter()
            .any(|(op, op_kind)| *op_kind == kind && matches!(op, LongOperationState::Idle))
    }

    pub fn is_running(&self, kind: LongOperationKind) -> bool {
        self.operations
            .iter()
            .any(|(op, op_kind)| *op_kind == kind && matches!(op, LongOperationState::Progress(_)))
    }

    pub fn is_any_running(&self) -> bool {
        self.operations
            .iter()
            .any(|(op, _)| matches!(op, LongOperationState::Progress(_)))
    }
}
