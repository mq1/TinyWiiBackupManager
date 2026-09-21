// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use compact_str::CompactString;

#[derive(Debug, Clone, Default)]
pub enum ConversionState {
    #[default]
    Idle,
    Progress(CompactString),
    Finished(CompactString),
    Errored(CompactString),
}
