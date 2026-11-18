// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod message;
pub mod update;

use twbm_common::Status;

#[derive(Debug, Clone, Default)]
pub struct LongOperations {
    pub import: Status,
    pub convert: Status,
    pub export: Status,
    pub scrub: Status,
    pub hash: Status,
}

impl LongOperations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Status> {
        [
            &self.import,
            &self.convert,
            &self.export,
            &self.scrub,
            &self.hash,
        ]
        .into_iter()
    }

    pub fn is_any_running(&self) -> bool {
        self.iter()
            .any(|state| matches!(state, Status::Progress(_)))
    }
}
