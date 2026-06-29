// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::game::Game,
    notifications::{Notification, Notifications},
    plugins::{self, plugin::Plugin},
    ui::pages::Page,
    util::{self, drive_info::DriveInfo},
};
use mlua::{ErrorContext, Function, Lua, LuaSerdeExt, Table};
use std::path::PathBuf;

pub(crate) struct AppState {
    pub data_dir: PathBuf,
    pub config: Config,
    pub notifications: Notifications,
    pub drive_info: Option<DriveInfo>,
    pub games: Vec<Game>,
    pub plugins: Vec<Plugin>,
    pub current_page: Page,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let config = Config::load(&data_dir);

        let mut initial = Self {
            data_dir,
            config,
            notifications: Notifications::new(),
            drive_info: None,
            games: Vec::new(),
            plugins: Vec::new(),
            current_page: Page::Games,
        };

        initial.reload_games();
        initial.reload_drive_info();
        initial.reload_plugins();

        initial
    }

    pub fn reload_drive_info(&mut self) {
        let mount_point = &self.config.contents.mount_point;

        if mount_point.as_os_str().is_empty() {
            self.drive_info = None;
            return;
        }

        self.drive_info = DriveInfo::try_from_path(mount_point)
            .map_err(|e| {
                let e = e.context("Failed to load drive info");
                self.notifications.add(Notification::error(e))
            })
            .ok();
    }

    pub fn reload_games(&mut self) {
        self.games = crate::games::list(
            &self.config.contents.mount_point,
            self.config.contents.sort_by,
        )
        .map_err(|e| {
            let e = e.context("Failed to load games");
            self.notifications.add(Notification::error(e))
        })
        .unwrap_or_default();
    }

    pub fn reload_plugins(&mut self) {
        self.plugins = plugins::load(&self.data_dir)
            .map_err(|e| {
                let e = e.context("Failed to load plugins");
                self.notifications.add(Notification::error(e))
            })
            .unwrap_or_default();
    }

    pub fn run_tool(&mut self, plugin_i: usize, tool_i: usize) {
        let lua = Lua::new();

        let res = lua.scope(|scope| {
            let plugin = lua.load(&self.plugins[plugin_i].code).eval::<Table>()?;

            let send_message = scope.create_function_mut({
                |lua, message| {
                    let message = lua.from_value(message)?;
                    let _ = self.update(message);
                    Ok(())
                }
            })?;

            let download_file = scope.create_function_mut({
                |_lua, (uri, dest): (String, String)| {
                    util::http::download_file(&uri, &dest).map_err(Into::into)
                }
            })?;

            let twbm = lua.create_table()?;
            twbm.set("send_message", send_message)?;
            twbm.set("download_file", download_file)?;

            let tools = plugin.get::<Vec<Table>>("tools")?;
            let run = tools[tool_i].get::<Function>("run")?;
            run.call::<()>(twbm)?;

            Ok(())
        });

        if let Err(e) = res {
            let e = e.context("Failed to run lua function");
            self.notifications.add(Notification::error(e));
        }
    }
}
