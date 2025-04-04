use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_light_2d::light::PointLight2d;

use crate::{
    rix::{
        RixComponent,
        enemies::Enemy,
        events::{ActivateBasicBomb, DespawnEnemy},
        get_item_layer,
        setup::LevelInfo,
        state::IsPaused,
    },
    utils::generate_item_position,
};

use super::{Item, on_pressed_item};

pub struct BasicBombPlugin;
impl Plugin for BasicBombPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, activate.run_if(in_state(IsPaused::Running)));
    }
}

//TODO: move asset loading to setup step.
pub fn spawn_basic_bomb(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut level_info: ResMut<LevelInfo>,
) {
    let pos = generate_item_position(&window, &mut level_info.rng);
    commands
        .spawn((
            RixComponent,
            Item::BasicBomb,
            Sprite::from_image(asset_server.load("items/bomb_basic.png")),
            Collider::circle(50.),
            RigidBody::Dynamic,
            Mass(15.),
            GravityScale(5.),
            LinearDamping(0.33),
            Transform::from_xyz(pos.x, pos.y, 1.),
            get_item_layer(),
            PointLight2d {
                intensity: 2.5,
                radius: 50.0,
                ..default()
            },
        ))
        .observe(on_pressed_item);
}

pub fn activate(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut level_info: ResMut<LevelInfo>,
    mut events: EventReader<ActivateBasicBomb>,
) {
    for _ in events.read() {
        if let Some(victim) = level_info.rng.choice(enemies.iter()) {
            commands.trigger(DespawnEnemy(victim));
        }
    }
}
