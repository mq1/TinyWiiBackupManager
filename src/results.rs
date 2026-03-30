// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DiscInfo, DiscInfoResult, EmptyResult};
use slint::{SharedString, ToSharedString};

impl<E> From<Result<(), E>> for EmptyResult
where
    E: ToSharedString,
{
    fn from(result: Result<(), E>) -> Self {
        match result {
            Ok(()) => EmptyResult::default(),
            Err(e) => EmptyResult {
                err: e.to_shared_string(),
            },
        }
    }
}

impl<E> From<Result<DiscInfo, E>> for DiscInfoResult
where
    E: ToSharedString,
{
    fn from(result: Result<DiscInfo, E>) -> Self {
        match result {
            Ok(info) => DiscInfoResult {
                value: info,
                err: SharedString::new(),
            },
            Err(e) => DiscInfoResult {
                value: DiscInfo::default(),
                err: e.to_shared_string(),
            },
        }
    }
}
