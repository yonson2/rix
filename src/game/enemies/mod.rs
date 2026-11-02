use arrow::spawn_arrow;
use avian2d::prelude::*;
use ball::spawn_ball;
use bevy::{ecs::system::SystemId, prelude::*};
use rng::calculate_enemy_count;
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
        Update,
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

#[derive(Resource, Clone, Copy)]
pub struct EnemySystems {
    pub spawn_ball: SystemId,
    pub spawn_square: SystemId,
    pub spawn_arrow: SystemId,
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct TeleportTimer(pub Timer);

pub fn register_enemy_systems(world: &mut World) {
    let spawn_ball = world.register_system_cached(spawn_ball);
    let spawn_square = world.register_system_cached(spawn_square);
    let spawn_arrow = world.register_system_cached(spawn_arrow);
    let systems = EnemySystems {
        spawn_ball,
        spawn_square,
        spawn_arrow,
    };
    world.insert_resource(systems);
}

pub(super) fn spawn_enemies(world: &mut World) {
    let systems = {
        let enemy_systems = world.resource::<EnemySystems>();
        [
            enemy_systems.spawn_ball,
            enemy_systems.spawn_square,
            enemy_systems.spawn_arrow,
        ]
    };

    let enemy_count = {
        let level = world.resource::<Run>().level;
        calculate_enemy_count(level, &mut world.resource_mut::<GameRng>())
    };

    for _ in 0..enemy_count {
        let system_id = {
            let mut rng = world.resource_mut::<GameRng>();
            rng.choice(&systems).copied().unwrap()
        };

        if let Err(e) = world.run_system(system_id) {
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
#[allow(clippy::too_many_arguments)]
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
}
