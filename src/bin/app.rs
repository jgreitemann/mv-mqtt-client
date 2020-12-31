#[macro_use]
mod macros;
mod app_ctrl;
mod client;

use std::cell::RefCell;
use std::rc::Rc;

use gio::prelude::*;

use crate::app::client::Subscription;
use app_ctrl::ApplicationController;
use client::Client;
use mvjson::Monitor;

#[allow(dead_code)]
pub struct App {
    application: gtk::Application,
    client: Rc<RefCell<Client>>,
    app_ctrl: Rc<RefCell<ApplicationController>>,
}

impl App {
    pub fn new() -> Self {
        let application = gtk::Application::new(
            Some("io.github.jgreitemann.mv-mqtt-client"),
            Default::default(),
        )
        .expect("Initialization failed...");

        let client = Rc::new(RefCell::new(Client::new("tcp://localhost:1883")));

        let app_ctrl = Rc::new(RefCell::new(ApplicationController::new(
            &application,
            Rc::downgrade(&client),
        )));
        ApplicationController::connect_callbacks(&application, &app_ctrl);

        client
            .borrow_mut()
            .update_subscriptions(vec![Subscription::<Monitor, _>::boxed_new(
                "merlic/monitor/json",
                weak!(&app_ctrl => move |m| app_ctrl.upgrade().unwrap().borrow().change_state(m.state)),
            )]);

        App {
            application,
            client,
            app_ctrl,
        }
    }

    pub fn run(self: &App, args: Vec<String>) {
        self.application.run(&args);
    }
}
