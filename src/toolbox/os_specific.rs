// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::toolbox::ToolboxGroup;

#[cfg(target_os = "macos")]
pub const ALL: &[ToolboxGroup] = {
    use crate::{
        errors::Error,
        toolbox::{ToolContext, ToolboxItem},
    };
    use async_trait::async_trait;
    use lucide_icons::Icon;
    use smol::process::Command;

    #[derive(Debug)]
    pub struct RunDotClean;

    #[async_trait]
    impl ToolboxItem for RunDotClean {
        fn label(&self) -> &'static str {
            "Run dot_clean (removes ._ files)"
        }

        async fn run(&self, ctx: ToolContext) -> Result<String, Error> {
            Command::new("dot_clean")
                .arg("-m")
                .arg(ctx.config.mount_point)
                .status()
                .await
                .map_err(Error::from)
                .and_then(|status| status.success().ok_or(Error::DotClean))
                .map(|_| "dot_clean ran successfully".to_string())
        }
    }

    &[ToolboxGroup {
        label: "macOS",
        icon: Icon::Apple,
        items: &[&RunDotClean],
    }]
};

#[cfg(target_os = "windows")]
pub const ALL: &[ToolboxGroup] = &[];

#[cfg(target_os = "linux")]
pub const ALL: &[ToolboxGroup] = &[];
