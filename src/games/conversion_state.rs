// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use smol_str::SmolStr;

#[derive(Debug, Clone, Default)]
pub enum ConversionState {
    #[default]
    Idle,
    Progress(SmolStr),
    Finished(SmolStr),
    Errored(SmolStr),
}
