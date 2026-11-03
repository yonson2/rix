use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::Sfx,
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
    // Each bomb of a tick takes an enemy no other bomb took; the despawn waits ([083]).
    let mut taken = vec![];
    for _ in events.read() {
        let free: Vec<_> = enemies.iter().filter(|(e, _)| !taken.contains(e)).collect();
        if let Some((victim, transform)) = pick(&mut rng, &free) {
            taken.push(victim);
            commands.trigger(DespawnEnemy(victim));
            commands.trigger(Shake(BOMB_SHAKE));
            commands.trigger(Sfx::Bomb);
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

/// A random item. `choice` draws a `usize`, which 32-bit wasm draws differently; this draws
/// a `u64`, the same as `choice` on 64-bit targets ([075]).
fn pick<T: Copy>(rng: &mut fastrand::Rng, items: &[T]) -> Option<T> {
    (!items.is_empty()).then(|| items[rng.u64(..items.len() as u64) as usize])
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    /// The enemies the bombs took and the bomb sounds they played.
    #[derive(Resource, Default)]
    struct Victims(Vec<Entity>, Vec<Sfx>);

    fn victims_of(bombs: usize, enemies: usize) -> (Vec<Entity>, Vec<Entity>) {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ExplosionMaterial>>();
        world.init_resource::<Messages<ActivateBasicBomb>>();
        world.init_resource::<Time>();
        world.init_resource::<Victims>();
        world.insert_resource(GameRng::new(1));
        world.add_observer(|hit: On<DespawnEnemy>, mut victims: ResMut<Victims>| {
            victims.0.push(hit.0);
        });
        world.add_observer(|sfx: On<Sfx>, mut victims: ResMut<Victims>| victims.1.push(*sfx));
        let enemies: Vec<Entity> = (0..enemies)
            .map(|_| world.spawn((Enemy, Transform::default())).id())
            .collect();
        for _ in 0..bombs {
            world.write_message(ActivateBasicBomb);
        }
        world.run_system_once(activate).unwrap();
        world.flush();
        let Victims(victims, sounds) = world.resource::<Victims>();
        assert_eq!(
            *sounds,
            vec![Sfx::Bomb; victims.len()],
            "a boom per kill ([042])"
        );
        (victims.clone(), enemies)
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn a_pick_draws_as_choice_on_64_bit() {
        for seed in 0..50 {
            for len in 1..12 {
                let items: Vec<usize> = (0..len).collect();
                let (mut a, mut b) = (
                    fastrand::Rng::with_seed(seed),
                    fastrand::Rng::with_seed(seed),
                );
                assert_eq!(pick(&mut a, &items), b.choice(items.iter().copied()));
                assert_eq!(a.get_seed(), b.get_seed(), "the same draws");
            }
        }
        assert_eq!(pick::<u8>(&mut fastrand::Rng::with_seed(1), &[]), None);
    }

    #[test]
    fn two_bombs_in_one_tick_take_two_enemies() {
        let (mut victims, mut enemies) = victims_of(2, 2);
        victims.sort();
        enemies.sort();
        assert_eq!(victims, enemies);
        let (victims, _) = victims_of(2, 1);
        assert_eq!(victims.len(), 1, "the second bomb finds no enemy left");
    }
}
