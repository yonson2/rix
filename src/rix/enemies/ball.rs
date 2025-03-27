use avian2d::prelude::*;
use bevy::prelude::*;
use fastrand;
use std::f32::consts::PI;

use crate::rix::{RixComponent, state::IsPaused};

use super::Enemy;

pub struct BallPlugin;

#[derive(Component)]
pub struct Ball;

// Component to track color transition
#[derive(Component)]
struct ColorTransition {
    timer: Timer,
}

const BALL_COLOR: Srgba = Srgba::rgb(2.72549, 0.10980392, 0.10980392);
const BALL_COLLISION_COLOR: Srgba = Srgba::rgb(4.72549, 0.10980392, 0.10980392);
const TRANSITION_DURATION: f32 = 0.2;
const BALL_SPEED: f32 = 500.0;
const BALL_RADIUS: f32 = 25.0;
const CHAOS_FACTOR: f32 = 0.2; // Value between 0.0 and 1.0 to control randomness

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_chaos_on_collision.run_if(in_state(IsPaused::Running)),
                update_ball_color.run_if(in_state(IsPaused::Running)),
            ),
        );
    }
}

// System to add chaos on collision and maintain constant speed
fn add_chaos_on_collision(
    mut collision_events: EventReader<Collision>,
    mut ball_query: Query<(&mut LinearVelocity, Entity), With<Ball>>,
    mut commands: Commands,
) {
    for Collision(contacts) in collision_events.read() {
        let entities = [contacts.entity1, contacts.entity2];

        // Check if one of the entities is a ball
        for entity in entities {
            if let Ok((mut velocity, ball_entity)) = ball_query.get_mut(entity) {
                // Add random perturbation to velocity
                let current_angle = velocity.y.atan2(velocity.x);

                // Generate a random angle deviation
                let angle_deviation =
                    (fastrand::f32() * 2.0 * CHAOS_FACTOR - CHAOS_FACTOR) * PI / 4.0;
                let new_angle = current_angle + angle_deviation;

                // Apply the new direction while maintaining constant speed
                velocity.x = new_angle.cos() * BALL_SPEED;
                velocity.y = new_angle.sin() * BALL_SPEED;

                // Start color transition
                commands.entity(ball_entity).insert(ColorTransition {
                    timer: Timer::from_seconds(TRANSITION_DURATION, TimerMode::Once),
                });
            }
        }
    }
}

fn update_ball_color(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ColorTransition, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transition, mesh_material) in &mut query {
        transition.timer.tick(time.delta());

        let t = transition.timer.fraction();

        // First half: transition from start to end color
        // Second half: transition from end back to start color
        let factor = if t < 0.5 {
            t * 2.0 // Scale 0.0-0.5 to 0.0-1.0
        } else {
            (1.0 - t) * 2.0 // Scale 0.5-1.0 to 1.0-0.0
        };

        // Mix colors using Srgba's mix function
        let current_color = BALL_COLOR.mix(&BALL_COLLISION_COLOR, factor);

        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.color = Color::from(current_color);
        }

        if transition.timer.finished() {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}

pub fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let random_angle = fastrand::f32() * 2.0 * PI;

    let initial_velocity = Vec2::new(
        random_angle.cos() * BALL_SPEED,
        random_angle.sin() * BALL_SPEED,
    );

    commands.spawn((
        Ball,
        Enemy,
        RixComponent,
        Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
        MeshMaterial2d(materials.add(Color::from(BALL_COLOR))),
        Transform::from_xyz(0.0, 0.0, 1.0),
        RigidBody::Dynamic,
        Collider::circle(BALL_RADIUS),
        LinearVelocity(initial_velocity),
        AngularDamping(0.),
        Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
        LinearDamping(0.),
    ));
}
