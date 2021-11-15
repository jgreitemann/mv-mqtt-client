use app::App;

mod app;

fn main() {
    let res =
        gio::Resource::load("data/mv-mqtt-client.gresource").expect("Could not load resource :-(");
    gio::resources_register(&res);

    App::new().run();
}
