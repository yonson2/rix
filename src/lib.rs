pub mod base;
pub mod camera;
pub mod colors;
pub mod common;
pub mod crt;
#[cfg(feature = "dev_tools")]
pub mod dev_tools;
pub mod menu;
pub mod rix;
pub mod shape;
pub mod utils;

use bevy::prelude::*;

/// The game app, shared by the desktop binary and the mobile library (`mobile/`).
pub fn app() -> App {
    let mut app = App::new();
    app.add_plugins((base::BasePlugin, rix::RixPlugin));
    #[cfg(feature = "dev_tools")]
    app.add_plugins(dev_tools::DevToolsPlugin);
    app
}
