// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        calc_sha1::calc_sha1, conversion_state::ConversionState, export::export_game,
        games_state::GamesState,
    },
    homebrew::homebrew_state::HomebrewState,
    messages::Message,
    notifications::notification::Notification,
    osc::osc_state::OscState,
    state::AppState,
    ui::{dialogs, modals::Modal, pages::Page},
};
use compact_str::CompactString;
use iced::Task;
use std::sync::Arc;
use tap::Pipe;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::Notify(notification) => {
                self.notifications.push(notification);
                Task::none()
            }
            Message::NavigateTo(page) => {
                self.current_page = page;

                if page == Page::Osc {
                    self.load_osc_contents_task()
                } else {
                    Task::none()
                }
            }
            Message::PickMountPoint => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_mount_point_dialog_task),
            Message::MountPointPicked(path) => {
                self.config.mount_point = path;
                self.write_config_task()
            }
            Message::CloseNotification(idx) => {
                self.notifications.close(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => Task::batch([
                self.get_games_task(),
                self.get_homebrew_apps_task(),
                self.get_drive_info_task(),
            ]),
            Message::GotConfig(config) => {
                let new_mount_point = config.mount_point != self.config.mount_point;

                self.config = config;

                if new_mount_point {
                    Task::done(Message::RefreshGamesAndApps)
                } else {
                    Task::none()
                }
            }
            Message::GotGames(games) => {
                self.games = games;
                self.reload_all_covers();
                self.download_ui_covers_task()
            }
            Message::GotHomebrew(homebrew) => {
                self.homebrew = homebrew;
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
            Message::SetViewAs(view_as) => {
                self.config.view_as = view_as;
                self.write_config_task()
            }
            Message::SetSortBy(sort_by) => {
                self.config.sort_by = sort_by;
                self.write_config_task()
            }
            Message::SetWiiIp(ip) => {
                self.config.wii_ip = ip;
                Task::none() // we'll write the config when wiiloading
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
                    let existing_ids = games
                        .get_all_game_ids()
                        .collect::<Box<[_]>>()
                        .pipe(Arc::new);

                    self.init_file_dialog_task().then(move |base| {
                        dialogs::make_pick_games_dialog_task(base, existing_ids.clone())
                    })
                } else {
                    Task::none()
                }
            }
            Message::PickGamesRecursively => {
                if let GamesState::Loaded(games) = &self.games {
                    let existing_ids = games
                        .get_all_game_ids()
                        .collect::<Box<[_]>>()
                        .pipe(Arc::new);

                    self.init_file_dialog_task().then(move |base| {
                        dialogs::make_pick_games_recursively_dialog_task(base, existing_ids.clone())
                    })
                } else {
                    Task::none()
                }
            }
            Message::ImportGames(paths) => self.import_games_task(paths),
            Message::SetImporting(importing) => {
                let task;
                (self.importing, task) = match importing {
                    ConversionState::Finished(_success) => {
                        //self.notifications.push(Notification::success(success));
                        (
                            ConversionState::Idle,
                            self.import_games_task(std::iter::empty()),
                        )
                    }
                    ConversionState::Errored(e) => {
                        self.notifications.push(Notification::error(e));
                        (
                            ConversionState::Idle,
                            self.import_games_task(std::iter::empty()),
                        )
                    }
                    _ => (importing, Task::none()),
                };

                task
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
            Message::ReloadOscIcon(slug) => {
                self.reload_osc_icon(&slug);
                Task::none()
            }
            Message::SetWiiOutputFormat(format) => {
                self.config.wii_output_format = format;
                self.write_config_task()
            }
            Message::SetGcOutputFormat(format) => {
                self.config.gc_output_format = format;
                self.write_config_task()
            }
            Message::SetAlwaysSplit(always_split) => {
                self.config.always_split = always_split;
                self.write_config_task()
            }
            Message::SetScrubUpdatePartition(scrub) => {
                self.config.scrub_update_partition = scrub;
                self.write_config_task()
            }
            Message::SetRemoveSourcesGames(remove) => {
                self.config.remove_sources_games = remove;
                self.write_config_task()
            }
            Message::SetRemoveSourcesApps(remove) => {
                self.config.remove_sources_apps = remove;
                self.write_config_task()
            }
            Message::SetTxtCodesSource(source) => {
                self.config.txt_codes_source = source;
                self.write_config_task()
            }
            Message::SetThemePreference(pref) => {
                self.config.theme_preference = pref;
                self.write_config_task()
            }
            Message::SetPreferredLanguage(lang) => {
                self.config.preferred_language = lang;
                self.write_config_task()
            }
            Message::SearchGames(search_term) => {
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_search_term(CompactString::from_str_to_lowercase(&search_term));
                }

                Task::none()
            }
            Message::SearchHomebrewApps(search_term) => {
                if let HomebrewState::Loaded(apps) = &mut self.homebrew {
                    apps.set_search_term(CompactString::from_str_to_lowercase(&search_term));
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
            Message::SendViaWiiload(path) => {
                Task::batch([self.write_config_task(), self.send_via_wiiload(path)])
            }
            Message::RefreshOscContents => self.load_osc_contents_task(),
            Message::GotOscContents(contents) => {
                self.osc_contents = contents;
                self.reload_all_osc_icons();
                self.download_osc_icons_task()
            }
            Message::SearchOscApps(search_term) => {
                if let OscState::Loaded(apps) = &mut self.osc_contents {
                    apps.set_search_term(CompactString::from_str_to_lowercase(&search_term));
                }

                Task::none()
            }
        }
    }
}
