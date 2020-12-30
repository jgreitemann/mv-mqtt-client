mod app_ctrl;
mod client;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gio::prelude::*;
use gtk::prelude::*;

use app_ctrl::ApplicationController;
use client::Client;

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
