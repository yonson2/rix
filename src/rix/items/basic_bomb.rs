use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    rix::{
        RixComponent,
        effects::{ExplosionMaterial, GlowMaterial},
        enemies::{Enemy, square::SQUARE_SIZE},
        events::{ActivateBasicBomb, DespawnEnemy},
        get_item_layer,
        level::DespawnAfter,
        setup::LevelInfo,
        state::IsPaused,
    },
    utils::generate_enemy_position,
};

use super::Item;

pub struct BasicBombPlugin;
impl Plugin for BasicBombPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, activate.run_if(in_state(IsPaused::Running)));
    }
}

//TODO: move asset loading to setup step.
pub fn spawn_basic_bomb(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut level_info: ResMut<LevelInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
    spatial_query: SpatialQuery,
) {
    const BOMB_SIZE: f32 = 100.0;

    let collider = Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE);
    //TODO: rename.
    let pos = generate_enemy_position(&spatial_query, &mut level_info.rng, &collider);
    let bomb_texture: Handle<Image> = asset_server.load("items/bomb_basic.png");
    commands.spawn((
        RixComponent,
        Item::BasicBomb,
        Name::new("GlowBomb"),
        Mesh2d(meshes.add(Rectangle::new(BOMB_SIZE, BOMB_SIZE))),
        MeshMaterial2d(materials.add(GlowMaterial {
            intensity: 2.5,
            // --------------------------------------
            texture: bomb_texture,
        })),
        collider,
        RigidBody::Dynamic,
        Mass(0.),
        GravityScale(0.),
        LinearDamping(0.33),
        Transform::from_xyz(pos.x, pos.y, 1.),
        DespawnAfter::from_seconds(7.),
        get_item_layer(),
    ));
}

pub fn activate(
    mut commands: Commands,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    time: Res<Time>,
    mut level_info: ResMut<LevelInfo>,
    mut events: MessageReader<ActivateBasicBomb>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in events.read() {
        if let Some((victim, transform)) = level_info.rng.choice(enemies.iter()) {
            commands.trigger(DespawnEnemy(victim));
            let pos = transform.translation.truncate();
            let bundle = (
                Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE * 2., SQUARE_SIZE * 2.))),
                MeshMaterial2d(explosion_materials.add(ExplosionMaterial {
                    time: 0.0,
                    start_time: time.elapsed_secs(),
                })),
                Transform::from_xyz(pos.x, pos.y, 2.),
                DespawnAfter::from_seconds(2.),
            );
            commands.spawn(bundle);
        }
    }
}

// Moved blink functionality to items/mod.rs
