use arrow::{Arrow, spawn_arrow};
use avian2d::prelude::*;
use ball::spawn_ball;
use bevy::prelude::*;
use rng::{EnemyKind, SPAWN_TABLE, calculate_enemy_count, pick_kind};
use splitter::Splitter;
use square::{SQUARE_SIZE, Square, spawn_square};

use super::{
    AppSystems, PausableSystems,
    effects::{PixelExplosionMaterial, pixel_explosion},
    events::{self, DespawnEnemy, IncreasePoints},
    fx::Burst,
    items::star::Star,
    polygon::TempLineSegment,
    setup::{GameRng, Run},
    skins::{self, KillSkin, SkinAssets},
    state::{IsPaused, Phase},
    tutorial::Kind,
};
use crate::{audio::Sfx, save::Save, screens::Screen, shape::Shape, theme::palette::NEON_WHITE};

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

/// An enemy that touches the line kills the player; while the Star runs, the line kills the
/// enemy instead ([096]).
fn handle_enemy_collision(
    mut commands: Commands,
    mut collision_event_reader: MessageReader<CollisionStart>,
    ball_q: Query<Entity, With<Enemy>>,
    lines_q: Query<Entity, With<TempLineSegment>>,
    star: Option<Res<Star>>,
    mut events: MessageWriter<events::Die>,
) {
    let mut touched = vec![];
    for CollisionStart {
        collider1: entity1,
        collider2: entity2,
        ..
    } in collision_event_reader.read()
    {
        let enemy = [*entity1, *entity2]
            .into_iter()
            .find(|e| ball_q.contains(*e));
        let line = lines_q.contains(*entity1) || lines_q.contains(*entity2);
        if let (Some(enemy), true) = (enemy, line)
            && !touched.contains(&enemy)
        {
            touched.push(enemy);
        }
    }
    for enemy in touched {
        if star.is_some() {
            commands.trigger(events::DespawnEnemy(enemy));
        } else {
            events.write(events::Die);
        }
    }
}

