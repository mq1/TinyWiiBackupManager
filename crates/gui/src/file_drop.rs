// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Dispatcher, Message};
use slint::winit_030::{CustomApplicationHandler, EventResult, winit};
use slint::{ToSharedString, Weak};
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

pub struct FileDropHandler {
    dispatcher: Rc<RefCell<MaybeUninit<Weak<Dispatcher<'static>>>>>,
}

impl FileDropHandler {
    pub fn new() -> (Self, Rc<RefCell<MaybeUninit<Weak<Dispatcher<'static>>>>>) {
        let dispatcher = Rc::new(RefCell::new(MaybeUninit::uninit()));
        (
            Self {
                dispatcher: dispatcher.clone(),
            },
            dispatcher,
        )
    }
}

impl CustomApplicationHandler for FileDropHandler {
    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _winit_window: Option<&winit::window::Window>,
        _slint_window: Option<&slint::Window>,
        event: &WindowEvent,
    ) -> EventResult {
        if let WindowEvent::DroppedFile(path) = event {
            let borrowed = self.dispatcher.borrow();
            let weak = unsafe { borrowed.assume_init_ref() };
            let dispatcher = weak.upgrade().unwrap();

            let payload = path.to_string_lossy().to_shared_string();
            dispatcher.invoke_dispatch(Message::FileDropped, payload);
        }

        EventResult::Propagate
    }
}
