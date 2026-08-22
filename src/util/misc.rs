// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

pub struct OptimalThreads {
    pub preloader: usize,
    pub processor: usize,
}

pub static OPTIMAL_THREADS: LazyLock<OptimalThreads> = LazyLock::new(|| {
    let cores = std::thread::available_parallelism().map_or(0, |n| n.get());

    let preloader = match cores {
        0 | 1 => 0,
        2..=4 => 1,
        5..=8 => 2,
        _ => 4,
    };

    let processor = cores - preloader;

    OptimalThreads {
        preloader,
        processor,
    }
});
