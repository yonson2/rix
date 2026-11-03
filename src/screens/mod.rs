//! The screens and the state that picks one ([026]).

use bevy::prelude::*;

pub mod game_over;
pub mod loading;
pub mod seed;
pub mod settings;
pub mod title;

/// The screen on show. Entities of a screen carry `DespawnOnExit(Screen::..)`.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Screen {
    #[default]
    Loading,
    Title,
    Settings,
    Seed,
    Gameplay,
    GameOver,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>().add_plugins((
        loading::plugin,
        title::plugin,
        settings::plugin,
        seed::plugin,
        game_over::plugin,
    ));
}
