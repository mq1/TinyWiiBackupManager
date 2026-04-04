// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DiscInfo, EmptyResult};
use slint::ToSharedString;

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

impl<E> From<Result<DiscInfo, E>> for DiscInfo
where
    E: ToSharedString,
{
    fn from(result: Result<DiscInfo, E>) -> Self {
        match result {
            Ok(info) => info,
            Err(e) => DiscInfo {
                err: e.to_shared_string(),
                ..DiscInfo::default()
            },
        }
    }
}
