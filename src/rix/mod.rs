use avian2d::prelude::*;
use bevy::prelude::*;
use enemies::EnemiesPlugin;
use game_over::GameOverPlugin;
use items::ItemsPlugin;
use level::LevelPlugin;
use polygon::PolygonPlugin;
use setup::SetupPlugin;
use state::{AppState, IsPaused, TransitionState};
use ui::UiPlugin;
use user_input::UserInputPlugin;
use wall::WallPlugin;

use crate::menu::MenuPlugin;

pub mod effects;
pub mod enemies;
pub mod events;
pub mod game_over;
pub mod items;
pub mod level;
pub mod polygon;
pub mod setup;
pub mod state;
pub mod ui;
pub mod user_input;
pub mod wall;

pub struct RixPlugin;

/// `RixComponent` is a marker component that is used to tag all
/// entities in the game that should be despawned when we stop
/// playing the game and want to transition to other screens.
#[derive(Component)]
pub struct RixComponent;

#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Player,
    Enemy,
    Shape,
    Item,
}

pub fn get_player_layer() -> CollisionLayers {
    CollisionLayers::new(
        GameLayer::Player,
        [GameLayer::Default, GameLayer::Enemy, GameLayer::Item],
    )
}
pub fn get_enemy_layer() -> CollisionLayers {
    CollisionLayers::new(
        GameLayer::Enemy,
        [GameLayer::Default, GameLayer::Enemy, GameLayer::Player],
    )
}

pub fn get_item_layer() -> CollisionLayers {
    CollisionLayers::new(GameLayer::Item, [GameLayer::Default, GameLayer::Player])
}

impl Plugin for RixPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<TransitionState>()
            .add_sub_state::<IsPaused>()
            .add_plugins(SetupPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(GameOverPlugin)
            .add_plugins(WallPlugin)
            .add_plugins(UserInputPlugin)
            .add_plugins(PolygonPlugin)
            .add_plugins(EnemiesPlugin)
            .add_plugins(ItemsPlugin)
            .add_plugins(LevelPlugin)
            .add_plugins(UiPlugin)
            .add_systems(OnExit(AppState::InGame), despawn_screen::<RixComponent>);
    }
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
