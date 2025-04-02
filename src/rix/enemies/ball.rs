use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use std::f32::consts::PI;

use crate::{
    colors::{BURGUNDY, NEON_BURGUNDY},
    rix::{
        RixComponent,
        enemies::{Enemy, Transition},
        get_enemy_layer,
        setup::LevelInfo,
        state::IsPaused,
        wall::WALL_THICKNESS,
    },
    utils::{ColorTransition, update_mesh_color},
};

const BALL_COLOR: Color = BURGUNDY;
const BALL_COLLISION_COLOR: Color = NEON_BURGUNDY;
const COLOR_TRANSITION_DURATION: f32 = 0.2;
const TRANSITION_DURATION: f32 = 1.;
const BALL_SPEED: f32 = 500.0;
const BALL_INITIAL_RADIUS: f32 = 5.0;
const BALL_RADIUS: f32 = 25.0;
const CHAOS_FACTOR: f32 = 0.2; // Value between 0.0 and 1.0 to control randomness

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_chaos_on_collision,
                update_mesh_color::<Ball>,
                update_ball_size,
            )
                .run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
pub struct Ball;

pub fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut level_info: ResMut<LevelInfo>,
) {
    let random_angle = level_info.rng.f32() * 2.0 * PI;

    let initial_velocity = Vec2::new(
        random_angle.cos() * BALL_SPEED,
        random_angle.sin() * BALL_SPEED,
    );
    
    let position = generate_valid_position(*window, &mut level_info);

    commands.spawn((
        Ball,
        Enemy,
        RixComponent,
        Mesh2d(meshes.add(Circle::new(BALL_INITIAL_RADIUS))),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(0.),
        Mass(1.),
        Collider::circle(BALL_INITIAL_RADIUS),
        LinearVelocity(initial_velocity),
        Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
        Transition(Timer::from_seconds(TRANSITION_DURATION, TimerMode::Once)),
        get_enemy_layer(),
    ));
}

// System to add chaos on collision and maintain constant speed
fn add_chaos_on_collision(
    mut collision_events: EventReader<Collision>,
    mut ball_query: Query<(&mut LinearVelocity, Entity), With<Ball>>,
    mut commands: Commands,
    mut level_info: ResMut<LevelInfo>,
) {
    for Collision(contacts) in collision_events.read() {
        let entities = [contacts.entity1, contacts.entity2];

        for entity in entities {
            if let Ok((mut velocity, ball_entity)) = ball_query.get_mut(entity) {
                // Add random perturbation to velocity
                let current_angle = velocity.y.atan2(velocity.x);

                let angle_deviation =
                    (level_info.rng.f32() * 2.0 * CHAOS_FACTOR - CHAOS_FACTOR) * PI / 4.0;
                let new_angle = current_angle + angle_deviation;

                // Apply the new direction while maintaining constant speed
                velocity.x = new_angle.cos() * BALL_SPEED;
                velocity.y = new_angle.sin() * BALL_SPEED;

                if let Some(mut ec) = commands.get_entity(ball_entity) {
                    ec.try_insert(ColorTransition {
                        timer: Timer::from_seconds(COLOR_TRANSITION_DURATION, TimerMode::Once),
                        from: BALL_COLOR,
                        to: BALL_COLLISION_COLOR,
                    });
                }
            }
        }
    }
}

fn update_ball_size(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transition, &mut Mesh2d, &mut Collider), With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut transition, mesh, mut collider) in &mut query {
        transition.0.tick(time.delta());

        let t = transition.0.fraction();

        let easing = EasingCurve::new(BALL_INITIAL_RADIUS, BALL_RADIUS, EaseFunction::BounceOut);
        let current_radius = easing.sample(t).unwrap();

        // Update the mesh with the new radius
        if let Some(mesh_asset) = meshes.get_mut(&mesh.0) {
            *mesh_asset = Circle::new(current_radius).into();
        }

        // Update the collider to match the new radius
        *collider = Collider::circle(current_radius);

        if transition.0.finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

fn generate_valid_position(window: &Window, level_info: &mut LevelInfo) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let margin = WALL_THICKNESS;
    let radius = BALL_RADIUS; // Using the final radius for positioning

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let min_x = -half_width + margin + radius;
    let max_x = half_width - margin - radius;
    let min_y = -half_height + margin + radius;
    let max_y = half_height - margin - radius;
    
    let x = min_x + level_info.rng.f32() * (max_x - min_x);
    let y = min_y + level_info.rng.f32() * (max_y - min_y);

    Vec2::new(x, y)
}
