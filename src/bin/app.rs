#[macro_use]
mod macros;
mod app_ctrl;
mod client;

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

use gio::prelude::*;

use crate::app::client::Subscription;
use app_ctrl::ApplicationController;
use client::Client;
use mvjson::*;

#[allow(dead_code)]
pub struct App {
    application: gtk::Application,
    current: Arc<Mutex<Current>>,
    client: Arc<RefCell<Client>>,
    app_ctrl: Arc<RefCell<ApplicationController>>,
}

impl App {
    pub fn new() -> Self {
        let application = gtk::Application::new(
            Some("io.github.jgreitemann.mv-mqtt-client"),
            Default::default(),
        )
        .expect("Initialization failed...");

        let current = Arc::new(Mutex::new(Current {
            state: State::Preoperational,
            mode: None,
            recipe_id: None,
            job_id: None,
        }));
        let current_weak = std::sync::Arc::downgrade(&current);

        let client = Arc::new(RefCell::new(Client::new("tcp://localhost:1883")));

        let app_ctrl = Arc::new(RefCell::new(ApplicationController::new(
            &application,
            Arc::downgrade(&client),
        )));
        ApplicationController::connect_callbacks(&application, &app_ctrl, &current);

        client
            .borrow_mut()
            .update_subscriptions(vec![Subscription::<Current, _>::boxed_new(
                "merlic/monitor/json",
                weak!(&app_ctrl => move |c| {
                    let strong = current_weak.upgrade().unwrap();
                    let mut guard = strong.lock().unwrap();
                    *guard.deref_mut() = c;

                    unsafe { gdk_sys::gdk_threads_init(); }
                    app_ctrl.upgrade().unwrap().borrow_mut().update_ui(guard.deref());
                    unsafe { gdk_sys::gdk_threads_leave(); }
                }),
            )]);

        App {
            application,
            current,
            client,
            app_ctrl,
        }
    }

    pub fn run(self: &App, args: Vec<String>) {
        self.application.run(&args);
    }
}
