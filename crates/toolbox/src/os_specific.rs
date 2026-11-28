// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ToolboxGroup;

#[cfg(target_os = "macos")]
pub const ALL: &[ToolboxGroup] = {
    use crate::ToolboxItem;
    use anyhow::{Context, anyhow};
    use std::process::Command;

    &[ToolboxGroup {
        label: "macOS",
        icon: "apple",
        items: &[ToolboxItem {
            label: "Run dot_clean (removes ._ files)",
            run: |ctx| {
                let mount_point = ctx.config.mount_point.context("Invalid mount point")?;

                let status = Command::new("dot_clean")
                    .arg("-m")
                    .arg(mount_point)
                    .status()?;

                if status.success() {
                    Ok("dot_clean ran successfully".into())
                } else {
                    Err(anyhow!("dot_clean failed"))
                }
            },
        }],
    }]
};

#[cfg(target_os = "windows")]
pub const ALL: &[ToolboxGroup] = &[];

#[cfg(target_os = "linux")]
pub const ALL: &[ToolboxGroup] = &[];
