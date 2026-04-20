// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConfigContents, ConversionKind, DriveInfo, QueuedConversion, Rust, archive::ArchiveConversion,
    scrub::ScrubConversion, standard_conversion::StandardConversion,
};
use anyhow::Result;
use slint::Weak;
use std::num::NonZeroUsize;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB
pub const HEADER_SIZE: usize = 131_072;

pub enum Conversion {
    Scrub(ScrubConversion),
    Archive(ArchiveConversion),
    Standard(StandardConversion),
}

impl Conversion {
    pub fn new(queued: &QueuedConversion, conf: &ConfigContents, drive_info: &DriveInfo) -> Self {
        match queued.kind {
            ConversionKind::Scrub => {
                Self::Scrub(ScrubConversion::new(&queued.scrub, conf, drive_info))
            }
            ConversionKind::Archive => Self::Archive(ArchiveConversion::new(&queued.archive)),
            ConversionKind::Standard => {
                Self::Standard(StandardConversion::new(&queued.standard, conf, drive_info))
            }
        }
    }

    pub fn perform(&mut self, weak: &Weak<Rust<'static>>) -> Result<()> {
        match self {
            Conversion::Scrub(scrub) => scrub.perform(weak),
            Conversion::Archive(archive) => archive.perform(weak),
            Conversion::Standard(standard) => standard.perform(weak),
        }
    }
}
