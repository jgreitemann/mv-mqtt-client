#[macro_use]
mod macros;
mod app_ctrl;
mod client;
mod helpers;

use std::cell::RefCell;
use std::sync::Arc;

use gio::prelude::*;

use crate::app::client::Subscription;
use app_ctrl::ApplicationController;
use client::Client;
use mvjson::*;

#[allow(dead_code)]
pub struct App {
    application: gtk::Application,
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

        let client = Arc::new(RefCell::new(Client::new("tcp://localhost:1883")));

        let app_ctrl = Arc::new(RefCell::new(ApplicationController::new(
            &application,
            Arc::downgrade(&client),
        )));

        let (current_tx, current_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (rlist_tx, rlist_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        ApplicationController::connect_callbacks(&application, &app_ctrl, current_rx, rlist_rx);

        client.borrow_mut().update_subscriptions(vec![
            Subscription::<Current, _>::boxed_new("merlic/current/json", move |c| {
                current_tx.send(c).unwrap()
            }),
            Subscription::<Vec<Recipe>, _>::boxed_new("merlic/recipes/json", move |rlist| {
                rlist_tx.send(rlist).unwrap()
            }),
        ]);

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
