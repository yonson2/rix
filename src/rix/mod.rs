use bevy::prelude::*;
use chain::ChainPlugin;
use mouse::MousePlugin;
use polygon::PolygonPlugin;
use snake::SnakePlugin;
use wall::WallPlugin;

pub mod chain;
pub mod mouse;
pub mod polygon;
pub mod snake;
pub mod wall;

pub struct RixPlugin;

impl Plugin for RixPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WallPlugin)
            .add_plugins(ChainPlugin)
            .add_plugins(SnakePlugin)
            .add_plugins(MousePlugin)
            .add_plugins(PolygonPlugin);
    }
}
