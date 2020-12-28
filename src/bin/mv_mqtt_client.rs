extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;

fn build_ui(application: &gtk::Application) {
    let builder = gtk::Builder::from_file("res/ui/MainWindow.ui");
    let window : gtk::ApplicationWindow = builder.get_object("window").unwrap();
    window.set_application(Some(application));
}

fn main() {
    let application =
        gtk::Application::new(Some("io.github.jgreitemann.mv-mqtt-client"), Default::default())
            .expect("Initialization failed...");

    application.connect_activate(|app| {
        let icon_theme = gtk::IconTheme::get_default().unwrap();
        icon_theme.append_search_path("res/icons/actions");

        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
