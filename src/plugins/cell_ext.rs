// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use marwood::cell::Cell;

pub trait CellExt {
    fn as_string(&self) -> Option<&str>;
    fn into_string(self) -> Option<String>;
    fn get_alist_value(&self, key: &str) -> Option<&Cell>;
}

impl CellExt for Cell {
    fn as_string(&self) -> Option<&str> {
        if let Cell::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    fn into_string(self) -> Option<String> {
        if let Cell::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    fn get_alist_value(&self, key: &str) -> Option<&Cell> {
        for item in self {
            if let Cell::Pair(k, v) = item
                && let Cell::Symbol(k) = k.as_ref()
                && k == key
            {
                return Some(v.as_ref());
            }
        }

        None
    }
}
