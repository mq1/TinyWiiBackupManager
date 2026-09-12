// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::toolbox::ToolboxGroup;

#[cfg(target_os = "macos")]
pub const ALL: &[ToolboxGroup] = {
    use crate::{errors::Error, toolbox::ToolboxItem};
    use iced::Task;
    use lucide_icons::Icon;
    use smol::process::Command;

    &[ToolboxGroup {
        label: "macOS",
        icon: Icon::Apple,
        items: &[ToolboxItem {
            label: "Run dot_clean (removes ._ files)",
            run: |ctx| {
                Task::future(async move {
                    Command::new("dot_clean")
                        .arg("-m")
                        .arg(ctx.config.mount_point)
                        .status()
                        .await
                        .map_err(Error::from)
                        .and_then(|status| status.success().ok_or(Error::DotClean))
                        .map(|_| "dot_clean ran successfully".to_string())
                })
            },
        }],
    }]
};

#[cfg(target_os = "windows")]
pub const ALL: &[ToolboxGroup] = &[];

#[cfg(target_os = "linux")]
pub const ALL: &[ToolboxGroup] = &[];
