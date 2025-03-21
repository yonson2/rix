use avian2d::prelude::*;
use bevy::{color::palettes::basic::RED, prelude::*};
use rand::prelude::*;
use std::f32::consts::PI;

pub struct BallPlugin;

#[derive(Component)]
pub struct Ball;

const BALL_SPEED: f32 = 1000.0;
const BALL_RADIUS: f32 = 15.0;
const CHAOS_FACTOR: f32 = 0.4; // Value between 0.0 and 1.0 to control randomness

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ball)
            .add_systems(Update, add_chaos_on_collision);
    }
}

// System to add chaos on collision and maintain constant speed
fn add_chaos_on_collision(
    mut collision_events: EventReader<Collision>,
    mut ball_query: Query<&mut LinearVelocity, With<Ball>>,
) {
    for Collision(contacts) in collision_events.read() {
        let entities = [contacts.entity1, contacts.entity2];

        // Check if one of the entities is a ball
        for entity in entities {
            if let Ok(mut velocity) = ball_query.get_mut(entity) {
                let mut rng = thread_rng();
                // Add random perturbation to velocity
                let current_angle = velocity.y.atan2(velocity.x);

                // Generate a random angle deviation
                let angle_deviation = rng.gen_range(-CHAOS_FACTOR..CHAOS_FACTOR) * PI / 4.0;
                let new_angle = current_angle + angle_deviation;

                // Apply the new direction while maintaining constant speed
                velocity.x = new_angle.cos() * BALL_SPEED;
                velocity.y = new_angle.sin() * BALL_SPEED;
            }
        }
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Generate a random angle between 0 and 2π
    let mut rng = thread_rng();
    let random_angle = rng.gen_range(0.0..2.0 * PI);

    // Calculate initial velocity vector based on the random angle
    let initial_velocity = Vec2::new(
        random_angle.cos() * BALL_SPEED,
        random_angle.sin() * BALL_SPEED,
    );

    commands
        .spawn((
            Ball,
            Mesh2d(meshes.add(Circle::new(BALL_RADIUS))),
            MeshMaterial2d(materials.add(Color::from(RED))),
            Transform::from_xyz(0.0, 0.0, 1.0), // Center of the screen, above other elements
            RigidBody::Dynamic,
            Collider::circle(BALL_RADIUS),
            LinearVelocity(initial_velocity),
            AngularDamping(0.),
            Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
            LinearDamping(0.),
        ))
        .with_children(|parent| {
            // Add an outline
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(BALL_RADIUS + 2.0))),
                MeshMaterial2d(materials.add(Color::BLACK)),
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
        });
}
