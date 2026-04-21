// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint::{StrongHandle, Weak};
use std::cell::{Ref, RefCell};

pub struct Mirrored<T> {
    inner: RefCell<T>,
    update_fn: RefCell<Box<dyn Fn(T)>>,
}

impl<T: Clone> Mirrored<T> {
    pub fn new(inner: T) -> Self {
        Mirrored {
            inner: RefCell::new(inner),
            update_fn: RefCell::new(Box::new(|_| {})),
        }
    }

    pub fn wire<S: StrongHandle + 'static>(&self, weak: Weak<S>, set_fn: impl Fn(&S, T) + 'static) {
        *self.update_fn.borrow_mut() = Box::new(move |new| {
            set_fn(&weak.upgrade().unwrap(), new);
        });
        (self.update_fn.borrow())(self.inner.borrow().clone());
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    pub fn edit<E>(&self, f: E)
    where
        E: Fn(&mut T),
    {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner);
        (self.update_fn.borrow())(inner.clone());
    }

    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value.clone();
        (self.update_fn.borrow())(value);
    }
}
