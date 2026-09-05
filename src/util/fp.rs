// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use tap::Tap;

pub trait VecExt<T> {
    fn extended_by(self, other: Self) -> Self;
    fn extends(self, other: Self) -> Self;
}

impl<T> VecExt<T> for Vec<T> {
    fn extended_by(self, other: Self) -> Self {
        self.tap_mut(|v| v.extend(other))
    }

    fn extends(self, other: Self) -> Self {
        other.extended_by(self)
    }
}
