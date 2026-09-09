// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::toolbox::ToolboxGroup;

#[cfg(target_os = "macos")]
pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    use crate::{errors::Error, toolbox::ToolboxItem};
    use iced::Task;
    use lucide_icons::Icon;
    use smol::process::Command;

    std::iter::once(&ToolboxGroup {
        label: "macOS",
        icon: Icon::Apple,
        items: &[ToolboxItem {
            label: "Run dot_clean (removes ._ files)",
            run: |ctx| {
                Task::future(async move {
                    let status = Command::new("dot_clean")
                        .arg("-m")
                        .arg(ctx.mount_point)
                        .status()
                        .await?;

                    if !status.success() {
                        return Err(Error::DotClean);
                    }

                    Ok("dot_clean ran successfully".to_string())
                })
            },
        }],
    })
}

#[cfg(target_os = "windows")]
pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    std::iter::empty()
}

#[cfg(target_os = "linux")]
pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    std::iter::empty()
}
