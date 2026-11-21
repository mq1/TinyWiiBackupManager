// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::message::ConfigMessage,
    long_operation::{Status, message::LongOperationsMessage},
    message::Message,
    notifications::notification::Notification,
    state::{AppState, LazyState},
    ui::{dialogs, modals::Modal, pages::Page},
};
use iced::Task;
use smol::{channel::bounded, unblock};
use twbm_games::{convert::convert_game, import_entry::ImportEntry, scrub::scrub_game};

const NEW_DRIVE_TEXT: &str = "New drive detected (or a breaking TWBM update has been installed)\nA path normalization run is recommended\nYou can find it in the Toolbox page";

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Config(message) => {
                match &message {
                    ConfigMessage::Written(Err(e)) => {
                        self.notifications.push(Notification::error(e));
                    }
                    ConfigMessage::NewMountPoint => {
                        self.notifications.push(Notification::info(NEW_DRIVE_TEXT));
                    }
                    _ => {}
                }

                self.config.update(message).map(Message::Config)
            }
            Message::LongOperations(message) => {
                match &message {
                    LongOperationsMessage::TriggerContinue => {
                        return Task::batch([
                            self.trigger_import_task(),
                            self.get_games_task(),
                            self.get_drive_info_task(),
                        ]);
                    }
                    LongOperationsMessage::SetImport(status)
                    | LongOperationsMessage::SetExport(status)
                    | LongOperationsMessage::SetConvert(status)
                    | LongOperationsMessage::SetScrub(status)
                    | LongOperationsMessage::SetHash(status) => match status {
                        Status::Finished(success) => {
                            self.notifications.push(Notification::success(success));
                        }
                        Status::Errored(errored) => {
                            self.notifications.push(Notification::error(errored));
                        }
                        _ => {}
                    },
                }

                self.long_operations
                    .update(message)
                    .map(Message::LongOperations)
            }
            Message::NoOp => Task::none(),
            Message::Notify(notification) => {
                self.notifications.push(notification);
                Task::none()
            }
            Message::NavigateTo(page) => self.navigate_to_task(page),
            Message::PickMountPoint => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_mount_point_dialog_task)
            }
            Message::MountPointPicked(path) => Task::batch([
                self.config
                    .update(ConfigMessage::SetMountPoint(path))
                    .map(Message::Config),
                self.refresh_games_and_apps_task(),
            ]),
            Message::CloseNotification(idx) => {
                self.notifications.close(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => self.refresh_games_and_apps_task(),
            Message::GotConfig(config) => {
                self.config = config;
                self.refresh_games_and_apps_task()
            }
            Message::GotGames(game_manager) => {
                self.game_manager = game_manager;
                self.reload_all_covers();
                self.download_ui_covers_task()
            }
            Message::GotHomebrew(homebrew) => {
                self.set_homebrew_apps(homebrew);
                Task::none()
            }
            Message::GotDrive(drive) => {
                self.drive = drive;
                Task::none()
            }
            Message::Open(url) => {
                if let Err(e) = open::that(url) {
                    self.notifications.push(Notification::error(e));
                }

                Task::none()
            }
            Message::OpenGameInfo(game) => {
                self.current_modal = Some(Modal::GameInfo((game.clone(), None, false)));

                Task::batch([
                    self.get_disc_info_task(game.clone()),
                    self.check_if_has_update_partition_task(game),
                ])
            }
            Message::OpenHomebrewAppInfo(app) => {
                self.current_modal = Some(Modal::HomebrewAppInfo(app));
                Task::none()
            }
            Message::OpenOscAppInfo(app) => {
                self.current_modal = Some(Modal::OscAppInfo(app));
                Task::none()
            }
            Message::CloseModal => {
                self.current_modal = None;
                Task::none()
            }
            Message::GotDiscInfo(meta) => {
                self.set_disc_info(meta);
                Task::none()
            }
            Message::GotUpdatePartitionCheckResult(has_update_partition) => {
                self.set_scrubbable(has_update_partition);
                Task::none()
            }
            Message::AskDeleteDir(path) => {
                self.current_modal = Some(Modal::DeleteDir(path));
                Task::none()
            }
            Message::DeleteDir(path) => self.delete_dir_task(path),
            Message::DirDeleted(res) => {
                if let Err(e) = res {
                    self.notifications.push(Notification::error(e));
                }

                Task::done(Message::RefreshGamesAndApps)
            }
            Message::PickHomebrewApps => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_homebrew_apps_dialog_task)
            }
            Message::ImportHomebrewApps(paths) => self.import_homebrew_apps_task(paths),
            Message::HomebrewAppsImported(n) if n > 0 => {
                self.notifications.push(Notification::success(format!(
                    "{n} Homebrew app(s) successfully imported"
                )));

                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::HomebrewAppsImported(_) => Task::none(),
            Message::CalcGameSha1(game) => self.calc_game_sha1_task(game),
            Message::PickGames => self.pick_games_task(),
            Message::PickGamesRecursively => self.pick_games_recursively_task(),
            Message::PickedGames(paths) => {
                if paths.is_empty() {
                    self.notifications
                        .push(Notification::info("No new games selected"));
                    Task::none()
                } else {
                    self.import_queue.extend(paths);
                    self.navigate_to_task(Page::ImportQueue)
                }
            }
            Message::CancelImport(i) => {
                self.import_queue.remove(i);
                Task::none()
            }
            Message::CancelAllImports => {
                self.import_queue.clear();
                Task::none()
            }
            Message::ToggleAnimationState => {
                if !matches!(self.long_operations.import, Status::Idle) {
                    self.animation_state.toggle();
                }

                Task::none()
            }
            Message::ReloadCover(game_id) => {
                self.reload_cover(game_id);
                Task::none()
            }
            Message::ReloadOscIcon(idx) => {
                self.reload_osc_icon(idx);
                Task::none()
            }
            Message::SearchGames(search_term) => {
                self.search_games(&search_term);
                Task::none()
            }
            Message::SearchHomebrewApps(search_term) => {
                self.search_homebrew_apps(&search_term);
                Task::none()
            }
            Message::ToggleShowWii(checked) => {
                self.set_show_wii(checked);
                Task::none()
            }
            Message::ToggleShowNgc(checked) => {
                self.set_show_ngc(checked);
                Task::none()
            }
            Message::PickExportDest(game) => dialogs::init_file_dialog_task().then(move |base| {
                dialogs::make_pick_export_game_dest_dialog_task(base, game.clone())
            }),
            Message::ExportGame(game, out_path) => self.export_game_task(game, out_path),
            Message::RunTool(tool) => self.run_tool_task(tool),
            Message::PickFileToSendViaWiiload => dialogs::init_file_dialog_task()
                .then(dialogs::make_pick_file_to_wiiload_dialog_task),
            Message::SendViaWiiload(path) => Task::batch([
                self.config
                    .update(ConfigMessage::Write)
                    .map(Message::Config),
                self.send_via_wiiload_task(path),
            ]),
            Message::RefreshOscContents => self.load_osc_state_task(),
            Message::GotOscContents(contents) => {
                self.set_osc_contents(contents);

                if self.current_page == Page::Osc
                    && matches!(self.osc_contents, LazyState::Loaded(_))
                {
                    self.reload_all_osc_icons();
                    self.download_osc_icons_task()
                } else {
                    Task::none()
                }
            }
            Message::SearchOscApps(search_term) => {
                self.search_osc_apps(&search_term);
                Task::none()
            }
            Message::InstallOscApp(app) => self.install_osc_app_task(app),
            Message::OscAppInstalled(res) => {
                let notification = match res {
                    Ok(app_name) => Notification::success(format!("Installed {app_name}")),
                    Err(err) => Notification::error(err),
                };

                self.notifications.push(notification);
                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::DownloadTxtCodes(game) => self.download_txt_codes_task(game),
            Message::SendOscAppViaWiiload(app) => self.send_osc_app_via_wiiload_task(app),
            Message::PickGameToConvert => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_in_out_dialogs_task)
            }
            Message::ConvertGame(disc_path, out_path) => {
                let (tx, rx) = bounded(1);

                let _ = std::thread::spawn(move || {
                    let send_progress = |msg| _ = tx.try_send(Status::Progress(msg));
                    let _ = tx.send_blocking(
                        match convert_game(&disc_path, &out_path, send_progress) {
                            Ok(msg) => Status::Finished(msg),
                            Err(e) => Status::Errored(e.to_string()),
                        },
                    );
                });

                Task::stream(rx)
                    .map(LongOperationsMessage::SetConvert)
                    .map(Message::LongOperations)
            }
            Message::GotUpdate(new_version) => {
                self.new_version = Some(new_version);
                Task::none()
            }
            Message::TriggerImport => self.trigger_import_task(),
            Message::FileDropped(path) => match self.current_page {
                Page::Games => {
                    Task::perform(unblock(move || ImportEntry::new(path)), |res| match res {
                        Ok(entry) => Message::PickedGames(vec![entry]),
                        Err(e) => Message::Notify(e.into()),
                    })
                }
                Page::HomebrewApps => Task::done(Message::ImportHomebrewApps(vec![path])),
                _ => Task::none(),
            },
            Message::AskConfirmScrub(game) => {
                self.current_modal = Some(Modal::ConfirmScrub(game));
                Task::none()
            }
            Message::Scrub(game) => {
                self.current_modal = None;
                let config = self.config.contents.clone();
                let has_4gib_filesize_limit = self.drive.has_4gib_filesize_limit();

                let (tx, rx) = bounded(1);

                let _ = std::thread::spawn(move || {
                    let send_progress = |msg| _ = tx.try_send(Status::Progress(msg));
                    let _ = tx.send_blocking(
                        match scrub_game(&game, &config, has_4gib_filesize_limit, send_progress) {
                            Ok(msg) => Status::Finished(msg),
                            Err(e) => Status::Errored(e.to_string()),
                        },
                    );
                });

                Task::stream(rx)
                    .map(LongOperationsMessage::SetScrub)
                    .map(Message::LongOperations)
                    .chain(Task::done(Message::RefreshGamesAndApps))
            }
        }
    }
}
