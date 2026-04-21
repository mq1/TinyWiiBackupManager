// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{MirroredProperties, model::AppModel};
use slint::{Global, ModelRc, ToSharedString};

impl MirroredProperties<'_> {
    pub fn link(&self, model: &AppModel) {
        self.set_app_version(env!("CARGO_PKG_VERSION").to_shared_string());

        model.config().wire(self.as_weak(), Self::set_config);
        model
            .drive_info()
            .wire(self.as_weak(), Self::set_drive_info);
        model.status().wire(self.as_weak(), Self::set_status);
        model
            .crc32_status()
            .wire(self.as_weak(), Self::set_crc32_status);

        self.set_games(ModelRc::from(model.games()));
        self.set_homebrew_apps(ModelRc::from(model.homebrew_apps()));
        self.set_osc_apps(ModelRc::from(model.osc_apps()));
        self.set_notifications(ModelRc::from(model.notifications()));
        self.set_conversion_queue(ModelRc::from(model.conversion_queue()));
        self.set_conversion_queue_buffer(ModelRc::from(model.conversion_queue_buffer()));
    }
}
