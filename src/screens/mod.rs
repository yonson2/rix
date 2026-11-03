//! The screens and the state that picks one ([026]).

use bevy::prelude::*;

pub mod game_over;
pub mod loading;
pub mod seed;
pub mod settings;
pub mod skins;
pub mod title;
pub mod wiki;

/// The screen on show. Entities of a screen carry `DespawnOnExit(Screen::..)`.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Screen {
    #[default]
    Loading,
    Title,
    Settings,
    Seed,
    Wiki,
    Skins,
    Gameplay,
    /// One frame between two runs: the pause menu's RESTART ([073]) leaves `Gameplay`, so
    /// its sub-states start fresh (setting `Gameplay` again keeps them).
    Restart,
    GameOver,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>().add_plugins((
        loading::plugin,
        title::plugin,
        settings::plugin,
        seed::plugin,
        game_over::plugin,
        wiki::plugin,
        skins::plugin,
    ));
}
