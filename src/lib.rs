pub mod asset_tracking;
pub mod base;
pub mod camera;
pub mod common;
pub mod crt;
#[cfg(feature = "dev_tools")]
pub mod dev_tools;
pub mod game;
pub mod screens;
pub mod shape;
pub mod theme;
pub mod utils;

use bevy::prelude::*;

/// The game app, shared by the desktop binary and the mobile library (`mobile/`).
pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        base::plugin,
        asset_tracking::plugin,
        theme::plugin,
        screens::plugin,
        game::plugin,
    ));
    #[cfg(feature = "dev_tools")]
    app.add_plugins(dev_tools::plugin);
    app
}
