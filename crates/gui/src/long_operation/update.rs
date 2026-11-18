// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::long_operation::{LongOperations, message::LongOperationsMessage};
use iced::Task;
use twbm_common::Status;

fn get_final_status(status: Status) -> Status {
    match status {
        Status::Finished(_) | Status::Errored(_) => Status::Idle,
        status => status,
    }
}

impl LongOperations {
    pub fn update(&mut self, message: LongOperationsMessage) -> Task<LongOperationsMessage> {
        match message {
            LongOperationsMessage::SetImport(status) => {
                let should_continue = matches!(status, Status::Finished(_) | Status::Errored(_));
                self.import = get_final_status(status);

                if should_continue {
                    Task::done(LongOperationsMessage::TriggerContinue)
                } else {
                    Task::none()
                }
            }
            LongOperationsMessage::SetConvert(status) => {
                self.convert = get_final_status(status);
                Task::none()
            }
            LongOperationsMessage::SetExport(status) => {
                self.export = get_final_status(status);
                Task::none()
            }
            LongOperationsMessage::SetScrub(status) => {
                self.scrub = get_final_status(status);
                Task::none()
            }
            LongOperationsMessage::SetHash(status) => {
                self.hash = get_final_status(status);
                Task::none()
            }
            LongOperationsMessage::TriggerContinue => Task::none(), // handled in AppState::update
        }
    }
}
