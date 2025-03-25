use bevy::prelude::*;
use chain::ChainPlugin;
use mouse::MousePlugin;
use polygon::PolygonPlugin;
use snake::SnakePlugin;
use state::{AppState, IsPaused};
use wall::WallPlugin;

pub mod chain;
pub mod mouse;
pub mod polygon;
pub mod snake;
pub mod state;
pub mod wall;

pub struct RixPlugin;

/// `RixComponent` is a marker component that is used to tag all
/// entities in the game that should be despawned when we stop
/// playing the game and want to transition to other screens.
#[derive(Component)]
pub struct RixComponent;

impl Plugin for RixPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<IsPaused>()
            .add_plugins(WallPlugin)
            .add_plugins(ChainPlugin)
            .add_plugins(SnakePlugin)
            .add_plugins(MousePlugin)
            .add_plugins(PolygonPlugin)
            .add_systems(OnExit(AppState::InGame), despawn_screen::<RixComponent>);
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
