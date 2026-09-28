// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::message::ConfigMessage,
    games::{
        calc_sha1::calc_sha1, conversion_state::ConversionState, convert::convert_game,
        export::export_game, games_state::GamesState, txtcodes::download_cheats,
    },
    homebrew::homebrew_state::HomebrewState,
    message::Message,
    notifications::notification::Notification,
    osc::osc_state::OscState,
    state::AppState,
    ui::{dialogs, modals::Modal, pages::Page},
};
use iced::Task;

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
            Message::NoOp => Task::none(),
            Message::Notify(notification) => {
                self.notifications.push(notification);
                Task::none()
            }
            Message::NavigateTo(page) => {
                self.current_page = page;

                if page == Page::Osc && matches!(self.osc_contents, OscState::Loaded(_)) {
                    self.reload_all_osc_icons();
                    self.download_osc_icons_task()
                } else {
                    Task::none()
                }
            }
            Message::PickMountPoint => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_mount_point_dialog_task),
            Message::MountPointPicked(path) => Task::batch([
                self.config
                    .update(ConfigMessage::SetMountPoint(path))
                    .map(Message::Config),
                self.refresh_games_and_apps(),
            ]),
            Message::CloseNotification(idx) => {
                self.notifications.close(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => self.refresh_games_and_apps(),
            Message::GotConfig(config) => {
                self.config = config;
                self.refresh_games_and_apps()
            }
            Message::GotGames(games) => {
                self.games = games;
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
                self.current_modal = Some(Modal::GameInfo((game.clone(), None)));
                self.get_disc_info_task(game)
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
            Message::GotDiscInfo(new_meta) => {
                if let Some(Modal::GameInfo((_, meta))) = &mut self.current_modal {
                    *meta = Some(new_meta);
                }

                Task::none()
            }
            Message::CouldNotGetDiscInfo(e) => {
                if let Some(Modal::GameInfo((_, meta))) = &mut self.current_modal {
                    *meta = None;
                }

                self.notifications.push(Notification::error(e));
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
            Message::PickHomebrewApps => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_homebrew_apps_dialog_task),
            Message::ImportHomebrewApps(paths) => self.import_homebrew_apps_task(paths),
            Message::HomebrewAppsImported(n) if n > 0 => {
                self.notifications.push(Notification::success(format!(
                    "{n} Homebrew app(s) successfully imported"
                )));

                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::HomebrewAppsImported(_) => Task::none(),
            Message::CalcGameSha1(game) => {
                self.hashing = ConversionState::Progress(String::new());
                Task::stream(calc_sha1(game)).map(Message::SetHashing)
            }
            Message::SetHashing(hashing) => {
                self.hashing = match hashing {
                    ConversionState::Finished(success) => {
                        self.notifications.push(Notification::success(success));
                        ConversionState::Idle
                    }
                    ConversionState::Errored(e) => {
                        self.notifications.push(Notification::error(e));
                        ConversionState::Idle
                    }
                    _ => hashing,
                };

                Task::none()
            }
            Message::PickGames => {
                if let GamesState::Loaded(games) = &self.games {
                    let existing_ids = games.get_all_game_ids().collect::<Box<[_]>>();

                    self.init_file_dialog_task().then(move |base| {
                        dialogs::make_pick_games_dialog_task(base, existing_ids.clone())
                    })
                } else {
                    Task::none()
                }
            }
            Message::PickGamesRecursively => {
                if let GamesState::Loaded(games) = &self.games {
                    let existing_ids = games.get_all_game_ids().collect::<Box<[_]>>();

                    self.init_file_dialog_task().then(move |base| {
                        dialogs::make_pick_games_recursively_dialog_task(base, existing_ids.clone())
                    })
                } else {
                    Task::none()
                }
            }
            Message::PickedGames(paths) => {
                if paths.is_empty() {
                    self.notifications
                        .push(Notification::info("No new games selected"));
                    Task::none()
                } else {
                    self.add_to_import_queue(paths);
                    self.current_page = Page::ImportQueue;
                    Task::none()
                }
            }
            Message::SetImporting(importing) => {
                let (importing, should_continue) = match importing {
                    ConversionState::Finished(success) => {
                        #[cfg(debug_assertions)]
                        self.notifications.push(Notification::success(success));

                        (ConversionState::Idle, true)
                    }
                    ConversionState::Errored(e) => {
                        self.notifications.push(Notification::error(e));
                        (ConversionState::Idle, true)
                    }
                    _ => (importing, false),
                };

                self.importing = importing;

                if should_continue {
                    Task::batch([
                        self.trigger_import_task(),
                        self.get_games_task(),
                        self.get_drive_info_task(),
                    ])
                } else {
                    Task::none()
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
                if let ConversionState::Progress(_) = self.importing {
                    self.animation_state = !self.animation_state;
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
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_search_term(search_term.to_lowercase());
                }

                Task::none()
            }
            Message::SearchHomebrewApps(search_term) => {
                if let HomebrewState::Loaded(apps) = &mut self.homebrew {
                    apps.set_search_term(search_term.to_lowercase());
                }

                Task::none()
            }
            Message::ToggleShowWii(checked) => {
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_show_wii(checked);
                }

                Task::none()
            }
            Message::ToggleShowNgc(checked) => {
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_show_ngc(checked);
                }

                Task::none()
            }
            Message::PickExportDest(game) => self.init_file_dialog_task().then(move |base| {
                dialogs::make_pick_export_game_dest_dialog_task(base, game.clone())
            }),
            Message::ExportGame(game, out_path) => {
                self.exporting = ConversionState::Progress(String::new());
                Task::stream(export_game(game, out_path)).map(Message::SetExporting)
            }
            Message::SetExporting(exporting) => {
                self.exporting = match exporting {
                    ConversionState::Finished(success) => {
                        self.notifications.push(Notification::success(success));
                        ConversionState::Idle
                    }
                    ConversionState::Errored(error) => {
                        self.notifications.push(Notification::error(error));
                        ConversionState::Idle
                    }
                    _ => exporting,
                };

                Task::none()
            }
            Message::RunTool(tool) => self.run_tool(tool),
            Message::PickFileToSendViaWiiload => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_file_to_wiiload_dialog_task),
            Message::SendViaWiiload(path) => Task::batch([
                self.config
                    .update(ConfigMessage::Write)
                    .map(Message::Config),
                self.send_via_wiiload(path),
            ]),
            Message::RefreshOscContents => {
                Task::perform(OscState::load(self.data_dir), Message::GotOscContents)
            }
            Message::GotOscContents(contents) => {
                self.set_osc_contents(contents);

                if self.current_page == Page::Osc
                    && matches!(self.osc_contents, OscState::Loaded(_))
                {
                    self.reload_all_covers();
                    self.download_osc_icons_task()
                } else {
                    Task::none()
                }
            }
            Message::SearchOscApps(search_term) => {
                if let OscState::Loaded(apps) = &mut self.osc_contents {
                    apps.set_search_term(search_term.to_lowercase());
                }

                Task::none()
            }
            Message::InstallOscApp(app) => {
                self.notifications
                    .push(Notification::info(format!("Installing {}", app.name())));

                let app_name = app.name().to_string();
                let mount_point = self.config.mount_point().to_path_buf();

                Task::perform(
                    async move { app.install(&mount_point).await },
                    move |res| match res {
                        Ok(()) => Message::OscAppInstalled(Ok(app_name)),
                        Err(e) => Message::OscAppInstalled(Err(e.to_string())),
                    },
                )
            }
            Message::OscAppInstalled(res) => {
                let notification = match res {
                    Ok(app_name) => Notification::success(format!("Installed {app_name}")),
                    Err(err) => Notification::error(err),
                };

                self.notifications.push(notification);
                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::DownloadTxtCodes(game) => {
                let game_title = game.title().to_string();
                let config = self.config.clone();

                self.notifications.push(Notification::info(format!(
                    "Downloading cheats for {game_title}",
                )));

                Task::perform(
                    async move { download_cheats(game.id(), &config).await },
                    move |res| match res {
                        Ok(()) => {
                            Notification::success(format!("Downloaded cheats for {game_title}"))
                        }
                        Err(err) => Notification::error(err.to_string()),
                    },
                )
                .map(Message::Notify)
            }
            Message::SendOscAppViaWiiload(app) => {
                let app_name = app.name().to_string();
                let wii_ip = self.config.wii_ip().to_string();

                self.notifications.push(Notification::info(format!(
                    "Sending {app_name} via wiiload"
                )));

                Task::batch([
                    self.config
                        .update(ConfigMessage::Write)
                        .map(Message::Config),
                    Task::perform(
                        async move { app.wiiload(&wii_ip).await },
                        move |res| match res {
                            Ok(()) => Notification::success(format!("Sent {app_name} via wiiload")),
                            Err(err) => Notification::error(err.to_string()),
                        },
                    )
                    .map(Message::Notify),
                ])
            }
            Message::PickGameToConvert => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_in_out_dialogs_task),
            Message::SetConverting(converting) => {
                self.converting = match converting {
                    ConversionState::Finished(success) => {
                        self.notifications.push(Notification::success(success));
                        ConversionState::Idle
                    }
                    ConversionState::Errored(error) => {
                        self.notifications.push(Notification::error(error));
                        ConversionState::Idle
                    }
                    _ => converting,
                };

                Task::none()
            }
            Message::ConvertGame(disc_path, out_path) => {
                Task::stream(convert_game(disc_path, out_path)).map(Message::SetConverting)
            }
            Message::GotUpdate(new_version) => {
                self.new_version = Some(new_version);
                Task::none()
            }
            Message::TriggerImport => self.trigger_import_task(),
            Message::FileDropped(path) => match self.current_page {
                Page::Games => Task::done(Message::PickedGames(vec![path])),
                Page::HomebrewApps => Task::done(Message::ImportHomebrewApps(vec![path])),
                _ => Task::none(),
            },
        }
    }
}
