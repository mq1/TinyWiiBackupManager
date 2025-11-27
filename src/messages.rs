// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::osc::OscApp;
use crate::ui::Modal;
use crate::wiitdb;
use eframe::egui;
use semver::Version;

pub enum Message {
    NotifyInfo(String),
    NotifyError(anyhow::Error),
    NotifySuccess(String),
    UpdateStatus(String),
    ClearStatus,
    TriggerRefreshImage(String),
    TriggerRefreshGames,
    TriggerRefreshHbcApps,
    GotNewVersion(Version),
    GotOscApps(Box<[OscApp]>),
    GotWiitdb(wiitdb::Datafile),
    OpenModal(Modal),
    CloseModal,
    ArchiveGame(u16),
    WriteConfig,
    OpenGameDir(u16),
}

pub fn process_msg(app: &mut App, ctx: &egui::Context, msg: Message) {
    match msg {
        Message::NotifyInfo(i) => {
            app.notifications.show_info(&i);
        }
        Message::NotifyError(e) => {
            app.notifications.show_err(e);
        }
        Message::NotifySuccess(s) => {
            app.notifications.show_success(&s);
        }
        Message::UpdateStatus(string) => {
            app.status = string;
        }
        Message::ClearStatus => {
            app.status.clear();
        }
        Message::TriggerRefreshImage(uri) => {
            ctx.forget_image(&uri);
        }
        Message::TriggerRefreshGames => {
            app.refresh_games();
            app.update_title(ctx);
        }
        Message::TriggerRefreshHbcApps => {
            app.refresh_hbc_apps();
            app.update_title(ctx);
        }
        Message::GotNewVersion(version) => {
            let info = format!("A new version is available: {}", &version);
            app.notifications.show_info_no_duration(&info);
            app.update = Some(version);
        }
        Message::GotOscApps(osc_apps) => {
            app.osc_apps = osc_apps;
            app.update_filtered_osc_apps();
        }
        Message::GotWiitdb(data) => {
            app.wiitdb = Some(data);
        }
        Message::OpenModal(modal) => {
            app.current_modal = Some(modal);
        }
        Message::CloseModal => {
            app.current_modal = None;
        }
        Message::ArchiveGame(i) => {
            app.archiving_game_i = i;
            app.choose_archive_path.save_file();
        }
        Message::WriteConfig => {
            app.save_config();
        }
        Message::OpenGameDir(i) => {
            app.open_game_dir(i);
        }
    }
}
