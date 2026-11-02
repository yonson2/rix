//! The look shared by every screen ([026]): the palette and the fonts now, UI widgets with
//! [036].

use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

pub mod palette;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<FontAssets>();
}

/// The fonts, loaded once on the loading screen ([027]).
#[derive(Resource, Asset, Clone, TypePath)]
pub struct FontAssets {
    #[dependency]
    pub main: Handle<Font>,
}

impl FromWorld for FontAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            main: asset_server.load("fonts/main.ttf"),
        }
    }
}
