// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use tap::Tap;

pub trait VecExt {
    fn appended_with(self, other: Self) -> Self;
    fn appended_to(self, other: Self) -> Self;
}

impl<T> VecExt for Vec<T> {
    fn appended_with(self, other: Self) -> Self {
        self.tap_mut(|v| v.extend(other))
    }

    fn appended_to(self, other: Self) -> Self {
        other.appended_with(self)
    }
}
