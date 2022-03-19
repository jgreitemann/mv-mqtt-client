#[macro_use]
mod app_ctrl;
mod client;
mod helpers;

use libadwaita as adw;

use std::cell::RefCell;
use std::sync::Arc;

use adw::prelude::*;
use glib::clone;

use crate::app::client::Subscription;
use crate::cli::*;
use app_ctrl::{ApplicationController, Message};
use clap::Parser;
use client::Client;
use mvjson::*;

#[allow(dead_code)]
pub struct App {
    application: adw::Application,
    client: Arc<RefCell<Client>>,
    app_ctrl: Arc<RefCell<ApplicationController>>,
}

impl App {
    pub fn new() -> Result<Self, CLIError> {
        let args = Args::parse();

        let application = adw::Application::new(
            Some("io.github.jgreitemann.mv-mqtt-client"),
            gio::ApplicationFlags::empty(),
        );

        let client = Arc::new(RefCell::new(Client::new(&format!(
            "tcp://{}:{}",
            args.host, args.port
        ))?));

        let app_ctrl = Arc::new(RefCell::new(ApplicationController::new(&application)));

        let (message_sender, message_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (action_sender, action_receiver): (glib::Sender<Action>, _) =
            glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let action_topic = format!("{}/action/json", args.prefix);
        action_receiver.attach(
            None,
            clone!(@strong client => move |action| {
                client.borrow().publish(&action_topic, &action);
                glib::Continue(true)
            }),
        );

        ApplicationController::connect_callbacks(
            &application,
            &app_ctrl,
            message_receiver,
            action_sender,
        );

        use Message::*;
        client.borrow_mut().update_subscriptions(vec![
            Subscription::<SystemStatus, _>::boxed_new(&format!("{}/status/json", args.prefix), {
                let message_sender = message_sender.clone();
                move |status| message_sender.send(SystemStatusUpdate(status)).unwrap()
            }),
            Subscription::<Vec<Recipe>, _>::boxed_new(&format!("{}/recipes/json", args.prefix), {
                let message_sender = message_sender.clone();
                move |rlist| message_sender.send(RecipeListUpdate(rlist)).unwrap()
            }),
            Subscription::<VisionResult, _>::boxed_new(
                &format!("{}/recipes/+/result/json", args.prefix),
                {
                    let message_sender = message_sender.clone();
                    move |result| message_sender.send(NewResult(result)).unwrap()
                },
            ),
        ])?;

        Ok(App {
            application,
            client,
            app_ctrl,
        })
    }

    pub fn run(self: &App) {
        self.application.run_with_args(&[] as &[String]);
    }
}
