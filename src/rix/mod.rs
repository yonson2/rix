use avian2d::prelude::*;
use bevy::prelude::*;
use enemies::EnemiesPlugin;
use game_over::GameOverPlugin;
use items::ItemsPlugin;
use level::LevelPlugin;
use pause::PausePlugin;
use polygon::PolygonPlugin;
use setup::SetupPlugin;
use state::{IsPaused, Phase, Screen, TransitionState};
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
pub mod pause;
pub mod polygon;
pub mod setup;
pub mod state;
pub mod ui;
pub mod user_input;
pub mod wall;

pub struct RixPlugin;

/// The order of the gameplay systems in `Update` ([026]): timers tick, then input is read,
/// then the rest reacts to it in the same frame.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AppSystems {
    TickTimers,
    RecordInput,
    Update,
}

/// Gameplay systems: they run only in `IsPaused::Running`, so they stop while paused and
/// outside `Phase::Playing`.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PausableSystems;

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
        app.init_state::<Screen>()
            .add_sub_state::<Phase>()
            .add_sub_state::<TransitionState>()
            .add_sub_state::<IsPaused>()
            .configure_sets(
                Update,
                (
                    AppSystems::TickTimers,
                    AppSystems::RecordInput,
                    AppSystems::Update,
                )
                    .chain(),
            )
            .configure_sets(Update, PausableSystems.run_if(in_state(IsPaused::Running)))
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
            .add_plugins(PausePlugin);
    }
}
