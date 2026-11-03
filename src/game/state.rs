use bevy::prelude::*;

use crate::screens::Screen;

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
    /// The line was hit: time stands still while it flashes, then the game over screen
    /// shows ([038]).
    Dying,
}

/// The pause menu or the settings panel inside it ([073]): the run stays.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(IsPaused = IsPaused::Paused)]
pub enum PausePanel {
    #[default]
    Menu,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(Phase = Phase::LevelTransition)]
pub enum TransitionState {
    #[default]
    Setup,
    Loading,
}
