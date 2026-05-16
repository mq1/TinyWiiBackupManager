// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint::SharedString;
use std::fmt::{Display, Write};

pub const GIB: f32 = 1024. * 1024. * 1024.;
pub const MIB: f32 = 1024. * 1024.;

pub fn display_list<T>(list: &[T]) -> SharedString
where
    T: Display,
{
    let mut s = SharedString::new();

    let last_i = list.len() - 1;
    for (i, value) in list.iter().enumerate() {
        write!(&mut s, "{value}").unwrap();

        if i != last_i {
            s.push_str(", ");
        }
    }

    s
}
