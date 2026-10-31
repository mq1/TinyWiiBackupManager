// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, Default)]
pub enum LongOperationState {
    #[default]
    Idle,
    Progress(String),
    Finished(String),
    Errored(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongOperationKind {
    Import,
    Convert,
    Export,
    Scrub,
    Hash,
}

#[derive(Debug, Clone, Default)]
pub struct LongOperations {
    import: LongOperationState,
    convert: LongOperationState,
    export: LongOperationState,
    scrub: LongOperationState,
    hash: LongOperationState,
}

impl LongOperations {
    pub fn new() -> Self {
        Self::default()
    }

    fn get(&self, kind: LongOperationKind) -> &LongOperationState {
        match kind {
            LongOperationKind::Import => &self.import,
            LongOperationKind::Convert => &self.convert,
            LongOperationKind::Export => &self.export,
            LongOperationKind::Scrub => &self.scrub,
            LongOperationKind::Hash => &self.hash,
        }
    }

    pub fn set(&mut self, kind: LongOperationKind, state: LongOperationState) {
        match kind {
            LongOperationKind::Import => self.import = state,
            LongOperationKind::Convert => self.convert = state,
            LongOperationKind::Export => self.export = state,
            LongOperationKind::Scrub => self.scrub = state,
            LongOperationKind::Hash => self.hash = state,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &LongOperationState> {
        [
            &self.import,
            &self.convert,
            &self.export,
            &self.scrub,
            &self.hash,
        ]
        .into_iter()
    }

    pub fn is_idle(&self, kind: LongOperationKind) -> bool {
        matches!(self.get(kind), LongOperationState::Idle)
    }

    pub fn is_any_running(&self) -> bool {
        self.iter()
            .any(|state| matches!(state, LongOperationState::Progress(_)))
    }
}
