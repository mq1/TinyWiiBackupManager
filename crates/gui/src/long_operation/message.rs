// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::long_operation::Status;

#[derive(Debug, Clone)]
pub enum LongOperationsMessage {
    SetImport(Status),
    SetConvert(Status),
    SetExport(Status),
    SetScrub(Status),
    SetHash(Status),
    TriggerContinue,
}