/// A touch on the line kills, unless the Star ([069]) runs (a Shooter's bullet still writes
/// `Die` then).
fn handle_die(
    mut events: MessageReader<events::Die>,
    star: Option<Res<Star>>,
    mut next: ResMut<NextState<IsPaused>>,
) {
    if events.read().count() > 0 && star.is_none() {
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
            Has<ball::Ball>,
            Has<shooter::Shooter>,
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
    save: Res<Save>,
    skin_assets: Res<SkinAssets>,
) {
    let enemy_entity = trigger.event().0;
    info!("Bye bye {}", enemy_entity);

    if let Ok((transform, mesh, material, shape, splitter, square, arrow, ball, shooter)) =
        query.get(enemy_entity)
    {
        let kinds = [
            (ball, EnemyKind::Ball),
            (square, EnemyKind::Square),
            (arrow, EnemyKind::Arrow),
            (splitter.is_some(), EnemyKind::Splitter),
            (shooter, EnemyKind::Shooter),
        ];
        if let Some((_, kind)) = kinds.into_iter().find(|(has, _)| *has) {
            *run.eaten.entry(Kind::Enemy(kind).id()).or_default() += 1;
        }
        if splitter == Some(&Splitter::Queen) {
            commands.spawn(splitter::hatch_soldiers());
        }
        let pos = transform.translation.truncate();
        // A shape's material is the shared white one: its fill is its color ([126]).
        let color = shape
            .and_then(|s| s.fill)
            .or(material.and_then(|m| colors.get(&m.0)).map(|m| m.color))
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
            // A shape's corpse gets its own mesh: its pop must not tint the shared material.
            if let Some(shape) = shape {
                corpse.insert(shape.clone());
            } else if let (Some(mesh), Some(material)) = (mesh, material) {
                corpse.insert((mesh.clone(), material.clone()));
            }
            if arrow {
                // An arrow spins away along its heading ([082]).
                corpse.insert(Drift {
                    velocity: (at.rotation * Vec3::Y).truncate() * ARROW_DRIFT,
                    spin: ARROW_SPIN,
                });
            }
        }
        // Drawn with every kill skin, so a skin keeps the run ([104], [116]).
        let seed = rng.f32();
        if save.skins.kill == KillSkin::Pixel {
            commands.spawn(pixel_explosion(
                &mut meshes,
                &mut explosion_materials,
                time.elapsed_secs(),
                pos,
                seed,
            ));
        } else {
            skins::spawn_kill(&mut commands, save.skins.kill, &skin_assets, pos, color);
        }
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
        // A shape's material is the shared one ([126]): only its fill changes.
        if let Some(mut shape) = shape {
            shape.fill = Some(color);
        } else if let Some(mut material) = material.and_then(|m| materials.get_mut(&m.0)) {
            material.color = color;
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
    use crate::game::level::DespawnAfter;

    #[derive(Resource, Default)]
    struct Sounds(Vec<Sfx>);

    fn add_skins(world: &mut World) {
        world.init_resource::<Assets<Image>>();
        world.init_resource::<SkinAssets>();
        world.init_resource::<Save>();
    }

    #[test]
    fn every_kill_skin_draws_the_same_from_game_rng() {
        let next_draw = |kill: KillSkin| {
            let mut world = World::new();
            world.init_resource::<Assets<Mesh>>();
            world.init_resource::<Assets<ColorMaterial>>();
            world.init_resource::<Assets<PixelExplosionMaterial>>();
            add_skins(&mut world);
            world.resource_mut::<Save>().skins.kill = kill;
            world.init_resource::<Messages<IncreasePoints>>();
            world.init_resource::<Time>();
            world.insert_resource(Run::with_seed(1));
            world.insert_resource(GameRng::new(1));
            world.add_observer(on_despawn_enemy);
            let enemy = world.spawn((Enemy, Transform::default())).id();
            world.trigger(DespawnEnemy(enemy));
            world.flush();
            world.resource_mut::<GameRng>().u64(..)
        };
        let pixel = next_draw(KillSkin::Pixel);
        assert_eq!(next_draw(KillSkin::Shockwave), pixel);
        assert_eq!(next_draw(KillSkin::Firework), pixel);
    }

    #[test]
    fn an_eaten_enemy_counts_and_pays_once_and_a_state_exit_counts_none() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        add_skins(&mut world);
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
        add_skins(&mut world);
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
        add_skins(&mut world);
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
    fn a_kill_counts_for_its_kind() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        add_skins(&mut world);
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Time>();
        world.insert_resource(Run::with_seed(1));
        world.insert_resource(GameRng::new(1));
        world.add_observer(on_despawn_enemy);
        for kind in [
            world.spawn((Enemy, ball::Ball, Transform::default())).id(),
            world.spawn((Enemy, ball::Ball, Transform::default())).id(),
            world.spawn((Enemy, Arrow, Transform::default())).id(),
            world.spawn((Enemy, Transform::default())).id(),
        ] {
            world.trigger(DespawnEnemy(kind));
            world.flush();
        }
        let run = world.resource::<Run>();
        assert_eq!(run.kill_count, 4);
        assert_eq!(
            run.eaten,
            std::collections::BTreeMap::from([("Ball".into(), 2), ("Arrow".into(), 1)]),
            "a kill with no kind counts only in the total"
        );
    }

    #[test]
    fn a_shape_enemy_dies_in_its_fill_and_leaves_the_shared_material_white() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        add_skins(&mut world);
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Time>();
        world.insert_resource(Run::with_seed(1));
        world.insert_resource(GameRng::new(1));
        world.add_observer(on_despawn_enemy);
        let shared = world
            .resource_mut::<Assets<ColorMaterial>>()
            .add(ColorMaterial::default());
        let fill = Color::srgb(1., 0.8, 0.3);
        let enemy = world
            .spawn((
                Enemy,
                Transform::default(),
                Shape::new(crate::shape::polygon(&[Vec2::ZERO, Vec2::X, Vec2::Y])).fill(fill),
                Mesh2d::default(),
                MeshMaterial2d(shared.clone()),
            ))
            .id();
        world.trigger(DespawnEnemy(enemy));
        world.flush();
        let (corpse, color, has_shape) = world
            .query::<(Entity, &Corpse, Has<Shape>)>()
            .single(&world)
            .map(|(entity, corpse, has_shape)| (entity, corpse.color, has_shape))
            .unwrap();
        assert_eq!(color, fill);
        assert!(has_shape, "its own shape");
        // As `mesh_shapes` gives it, then its pop and its fade.
        world
            .entity_mut(corpse)
            .insert(MeshMaterial2d(shared.clone()));
        for _ in 0..2 {
            world.run_system_once(animate_corpses).unwrap();
            let material = world.resource::<Assets<ColorMaterial>>().get(&shared);
            assert_eq!(material.unwrap().color, Color::WHITE);
            world
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(200));
        }
        let shape = world.get::<Shape>(corpse).unwrap();
        assert_eq!(shape.fill, Some(fill), "its fill after the pop");
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

    /// [069]: the line has no body, so a ball passes through it; the touch still counts,
    /// and only the Star keeps the player alive.
    #[test]
    fn a_ball_passes_through_the_line_and_touches_it() {
        let (touches, kills, y) = ball_meets_line(false);
        assert_eq!((touches, kills), (1, 0));
        assert!(y.unwrap() < -10., "it crossed the line: {y:?}");
    }

    #[test]
    fn in_a_star_the_line_kills_the_ball_that_touches_it() {
        assert_eq!(ball_meets_line(true), (0, 1, None));
    }

    #[derive(Resource, Default)]
    struct Kills(usize);

    /// A ball falls through a line segment: the `Die` messages, the `DespawnEnemy` events,
    /// and where the ball ends (`None` once it is gone).
    fn ball_meets_line(star: bool) -> (usize, usize, Option<f32>) {
        use crate::game::{get_enemy_layer, get_player_layer};
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
        ))
        .add_message::<events::Die>()
        .init_resource::<Kills>()
        .add_observer(
            |kill: On<events::DespawnEnemy>, mut commands: Commands, mut kills: ResMut<Kills>| {
                kills.0 += 1;
                commands.entity(kill.0).despawn();
            },
        )
        .add_systems(FixedUpdate, handle_enemy_collision);
        if star {
            app.insert_resource(Star::default());
        }
        app.register_required_components::<Collider, CollisionEventsEnabled>();
        app.finish();
        app.world_mut().spawn((
            TempLineSegment,
            Collider::rectangle(200., 4.),
            Transform::default(),
            get_player_layer(),
        ));
        let ball = app
            .world_mut()
            .spawn((
                Enemy,
                RigidBody::Dynamic,
                GravityScale(0.),
                Collider::circle(10.),
                Transform::from_xyz(0., 60., 0.),
                LinearVelocity(Vec2::new(0., -200.)),
                Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
                get_enemy_layer(),
            ))
            .id();
        let mut touches = 0;
        for _ in 0..40 {
            app.update();
            touches += app
                .world()
                .resource::<Messages<events::Die>>()
                .iter_current_update_messages()
                .count();
        }
        let y = app
            .world()
            .get::<Transform>(ball)
            .map(|transform| transform.translation.y);
        (touches, app.world().resource::<Kills>().0, y)
    }

    #[test]
    fn a_touch_kills_only_without_the_star() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Messages<events::Die>>();
        world.init_resource::<NextState<IsPaused>>();
        world.insert_resource(Star::default());
        world.write_message(events::Die);
        world.run_system_once(handle_die).unwrap();
        assert!(matches!(
            world.resource::<NextState<IsPaused>>(),
            NextState::Unchanged
        ));

        world.remove_resource::<Star>();
        world.write_message(events::Die);
        world.run_system_once(handle_die).unwrap();
        assert!(matches!(
            world.resource::<NextState<IsPaused>>(),
            NextState::Pending(IsPaused::Dying)
        ));
    }
}
