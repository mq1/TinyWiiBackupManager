// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Idle,
    Progress(Box<str>),
    Finished(Box<str>),
    Errored(Box<str>),
}

impl<T: ToString, E: ToString> From<Result<T, E>> for Status {
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(val) => Self::Finished(val.to_string().into()),
            Err(e) => Self::Errored(e.to_string().into()),
        }
    }
}
