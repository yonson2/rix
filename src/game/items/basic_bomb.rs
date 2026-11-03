use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    game::{
        AppSystems, PausableSystems,
        effects::{ExplosionMaterial, GlowMaterial},
        enemies::{Enemy, square::SQUARE_SIZE},
        events::{ActivateBasicBomb, DespawnEnemy},
        fx::{BOMB_SHAKE, Shake},
        get_item_layer,
        level::DespawnAfter,
        setup::GameRng,
        state::Phase,
        wall::Field,
    },
    theme::palette::NEON_YELLOW,
    utils::generate_enemy_position,
};

use super::Item;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<ItemAssets>();
    app.add_systems(
        FixedUpdate,
        activate.in_set(AppSystems::Update).in_set(PausableSystems),
    );
}

/// The item images, loaded once on the loading screen ([027]).
#[derive(Resource, Asset, Clone, TypePath)]
pub struct ItemAssets {
    #[dependency]
    bomb_basic: Handle<Image>,
}

impl FromWorld for ItemAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            bomb_basic: asset_server.load("items/bomb_basic.png"),
        }
    }
}

pub fn spawn_basic_bomb(
    mut commands: Commands,
    item_assets: Res<ItemAssets>,
    mut rng: ResMut<GameRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
    spatial_query: SpatialQuery,
    field: Res<Field>,
) {
    const BOMB_SIZE: f32 = 100.0;

    let collider = Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE);
    //TODO: rename.
    let Some(pos) = generate_enemy_position(&spatial_query, &mut rng, &collider, &field) else {
        return;
    };
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        Item::BasicBomb,
        Name::new("GlowBomb"),
        Mesh2d(meshes.add(Rectangle::new(BOMB_SIZE, BOMB_SIZE))),
        MeshMaterial2d(materials.add(GlowMaterial {
            intensity: 2.5,
            outline: NEON_YELLOW.to_linear(),
            // --------------------------------------
            texture: item_assets.bomb_basic.clone(),
        })),
        collider,
        RigidBody::Dynamic,
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
    mut rng: ResMut<GameRng>,
    mut events: MessageReader<ActivateBasicBomb>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for _ in events.read() {
        if let Some((victim, transform)) = rng.choice(enemies.iter()) {
            commands.trigger(DespawnEnemy(victim));
            commands.trigger(Shake(BOMB_SHAKE));
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
