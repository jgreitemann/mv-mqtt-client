mod app;
mod client;

use app::App;
use std::env::args;

fn main() {
    App::new().run(args().collect::<Vec<_>>());
}
