use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    Menu,
    #[default]
    InGame,
}

// In this case, instead of deriving `States`, we derive `SubStates`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
// And we need to add an attribute to let us know what the source state is
// and what value it needs to have. This will ensure that unless we're
// in [`AppState::InGame`], the [`IsPaused`] state resource
// will not exist.
#[source(AppState = AppState::InGame)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}
