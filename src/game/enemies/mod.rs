use arrow::{Arrow, spawn_arrow};
use avian2d::prelude::*;
use ball::spawn_ball;
use bevy::prelude::*;
use rng::{EnemyKind, SPAWN_TABLE, calculate_enemy_count, pick_kind};
use splitter::Splitter;
use square::{SQUARE_SIZE, Square, spawn_square};

use super::{
    AppSystems, PausableSystems,
    effects::PixelExplosionMaterial,
    events::{self, DespawnEnemy, IncreasePoints},
    fx::Burst,
    level::DespawnAfter,
    polygon::TempLineSegment,
    setup::{GameRng, Run},
    state::{IsPaused, Phase},
};
use crate::{audio::Sfx, screens::Screen, shape::Shape, theme::palette::NEON_WHITE};

pub mod arrow;
pub mod ball;
pub mod rng;
pub mod shooter;
pub mod splitter;
pub mod square;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        ball::plugin,
        square::plugin,
        arrow::plugin,
        splitter::plugin,
        shooter::plugin,
    ));
    app.add_systems(OnEnter(Phase::Playing), spawn_enemies);
    app.add_systems(
        FixedUpdate,
        (handle_enemy_collision, handle_die)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    // Not pausable: a corpse of the claim that clears the level plays during the transition.
    app.add_systems(Update, animate_corpses);

    app.add_observer(on_despawn_enemy);
}

#[derive(Component)]
/// An `Enemy` is a marker component to associate entities
/// as enemies.
pub struct Enemy;

pub(super) fn spawn_enemies(world: &mut World) {
    let level = world.resource::<Run>().level;
    let enemy_count = calculate_enemy_count(level, &mut world.resource_mut::<GameRng>());

    for _ in 0..enemy_count {
        let Some(kind) = pick_kind(SPAWN_TABLE, level, &mut world.resource_mut::<GameRng>()) else {
            return;
        };
        let spawned = match kind {
            EnemyKind::Ball => world.run_system_cached(spawn_ball),
            EnemyKind::Square => world.run_system_cached(spawn_square),
            EnemyKind::Arrow => world.run_system_cached(spawn_arrow),
            EnemyKind::Splitter => world.run_system_cached(splitter::spawn_splitter),
            EnemyKind::Shooter => world.run_system_cached(shooter::spawn_shooter),
        };
        if let Err(e) = spawned {
            error!("Error running system: {e}");
        }
    }
}

fn handle_enemy_collision(
    mut collision_event_reader: MessageReader<CollisionStart>,
    ball_q: Query<Entity, With<Enemy>>,
    lines_q: Query<Entity, With<TempLineSegment>>,
    mut events: MessageWriter<events::Die>,
) {
    let enemies: Vec<Entity> = ball_q.iter().collect();
    let line: Vec<Entity> = lines_q.iter().collect();

    for CollisionStart {
        collider1: entity1,
        collider2: entity2,
        ..
    } in collision_event_reader.read()
    {
        let enemies = enemies.contains(entity1) || enemies.contains(entity2);
        let line = line.contains(entity1) || line.contains(entity2);
        if enemies && line {
            events.write(events::Die);
        }
    }
}

fn handle_die(mut events: MessageReader<events::Die>, mut next: ResMut<NextState<IsPaused>>) {
    for _ in events.read() {
        next.set(IsPaused::Dying);
    }
}

/// A dead enemy's look for a moment ([039]): a copy of its mesh or shape above the claim.
#[derive(Component)]
struct Corpse {
    timer: Timer,
    color: Color,
}

const CORPSE_SECS: f32 = 0.4;
/// Above the claimed shapes and the enemies, under the explosion.
const CORPSE_Z: f32 = 1.5;
const SHARD_SPEED: f32 = 160.;
/// Radians per second.
const SHARD_SPIN: f32 = 8.;
const ARROW_DRIFT: f32 = 140.;
const ARROW_SPIN: f32 = 14.;

