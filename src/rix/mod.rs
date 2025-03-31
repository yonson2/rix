use bevy::prelude::*;
use chain::ChainPlugin;
use enemies::EnemiesPlugin;
use level::LevelPlugin;
use menu::RixMenuPlugin;
use polygon::PolygonPlugin;
use setup::SetupPlugin;
use state::{AppState, IsPaused, TransitionState};
use user_input::UserInputPlugin;
use wall::WallPlugin;

pub mod chain;
pub mod enemies;
pub mod events;
pub mod level;
pub mod menu;
pub mod polygon;
pub mod setup;
pub mod state;
pub mod user_input;
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
            .add_sub_state::<TransitionState>()
            .add_sub_state::<IsPaused>()
            .add_plugins(SetupPlugin)
            .add_plugins(WallPlugin)
            .add_plugins(ChainPlugin)
            .add_plugins(UserInputPlugin)
            .add_plugins(PolygonPlugin)
            .add_plugins(RixMenuPlugin)
            .add_plugins(EnemiesPlugin)
            .add_plugins(LevelPlugin)
            .add_systems(OnExit(AppState::InGame), despawn_screen::<RixComponent>);
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
