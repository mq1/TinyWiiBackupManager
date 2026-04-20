// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::AppWindow;
use slint::{ComponentHandle, Weak};
use std::cell::{Ref, RefCell};

pub struct Mirrored<T> {
    inner: RefCell<T>,
    weak: Weak<AppWindow>,
    update_fn: fn(&AppWindow, T),
}

impl<T: Clone> Mirrored<T> {
    pub fn new(inner: T, app: &AppWindow, update_fn: fn(&AppWindow, T)) -> Self {
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
        E: Fn(&mut T),
    {
        f(&mut self.inner.borrow_mut());
        self.sync();
    }

    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
        self.sync();
    }
}
