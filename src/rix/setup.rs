use bevy::prelude::*;

use super::{enemies::register_enemy_systems, state::AppState};

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::Setup),
            (setup_world, finish_setup).chain(),
        );
    }
}

fn setup_world(world: &mut World) {
    register_enemy_systems(world);
}

fn finish_setup(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}
