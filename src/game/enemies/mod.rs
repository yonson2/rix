use arrow::spawn_arrow;
use avian2d::prelude::*;
use ball::spawn_ball;
use bevy::prelude::*;
use rng::{EnemyKind, SPAWN_TABLE, calculate_enemy_count, pick_kind};
use square::{SQUARE_SIZE, spawn_square};

use super::{
    AppSystems, PausableSystems,
    effects::PixelExplosionMaterial,
    events::{self, DespawnEnemy, IncreasePoints},
    level::DespawnAfter,
    polygon::TempLineSegment,
    setup::{GameRng, Run},
    state::Phase,
};
use crate::screens::Screen;

pub mod arrow;
pub mod ball;
pub mod rng;
pub mod square;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((ball::plugin, square::plugin, arrow::plugin));
    app.add_systems(OnEnter(Phase::Playing), spawn_enemies);
    app.add_systems(
        FixedUpdate,
        (handle_enemy_collision, handle_die)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

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

fn handle_die(mut events: MessageReader<events::Die>, mut next_state: ResMut<NextState<Screen>>) {
    for _ in events.read() {
        next_state.set(Screen::GameOver);
    }
}

/// A kill (an eaten or bombed enemy): it counts once and leaves a pixel explosion. An
/// enemy that leaves with its state (level clear, death) is no kill ([030]).
fn on_despawn_enemy(
    trigger: On<DespawnEnemy>,
    query: Query<&Transform, With<Enemy>>,
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

    if let Ok(transform) = query.get(enemy_entity) {
        let pos = transform.translation.truncate();
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
    }

    points.write(IncreasePoints(super::level::ENEMY_POINTS));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_eaten_enemy_counts_once_and_a_state_exit_counts_none() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<PixelExplosionMaterial>>();
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Time>();
        world.insert_resource(Run::with_seed(1));
        world.insert_resource(GameRng::new(1));
        world.add_observer(on_despawn_enemy);
        let enemies: Vec<Entity> = (0..4)
            .map(|_| world.spawn((Enemy, Transform::default())).id())
            .collect();
        let explosions = |world: &mut World| {
            world
                .query_filtered::<(), With<DespawnAfter>>()
                .iter(world)
                .count()
        };

        world.trigger(DespawnEnemy(enemies[0]));
        world.flush();
        assert_eq!(world.resource::<Run>().kill_count, 1);
        assert_eq!(explosions(&mut world), 1);
        assert!(world.get_entity(enemies[0]).is_err());

        // The run ends with three enemies alive: `DespawnOnExit` removes them.
        for enemy in &enemies[1..] {
            world.despawn(*enemy);
        }
        world.flush();
        assert_eq!(world.resource::<Run>().kill_count, 1);
        assert_eq!(explosions(&mut world), 1);
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
