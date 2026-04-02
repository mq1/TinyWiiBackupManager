// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint::{Model, ModelRc, VecModel};

pub trait MyModelExt<T> {
    fn push(&self, item: T);
    fn pop(&self) -> Option<T>;
    fn pop_first(&self) -> Option<T>;
    fn remove(&self, index: i32) -> T;
    fn append(&self, other: Self);
    fn sort_by(&self, compare: impl FnMut(&T, &T) -> std::cmp::Ordering);
}

impl<T: 'static> MyModelExt<T> for ModelRc<T> {
    fn push(&self, item: T) {
        let v = self.as_any().downcast_ref::<VecModel<T>>().unwrap();
        v.push(item);
    }

    fn pop(&self) -> Option<T> {
        let i = self.row_count();
        if i == 0 {
            return None;
        }

        let v = self.as_any().downcast_ref::<VecModel<T>>().unwrap();
        Some(v.remove(i - 1))
    }

    fn pop_first(&self) -> Option<T> {
        if self.row_count() == 0 {
            return None;
        }

        let v = self.as_any().downcast_ref::<VecModel<T>>().unwrap();
        Some(v.remove(0))
    }

    #[allow(clippy::cast_sign_loss)]
    fn remove(&self, index: i32) -> T {
        let v = self.as_any().downcast_ref::<VecModel<T>>().unwrap();
        v.remove(index as usize)
    }

    fn append(&self, other: Self) {
        while let Some(item) = other.pop() {
            self.push(item);
        }
    }

    fn sort_by(&self, compare: impl FnMut(&T, &T) -> std::cmp::Ordering) {
        let mut v = self.iter().collect::<Vec<_>>();
        v.sort_unstable_by(compare);
        self.as_any()
            .downcast_ref::<VecModel<T>>()
            .unwrap()
            .set_vec(v);
    }
}
