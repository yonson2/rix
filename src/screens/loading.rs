//! The first screen: it waits until every asset collection (`LoadResource`) is loaded,
//! then shows the title ([027]).

use bevy::prelude::*;

use super::Screen;
use crate::asset_tracking::ResourceHandles;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        enter_title.run_if(in_state(Screen::Loading).and_then(all_assets_loaded)),
    );
}

fn enter_title(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
    resource_handles.is_all_done()
}
