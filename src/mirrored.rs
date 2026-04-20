// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint::{ComponentHandle, StrongHandle, Weak};
use std::cell::{Ref, RefCell};

pub struct Mirrored<T, A: StrongHandle> {
    inner: RefCell<T>,
    weak: Weak<A>,
    update_fn: fn(&A, T),
}

impl<T, A> Mirrored<T, A>
where
    T: Clone,
    A: StrongHandle + ComponentHandle,
{
    pub fn new(inner: T, app: &A, update_fn: fn(&A, T)) -> Self {
        let m = Mirrored {
            inner: RefCell::new(inner),
            weak: app.as_weak(),
            update_fn,
        };

        m.sync();
        m
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    fn sync(&self) {
        let app = self.weak.upgrade().unwrap();
        (self.update_fn)(&app, self.inner.borrow().clone());
    }

    pub fn edit<E>(&self, f: E)
    where
        E: Fn(&mut T) -> (),
    {
        f(&mut self.inner.borrow_mut());
        self.sync();
    }
}
