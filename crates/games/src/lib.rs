// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::LazyLock;

pub mod calc_sha1;
pub mod convert;
mod disc_stream;
pub mod export;
pub mod game;
pub mod has_update_partition;
pub mod import;
pub mod import_entry;
pub mod manager;
mod multi_disc;
pub mod scrub;

pub trait FromExt: Sized {
    fn from_ext<'a>(ext: impl TryInto<&'a str>) -> Option<Self>;
}

impl FromExt for nod::common::Format {
    fn from_ext<'a>(ext: impl TryInto<&'a str>) -> Option<Self> {
        match ext.try_into() {
            Ok("iso" | "ISO" | "gcm" | "GCM") => Some(nod::common::Format::Iso),
            Ok("ciso" | "CISO") => Some(nod::common::Format::Ciso),
            Ok("gcz" | "CGZ") => Some(nod::common::Format::Gcz),
            Ok("rvz" | "RVZ") => Some(nod::common::Format::Rvz),
            Ok("wbfs" | "WBFS") => Some(nod::common::Format::Wbfs),
            Ok("wia" | "WIA") => Some(nod::common::Format::Wia),
            Ok("tgc" | "TGC") => Some(nod::common::Format::Tgc),
            _ => None,
        }
    }
}

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
