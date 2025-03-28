use bevy::prelude::*;

use super::state::AppState;

pub struct RixMenuPlugin;
impl Plugin for RixMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu);
    }
}

fn setup_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}
