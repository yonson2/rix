use bevy::prelude::*;
use rix::{base::BasePlugin, rix::RixPlugin};

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(BasePlugin);
    app.add_plugins(RixPlugin);
    #[cfg(feature = "dev_tools")]
    app.add_plugins(rix::dev_tools::DevToolsPlugin);
    app.run()
}
