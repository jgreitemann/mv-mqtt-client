mod app;

use std::env::args;
use app::App;

fn main() {
    App::new().run(args().collect::<Vec<_>>());
}
