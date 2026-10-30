// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

//! The task manager (M1a): every longer action is a task with progress, run in parallel where the
//! resources it uses allow it. Nothing here knows iced, so the server (M9) can use it too.

// Wired into the UI in the next M1a pull requests; until then only the tests use all of it.
#![allow(dead_code, unused_imports)]

mod control;
mod manager;
mod progress;

pub use control::{Cancelled, TaskControl};
pub use manager::{Resource, Task, TaskId, TaskManager, TaskState};
pub use progress::{Progress, SpeedMeter};
