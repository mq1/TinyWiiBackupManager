// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{Config, message::ConfigMessage};
use iced::Task;

impl Config {
    pub fn update(&mut self, message: ConfigMessage) -> Task<ConfigMessage> {
        match message {
            ConfigMessage::SetMountPoint(path) => {
                self.contents.mount_point = path;
                self.write()
            }
            ConfigMessage::SetWiiIp(ip) => {
                self.contents.wii_ip = ip;
                Task::none()
            }
            ConfigMessage::SetShowWii(show) => {
                self.contents.show_wii = show;
                self.write()
            }
            ConfigMessage::SetShowGc(show) => {
                self.contents.show_gc = show;
                self.write()
            }
            ConfigMessage::SetSortBy(sort_by) => {
                self.contents.sort_by = sort_by;
                self.write()
            }
            ConfigMessage::SetViewAs(view_as) => {
                self.contents.view_as = view_as;
                self.write()
            }
            ConfigMessage::SetAlwaysSplit(split) => {
                self.contents.always_split = split;
                self.write()
            }
            ConfigMessage::SetRemoveSourcesGames(remove) => {
                self.contents.remove_sources_games = remove;
                self.write()
            }
            ConfigMessage::SetRemoveSourcesApps(remove) => {
                self.contents.remove_sources_apps = remove;
                self.write()
            }
            ConfigMessage::SetScrubUpdatePartition(scrub) => {
                self.contents.scrub_update_partition = scrub;
                self.write()
            }
            ConfigMessage::SetTxtCodesSource(source) => {
                self.contents.txt_codes_source = source;
                self.write()
            }
            ConfigMessage::SetThemePreference(pref) => {
                self.contents.theme_preference = pref;
                self.write()
            }
            ConfigMessage::SetWiiOutputFormat(format) => {
                self.contents.wii_output_format = format;
                self.write()
            }
            ConfigMessage::SetGcOutputFormat(format) => {
                self.contents.gc_output_format = format;
                self.write()
            }
            ConfigMessage::SetPreferredLanguage(language) => {
                self.contents.preferred_language = language;
                self.write()
            }
            ConfigMessage::Write => self.write(),
            ConfigMessage::Written(_) => Task::none(), // handled in State::update
        }
    }
}
