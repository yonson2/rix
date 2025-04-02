use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};

use super::{setup::LevelInfo, state::AppState};

pub struct ItemsPlugin;
impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_basic_bomb);
    }
}

#[derive(Component)]
pub struct BasicBomb;

//TODO: move to load step.
pub fn spawn_basic_bomb(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    spatial_query: SpatialQuery,
    asset_server: Res<AssetServer>,
    mut level_info: ResMut<LevelInfo>,
) {
    commands.spawn((
        BasicBomb,
        Sprite::from_image(asset_server.load("items/bomb_basic.png")),
        Transform::from_xyz(100., 0., 0.),
    ));
}
