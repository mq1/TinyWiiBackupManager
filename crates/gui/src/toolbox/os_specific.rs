// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::toolbox::ToolboxGroup;

#[cfg(target_os = "macos")]
pub const ALL: &[ToolboxGroup] = {
    use crate::toolbox::ToolboxItem;
    use anyhow::{Context, anyhow};
    use lucide_icons::Icon;
    use smol::process::Command;

    &[ToolboxGroup {
        label: "macOS",
        icon: Icon::Apple,
        items: &[ToolboxItem {
            label: "Run dot_clean (removes ._ files)",
            run_fn: |ctx| {
                Box::pin(async move {
                    let mount_point = ctx
                        .config
                        .contents
                        .mount_point
                        .context("Invalid mount point")?;

                    let status = Command::new("dot_clean")
                        .arg("-m")
                        .arg(mount_point)
                        .status()
                        .await?;

                    if status.success() {
                        Ok("dot_clean ran successfully".to_string())
                    } else {
                        Err(anyhow!("dot_clean failed"))
                    }
                })
            },
        }],
    }]
};

#[cfg(target_os = "windows")]
pub const ALL: &[ToolboxGroup] = &[];

#[cfg(target_os = "linux")]
pub const ALL: &[ToolboxGroup] = &[];
