// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

/// Returns the number of preloader and processor threads to use.
pub fn get_threads_num() -> (usize, usize) {
    let cpus = num_cpus::get();

    let preloader_threads = if cpus <= 4 {
        1
    } else if cpus <= 8 {
        2
    } else {
        4
    };

    let processor_threads = cpus - preloader_threads;

    (preloader_threads, processor_threads)
}
