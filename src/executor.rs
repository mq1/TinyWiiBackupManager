// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::futures::executor::block_on;

pub struct DumbExecutor;

impl iced::Executor for DumbExecutor {
    fn new() -> Result<Self, std::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = std::thread::spawn(move || block_on(future));
    }

    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        block_on(future)
    }
}
