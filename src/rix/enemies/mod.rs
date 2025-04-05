use arrow::{ArrowPlugin, spawn_arrow};
use avian2d::prelude::*;
use ball::{BallPlugin, spawn_ball};
use bevy::{ecs::system::SystemId, prelude::*};
use rng::calculate_enemy_count;
use square::{SQUARE_SIZE, SquarePlugin, spawn_square};

use super::{
    effects::ExplosionMaterial,
    events::{self, DespawnEnemy, IncreasePoints},
    level::DespawnAfter,
    polygon::TempLineSegment,
    setup::LevelInfo,
    state::{AppState, IsPaused},
};

pub mod arrow;
pub mod ball;
pub mod rng;
pub mod square;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BallPlugin);
        app.add_plugins(SquarePlugin);
        app.add_plugins(ArrowPlugin);
        app.add_systems(OnEnter(AppState::InGame), spawn_enemies);
        app.add_systems(
            Update,
            (handle_enemy_collision, handle_die, despawn_enemy).run_if(in_state(IsPaused::Running)),
        );

        app.add_observer(on_despawn_enemy);
        app.add_observer(on_remove_enemy);
    }
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

fn spawn_enemies(world: &mut World) {
    let systems = {
        let enemy_systems = world.resource::<EnemySystems>();
        [
            enemy_systems.spawn_ball,
            enemy_systems.spawn_square,
            enemy_systems.spawn_arrow,
        ]
    };

    let enemy_count = {
        let mut level_info = world.resource_mut::<LevelInfo>();
        let level = level_info.level;
        calculate_enemy_count(level, &mut level_info.rng)
    };

    for _ in 0..enemy_count {
        let system_id = {
            let mut level_info = world.resource_mut::<LevelInfo>();
            level_info.rng.choice(&systems).copied().unwrap()
        };

        if let Err(e) = world.run_system(system_id) {
            error!("Error running system: {e}");
        }
    }
}

fn handle_enemy_collision(
    mut collision_event_reader: EventReader<Collision>,
    ball_q: Query<Entity, With<Enemy>>,
    lines_q: Query<Entity, With<TempLineSegment>>,
    mut events: EventWriter<events::Die>,
) {
    let enemies: Vec<Entity> = ball_q.iter().collect();
    let line: Vec<Entity> = lines_q.iter().collect();

    for Collision(contacts) in collision_event_reader.read() {
        let enemies = enemies.contains(&contacts.entity1) || enemies.contains(&contacts.entity2);
        let line = line.contains(&contacts.entity1) || line.contains(&contacts.entity2);
        if enemies && line {
            events.send(events::Die);
        }
    }
}

fn handle_die(mut events: EventReader<events::Die>, mut next_state: ResMut<NextState<AppState>>) {
    for _ in events.read() {
        next_state.set(AppState::GameOver);
    }
}

fn on_despawn_enemy(
    trigger: Trigger<DespawnEnemy>,
    mut commands: Commands,
    mut points: EventWriter<IncreasePoints>,
    mut level_info: ResMut<LevelInfo>,
) {
    let enemy_entity = trigger.event().0;
    info!("Bye bye {}", enemy_entity);

    if let Some(mut ec) = commands.get_entity(enemy_entity) {
        ec.despawn();
        level_info.kill_count += 1;
    }

    points.send(IncreasePoints(50)); //TODO: same as below.
}

//TODO: maybe we don't need this and can just use the trigger
//seems bad to have both.
fn despawn_enemy(
    mut commands: Commands,
    mut events: EventReader<DespawnEnemy>,
    mut points: EventWriter<IncreasePoints>,
    mut level_info: ResMut<LevelInfo>,
) {
    for e in events.read() {
        info!("Bye bye {}", e.0);
        if let Some(mut e) = commands.get_entity(e.0) {
            e.despawn();
            level_info.kill_count += 1;
        }
        points.send(IncreasePoints(50)); //TODO: different enemies increase different points
    }
}

// These are things to be done when an enemy is removed. i.e. when the
// despawn event or trigger has already been processed and the entity
// is gone.
fn on_remove_enemy(
    trigger: Trigger<OnRemove, Enemy>,
    query: Query<&Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut explosion_materials: ResMut<Assets<ExplosionMaterial>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    let enemy_pos = query.get(trigger.entity()).unwrap();
    let pos = enemy_pos.translation.truncate();
    let bundle = (
        Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE * 2., SQUARE_SIZE * 2.))),
        MeshMaterial2d(explosion_materials.add(ExplosionMaterial {
            time: 0.0,
            start_time: time.elapsed_secs(),
        })),
        Transform::from_xyz(pos.x, pos.y, 2.),
        DespawnAfter::new(2.),
    );
    commands.spawn(bundle);
}
