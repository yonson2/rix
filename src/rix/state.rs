use bevy::prelude::*;

/// The screen on show. Entities of a screen carry `DespawnOnExit(Screen::..)` ([026]).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum Screen {
    #[default]
    Setup,
    Title,
    Gameplay,
    GameOver,
}

/// The gameplay screen plays a level or moves to the next one. Walls and claimed shapes
/// live for the screen; enemies, items, lines and the HUD for one level.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(Screen = Screen::Gameplay)]
pub enum Phase {
    #[default]
    Playing,
    LevelTransition,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(Phase = Phase::Playing)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(Phase = Phase::LevelTransition)]
pub enum TransitionState {
    #[default]
    Setup,
    Loading,
}
