mod asset_tracking;
mod audio;
mod base;
mod camera;
mod common;
pub mod crt;
#[cfg(feature = "dev_tools")]
mod dev_tools;
mod game;
mod haptics;
mod screens;
mod shape;
pub mod theme;
mod utils;

use bevy::prelude::*;

/// The game app, shared by the desktop binary and the mobile library (`mobile/`).
pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        base::plugin,
        asset_tracking::plugin,
        audio::plugin,
        theme::plugin,
        screens::plugin,
        game::plugin,
        haptics::plugin,
    ));
    #[cfg(feature = "dev_tools")]
    app.add_plugins(dev_tools::plugin);
    app
}
