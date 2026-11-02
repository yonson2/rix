use avian2d::prelude::*;
use bevy::prelude::*;
use state::{IsPaused, Phase, TransitionState};

pub mod effects;
pub mod enemies;
pub mod events;
pub mod items;
pub mod level;
pub mod pause;
pub mod polygon;
pub mod setup;
pub mod state;
pub mod ui;
pub mod user_input;
pub mod wall;

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

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<Phase>()
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
        .add_plugins((
            setup::plugin,
            wall::plugin,
            user_input::plugin,
            polygon::plugin,
            enemies::plugin,
            items::plugin,
            level::plugin,
            ui::plugin,
            pause::plugin,
        ));
}
