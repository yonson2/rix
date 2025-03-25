use bevy::prelude::*;
use rix::{base::BasePlugin, rix::RixPlugin};

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(BasePlugin);
    app.add_plugins(RixPlugin);
    app.run()
}