/// A corpse that flies off while it fades ([082]): a square's pieces, an arrow.
#[derive(Component)]
struct Drift {
    velocity: Vec2,
    spin: f32,
}

/// A kill (an eaten or bombed enemy): it counts once and leaves a corpse and a pixel
/// explosion. An enemy that leaves with its state (level clear, death) is no kill ([030]).
fn on_despawn_enemy(
    trigger: On<DespawnEnemy>,
    query: Query<
        (
            &Transform,
            Option<&Mesh2d>,
            Option<&MeshMaterial2d<ColorMaterial>>,
            Option<&Shape>,
            Option<&Splitter>,
            Has<Square>,
            Has<Arrow>,
        ),
        With<Enemy>,
    >,
    colors: Res<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<PixelExplosionMaterial>>,
    mut commands: Commands,
    mut points: MessageWriter<IncreasePoints>,
    mut run: ResMut<Run>,
    mut rng: ResMut<GameRng>,
    time: Res<Time>,
) {
    let enemy_entity = trigger.event().0;
    info!("Bye bye {}", enemy_entity);

    if let Ok((transform, mesh, material, shape, splitter, square, arrow)) = query.get(enemy_entity)
    {
        if splitter == Some(&Splitter::Queen) {
            commands.spawn(splitter::hatch_soldiers());
        }
        let pos = transform.translation.truncate();
        let color = material
            .and_then(|m| colors.get(&m.0))
            .map(|m| m.color)
            .or(shape.and_then(|s| s.fill))
            .unwrap_or(NEON_WHITE)
            // A square fades in and out: its kill shows in full ([082]).
            .with_alpha(1.);
        commands.trigger(Burst::kill(pos, color));
        commands.trigger(Sfx::Eat);
        let corpse = || {
            (
                Corpse {
                    timer: Timer::from_seconds(CORPSE_SECS, TimerMode::Once),
                    color,
                },
                DespawnOnExit(Screen::Gameplay),
            )
        };
        let at = transform.with_translation(pos.extend(CORPSE_Z));
        if let (true, Some(material)) = (square, material) {
            // A square shatters: four pieces fly apart and spin ([082]).
            let piece = meshes.add(Rectangle::from_length(SQUARE_SIZE / 2.));
            for corner in [
                Vec2::ONE,
                Vec2::new(-1., 1.),
                Vec2::NEG_ONE,
                Vec2::new(1., -1.),
            ] {
                let offset = (at.rotation * (corner * SQUARE_SIZE / 4.).extend(0.)).truncate();
                commands.spawn((
                    corpse(),
                    Drift {
                        velocity: offset.normalize() * SHARD_SPEED,
                        spin: corner.x * corner.y * SHARD_SPIN,
                    },
                    Mesh2d(piece.clone()),
                    material.clone(),
                    at.with_translation(at.translation + offset.extend(0.)),
                ));
            }
        } else {
            let mut corpse = commands.spawn((corpse(), at));
            if let (Some(mesh), Some(material)) = (mesh, material) {
                corpse.insert((mesh.clone(), material.clone()));
            } else if let Some(shape) = shape {
                corpse.insert(shape.clone());
            }
            if arrow {
                // An arrow spins away along its heading ([082]).
                corpse.insert(Drift {
                    velocity: (at.rotation * Vec3::Y).truncate() * ARROW_DRIFT,
                    spin: ARROW_SPIN,
                });
            }
        }
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE * 2., SQUARE_SIZE * 2.))),
            MeshMaterial2d(explosion_materials.add(PixelExplosionMaterial {
                time: 0.0,
                start_time: time.elapsed_secs(),
                seed: rng.f32(),
            })),
            Transform::from_xyz(pos.x, pos.y, 2.),
            DespawnAfter::from_seconds(2.),
        ));
        commands.entity(enemy_entity).despawn();
        run.kill_count += 1;
        points.write(IncreasePoints(super::level::ENEMY_POINTS));
    }
}

