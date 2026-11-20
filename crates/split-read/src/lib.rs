// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod part;
mod plain;
mod read_at;
mod shared;

pub use plain::SplitFileReader;
pub use shared::SharedSplitFileReader;
