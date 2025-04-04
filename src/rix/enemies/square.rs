use avian2d::prelude::*;
use bevy::{prelude::*, time::Time, window::PrimaryWindow};

use crate::{
    colors::{GREEN, NEON_GREEN},
    rix::{
        RixComponent,
        enemies::{Enemy, Transition},
        get_enemy_layer,
        setup::LevelInfo,
        state::IsPaused,
    },
    utils::{ColorTransition, generate_enemy_position, update_mesh_color},
};

pub const SQUARE_SIZE: f32 = 40.;
const SQUARE_COLOR: Color = GREEN;
const SQUARE_NEON_COLOR: Color = NEON_GREEN;
const TRANSITION_DURATION: f32 = 1.8;
const SPARK_TRANSITION_DURATION: f32 = 0.2;
const PULSE_MIN_SCALE: f32 = 1.;
const PULSE_MAX_SCALE: f32 = 2.;
const PULSE_SPEED: f32 = 1.5;

pub struct SquarePlugin;

impl Plugin for SquarePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                flicker_square,
                update_mesh_color::<Square>,
                handle_collision,
                pulse_square,
            )
                .run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
pub struct Square {
    pub spawn_position: Vec2,
    pub time_offset: f32,
}

#[derive(Component)]
struct PulseAnimation {
    timer: Timer,
    direction: PulseDirection,
}

#[derive(Clone, Copy)]
enum PulseDirection {
    Expanding,
    Contracting,
}

pub fn spawn_square(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
    spatial_query: SpatialQuery,
    mut level_info: ResMut<LevelInfo>,
) {
    let mut transparent_color = SQUARE_COLOR;
    transparent_color.set_alpha(0.0);

    let position = generate_valid_position(*window, &mut level_info, spatial_query);
    let time_offset = level_info.rng.f32() * std::f32::consts::TAU;
    let dir = if level_info.rng.bool() { 1. } else { -1. };

    commands.spawn((
        Square {
            spawn_position: position,
            time_offset,
        },
        Enemy,
        RixComponent,
        Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE))),
        MeshMaterial2d(materials.add(transparent_color)),
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(2.5 * dir),
        Mass(15.0),
        Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
        LinearVelocity(Vec2::ZERO),
        Restitution::new(0.8).with_combine_rule(CoefficientCombine::Average),
        Transition(Timer::from_seconds(TRANSITION_DURATION, TimerMode::Once)),
        get_enemy_layer(),
    ));
}

fn flicker_square(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transition, &MeshMaterial2d<ColorMaterial>), With<Square>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transition, mesh_material) in &mut query {
        transition.0.tick(time.delta());

        let t = transition.0.fraction();

        let easing = EasingCurve::new(0.0, 1.0, EaseFunction::BounceInOut);
        let alpha = easing.sample(t).unwrap();

        let mut current_color = SQUARE_COLOR;
        current_color.set_alpha(alpha);

        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.color = current_color;
        }

        if transition.0.finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

fn handle_collision(
    mut commands: Commands,
    mut collision_events: EventReader<Collision>,
    mut square_query: Query<(Entity, &mut GravityScale, &mut LinearVelocity), With<Square>>,
    transforms: Query<&Transform>,
    pulse_query: Query<(), With<PulseAnimation>>,
) {
    for Collision(contacts) in collision_events.read() {
        let entities = [contacts.entity1, contacts.entity2];
        for entity in entities {
            if let Ok((square_ent, mut gravity, mut velocity)) = square_query.get_mut(entity) {
                if let Some(mut ec) = commands.get_entity(square_ent) {
                    ec.try_insert(ColorTransition {
                        timer: Timer::from_seconds(SPARK_TRANSITION_DURATION, TimerMode::Once),
                        from: SQUARE_COLOR,
                        to: SQUARE_NEON_COLOR,
                    });

                    if pulse_query.get(square_ent).is_err() {
                        ec.try_insert(PulseAnimation {
                            timer: Timer::from_seconds(PULSE_SPEED, TimerMode::Once),
                            direction: PulseDirection::Expanding,
                        });
                    }

                    gravity.0 *= -1.;
                    if let Ok(t1) = transforms.get(square_ent) {
                        let pos1 = t1.translation.truncate();
                        let pos2 = Vec2::ZERO;

                        let direction = (pos2 - pos1).normalize_or_zero();
                        velocity.0 += direction * 70.0;
                    }
                }
            }
        }
    }
}

fn pulse_square(
    time: Res<Time>,
    mut commands: Commands,
    mut square_query: Query<(Entity, &mut Transform, &mut PulseAnimation)>,
) {
    for (entity, mut transform, mut pulse_anim) in square_query.iter_mut() {
        pulse_anim.timer.tick(time.delta());

        let progress = pulse_anim.timer.fraction();

        let scale_factor = match pulse_anim.direction {
            PulseDirection::Expanding => lerp(PULSE_MIN_SCALE, PULSE_MAX_SCALE, progress),
            PulseDirection::Contracting => lerp(PULSE_MAX_SCALE, PULSE_MIN_SCALE, progress),
        };

        transform.scale = Vec3::splat(scale_factor);

        if pulse_anim.timer.finished() {
            match pulse_anim.direction {
                PulseDirection::Expanding => {
                    pulse_anim.direction = PulseDirection::Contracting;
                    pulse_anim.timer.reset();
                }
                PulseDirection::Contracting => {
                    commands.entity(entity).remove::<PulseAnimation>();
                }
            }
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn generate_valid_position(
    window: &Window,
    level_info: &mut LevelInfo,
    spatial_query: SpatialQuery,
) -> Vec2 {
    let collider = Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE);

    generate_enemy_position(window, spatial_query, &mut level_info.rng, &collider)
}
