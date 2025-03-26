use avian2d::prelude::*;
use ball::{BallPlugin, spawn_ball};
use bevy::{ecs::system::SystemId, prelude::*};

use super::{
    polygon::{TempLineSegment, events},
    state::{AppState, IsPaused},
};

pub mod ball;

pub struct EnemiesPlugin;
impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BallPlugin);
        app.add_systems(OnEnter(AppState::InGame), spawn_enemies);
        app.add_systems(
            Update,
            (handle_enemy_collision, handle_die).run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
/// An `Enemy` is a marker component to associate entities
/// as enemies.
pub struct Enemy;

#[derive(Resource)]
pub struct EnemySystems {
    pub spawn_ball: SystemId,
}

pub fn register_enemy_systems(world: &mut World) {
    let spawn_ball = world.register_system_cached(spawn_ball);
    let systems = EnemySystems { spawn_ball };
    world.insert_resource(systems);
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

/// Listens for the death event and resets the game.
fn handle_die(mut events: EventReader<events::Die>, mut next_state: ResMut<NextState<AppState>>) {
    // Not using a for-in here as multiple events kinda
    // don't make sense?
    if events.read().next().is_some() {
        next_state.set(AppState::Menu);
    }
}

fn spawn_enemies(world: &mut World) {
    let enemy_systems: &EnemySystems = world.resource();
    if let Err(e) = world.run_system(enemy_systems.spawn_ball) {
        error!("Error running system: {e}");
    }
}

/// Returns a vector of world coordinates for all points of the snake
pub fn get_points(nodes: &[&Transform]) -> Vec<Vec2> {
    nodes.iter().map(|&t| t.translation.truncate()).collect()
}
