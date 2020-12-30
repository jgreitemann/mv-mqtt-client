use std::env::args;

use app::App;

mod app;

fn main() {
    App::new().run(args().collect::<Vec<_>>());
}
