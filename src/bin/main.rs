mod app;
mod cli;
mod config;

use app::App;
use cli::CLIError;
use config::RESOURCES_FILE;

fn main() -> Result<(), CLIError> {
    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load resource :-(");
    gio::resources_register(&res);

    App::new()?.run();
    Ok(())
}