fn animate_corpses(
    mut commands: Commands,
    time: Res<Time>,
    mut corpses: Query<(
        Entity,
        &mut Corpse,
        &mut Transform,
        Option<&MeshMaterial2d<ColorMaterial>>,
        Option<&mut Shape>,
        Option<&Drift>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut corpse, mut transform, material, shape, drift) in &mut corpses {
        corpse.timer.tick(time.delta());
        if let Some(drift) = drift {
            transform.translation += (drift.velocity * time.delta_secs()).extend(0.);
            transform.rotate_z(drift.spin * time.delta_secs());
        }
        let (scale, flash) = corpse_look(corpse.timer.fraction());
        transform.scale = Vec3::splat(scale);
        let color = if flash { NEON_WHITE } else { corpse.color };
        if let Some(mut material) = material.and_then(|m| materials.get_mut(&m.0)) {
            material.color = color;
        }
        if let Some(mut shape) = shape {
            shape.fill = Some(color);
        }
        if corpse.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// At `t` (0 to 1) of the corpse's time: its scale, and whether it flashes white. It pops
/// to 1.3 times its size in white over the first quarter, then shrinks to nothing.
fn corpse_look(t: f32) -> (f32, bool) {
    if t < 0.25 {
        (1. + 0.3 * t / 0.25, true)
    } else {
        (1.3 * (1. - (t - 0.25) / 0.75), false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct Sounds(Vec<Sfx>);

    #[test]
    fn an_eaten_enemy_counts_and_pays_once_and_a_state_exit_counts_none() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Time>();
        world.insert_resource(Run::with_seed(1));
        world.insert_resource(GameRng::new(1));
        world.add_observer(on_despawn_enemy);
        world.init_resource::<Sounds>();
        world.add_observer(|sfx: On<Sfx>, mut sounds: ResMut<Sounds>| sounds.0.push(*sfx));
        let enemies: Vec<Entity> = (0..4)
            .map(|_| world.spawn((Enemy, Transform::default())).id())
            .collect();
        let explosions = |world: &mut World| {
            world
                .query_filtered::<(), With<DespawnAfter>>()
                .iter(world)
                .count()
        };

        let corpses = |world: &mut World| world.query::<&Corpse>().iter(world).count();
        world.trigger(DespawnEnemy(enemies[0]));
        world.flush();
        world.trigger(DespawnEnemy(enemies[0]));
        world.flush();
        assert_eq!(world.resource::<Run>().kill_count, 1);
        assert_eq!(
            world.resource::<Messages<IncreasePoints>>().len(),
            1,
            "a second trigger pays nothing ([083])"
        );
        assert_eq!(explosions(&mut world), 1);
        assert_eq!(corpses(&mut world), 1);
        assert_eq!(
            world.resource::<Sounds>().0,
            [Sfx::Eat],
            "one sound ([042])"
        );
        assert!(world.get_entity(enemies[0]).is_err());

        // The run ends with three enemies alive: `DespawnOnExit` removes them.
        for enemy in &enemies[1..] {
            world.despawn(*enemy);
        }
        world.flush();
        assert_eq!(world.resource::<Run>().kill_count, 1);
        assert_eq!(explosions(&mut world), 1);
        assert_eq!(corpses(&mut world), 1);
    }

    #[test]
    fn a_square_shatters_into_four_pieces_and_an_arrow_spins_away() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Time>();
        world.insert_resource(Run::with_seed(1));
        world.insert_resource(GameRng::new(1));
        world.add_observer(on_despawn_enemy);
        let mesh = Mesh2d(
            world
                .resource_mut::<Assets<Mesh>>()
                .add(Rectangle::default()),
        );
        let material = MeshMaterial2d(
            world
                .resource_mut::<Assets<ColorMaterial>>()
                .add(ColorMaterial::default()),
        );
        let square = world
            .spawn((Square, Enemy, Transform::default(), mesh, material))
            .id();
        let arrow = world
            .spawn((
                Arrow,
                Enemy,
                Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            ))
            .id();
        let drifts = |world: &mut World| {
            world
                .query_filtered::<&Drift, With<Corpse>>()
                .iter(world)
                .map(|d| d.velocity)
                .collect::<Vec<_>>()
        };

        world.trigger(DespawnEnemy(square));
        world.flush();
        let pieces = drifts(&mut world);
        assert_eq!(pieces.len(), 4);
        assert!(
            pieces.iter().sum::<Vec2>().length() < 1e-3,
            "apart: {pieces:?}"
        );

        world.trigger(DespawnEnemy(arrow));
        world.flush();
        let heading_left = Vec2::NEG_X * ARROW_DRIFT;
        assert_eq!(drifts(&mut world).len(), 5);
        assert!(
            drifts(&mut world)
                .iter()
                .any(|v| v.abs_diff_eq(heading_left, 1e-3))
        );
    }

    #[test]
    fn only_a_dead_queen_hatches_soldiers() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Time>();
        world.insert_resource(Run::with_seed(1));
        world.insert_resource(GameRng::new(1));
        world.add_observer(on_despawn_enemy);
        let hatches = |world: &mut World| world.query::<&splitter::Hatch>().iter(world).count();
        for (kind, expected) in [(Splitter::Soldier, 0), (Splitter::Queen, 1)] {
            let enemy = world.spawn((kind, Enemy, Transform::default())).id();
            world.trigger(DespawnEnemy(enemy));
            world.flush();
            assert_eq!(hatches(&mut world), expected, "{kind:?}");
        }
    }

    #[test]
    fn a_corpse_pops_white_then_shrinks_to_nothing() {
        assert_eq!(corpse_look(0.), (1., true));
        assert!(corpse_look(0.2).1);
        let (scale, flash) = corpse_look(0.25);
        assert!((scale - 1.3).abs() < 1e-6 && !flash);
        assert!(corpse_look(0.6).0 < 1.3);
        assert_eq!(corpse_look(1.), (0., false));
    }

    /// Rests `body` on a wall for 2 s, then spawns a line collider (no rigid body, as
    /// `handle_draw_line` does) through it. Returns whether the body slept and how many
    /// `CollisionStart`s it had with the line.
    fn rest_then_draw_a_line(body: impl Bundle) -> (bool, usize) {
        use crate::game::get_player_layer;
        use std::time::Duration;

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            TransformPlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f64(1. / 60.),
        ));
        app.register_required_components::<Collider, CollisionEventsEnabled>();
        app.finish();
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::rectangle(400., 35.),
            Transform::from_xyz(0., -37.5, 0.),
        ));
        let body = app.world_mut().spawn(body).id();
        for _ in 0..120 {
            app.update();
        }
        let slept = app.world().get::<Sleeping>(body).is_some();
        let line = app
            .world_mut()
            .spawn((
                Collider::rectangle(100., 4.),
                Transform::default(),
                get_player_layer(),
            ))
            .id();
        let mut starts = 0;
        for _ in 0..5 {
            app.update();
            starts += app
                .world()
                .resource::<Messages<CollisionStart>>()
                .iter_current_update_messages()
                .filter(|c| {
                    [c.collider1, c.collider2] == [body, line]
                        || [c.collider2, c.collider1] == [body, line]
                })
                .count();
        }
        (slept, starts)
    }

    /// [063]: a body asleep on a wall still meets a new line (avian 0.8 queries colliders
    /// with no body against sleeping bodies).
    #[test]
    fn a_sleeping_square_and_bomb_still_meet_a_new_line() {
        use crate::game::{get_enemy_layer, get_item_layer};

        let square = (
            RigidBody::Dynamic,
            Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
            Transform::default(),
            GravityScale(2.5),
            Mass(15.),
            get_enemy_layer(),
        );
        assert_eq!(rest_then_draw_a_line(square), (true, 1));
        // As `spawn_basic_bomb`: no gravity, at rest from the start.
        let bomb = (
            RigidBody::Dynamic,
            Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
            Transform::from_xyz(0., 10., 0.),
            GravityScale(0.),
            LinearDamping(0.33),
            get_item_layer(),
        );
        assert_eq!(rest_then_draw_a_line(bomb), (true, 1));
    }
}
