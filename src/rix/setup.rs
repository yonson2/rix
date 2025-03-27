use bevy::prelude::*;

use super::{enemies::register_enemy_systems, events, state::AppState};

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::DrawLine>()
            .add_event::<events::BuildPolygon>()
            .add_event::<events::ClearLines>()
            .add_event::<events::DrawPolygon>()
            .add_event::<events::Die>()
            .add_event::<events::DespawnEnemy>()
            .add_systems(
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
