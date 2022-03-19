mod app;
mod cli;

use app::App;
use cli::CLIError;

fn main() -> Result<(), CLIError> {
    let res =
        gio::Resource::load("data/mv-mqtt-client.gresource").expect("Could not load resource :-(");
    gio::resources_register(&res);

    App::new()?.run();
    Ok(())
}
