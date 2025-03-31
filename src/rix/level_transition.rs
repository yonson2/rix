use super::setup::LevelInfo;
use super::state::{AppState, TransitionState};
use bevy::prelude::*;

pub struct LevelTransitionPlugin;

impl Plugin for LevelTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LevelTransition), start_transition)
            .add_systems(
                Update,
                handle_transition_loading.run_if(in_state(TransitionState::Loading)),
            );
    }
}

fn start_transition(mut next_state: ResMut<NextState<TransitionState>>) {
    //TODO: animate the shapes, going out, setting alpha, etc...
    next_state.set(TransitionState::Loading);
}

fn handle_transition_loading(
    mut next_state: ResMut<NextState<AppState>>,
    mut level_info: ResMut<LevelInfo>,
) {
    level_info.level += 1;
    level_info.current = 0.;
    info!("Congrats, going to new level! {}", level_info.level);

    next_state.set(AppState::InGame);
}
