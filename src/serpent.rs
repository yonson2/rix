use avian2d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::{color::palettes::css::PURPLE, prelude::*, window::PrimaryWindow};

use crate::camera::get_cursor_world_position;

const SERPENT_SEGMENT_RADIUS: f32 = 30.;
const SERPENT_DEFAULT_LENGTH: usize = 10;
const SERPENT_SPEED: f32 = 3000.;

pub struct SerpentPlugin;

impl Plugin for SerpentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_serpent);
        app.add_systems(Update, move_serpents);
    }
}

#[derive(Component)]
pub struct Serpent;

#[derive(Component)]
pub struct SerpentHead;

fn spawn_serpent(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let snake_length = SERPENT_DEFAULT_LENGTH;
    let mut previous_segment_id = None;

    // Spawn all segments
    for i in 0..snake_length {
        let y_position = -1.0 * (i as f32) * SERPENT_SEGMENT_RADIUS;

        let segment_components = (
            Serpent,
            Mesh2d(meshes.add(Circle::new(SERPENT_SEGMENT_RADIUS))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::from_xyz(0.0, y_position, 0.0),
            RigidBody::Dynamic,
            Collider::circle(SERPENT_SEGMENT_RADIUS),
            LinearVelocity(Vec2::splat(SERPENT_SPEED)),
            AngularDamping(0.),
            Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
            LinearDamping(0.),
        );

        // Add head component to the first segment
        let segment_id = if i == 0 {
            commands
                .spawn(segment_components)
                .insert(SerpentHead)
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(meshes.add(Circle::new(SERPENT_SEGMENT_RADIUS + 2.0))),
                        MeshMaterial2d(materials.add(Color::BLACK)),
                        Transform::from_xyz(0.0, 0.0, -0.1),
                    ));
                })
                .id()
        } else {
            commands
                .spawn(segment_components)
                .with_children(|parent| {
                    parent.spawn((
                        Mesh2d(meshes.add(Circle::new(SERPENT_SEGMENT_RADIUS + 2.0))),
                        MeshMaterial2d(materials.add(Color::BLACK)),
                        Transform::from_xyz(0.0, 0.0, -0.1),
                    ));
                })
                .id()
        };

        if let Some(prev_id) = previous_segment_id {
            commands.spawn(DistanceJoint {
                entity1: prev_id,
                entity2: segment_id,
                local_anchor1: Vec2::ZERO,
                local_anchor2: Vec2::ZERO,
                rest_length: SERPENT_SEGMENT_RADIUS,
                length_limits: None,
                damping_linear: 0.0,
                damping_angular: 0.0,
                lagrange: Scalar::default(),
                compliance: 0.00,
                force: Vector::ZERO,
            });
        }

        previous_segment_id = Some(segment_id);
    }
}

fn move_serpents(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_heads: Query<(&mut Transform, &Serpent, &mut SerpentHead)>,
    q_nodes: Query<(&Transform, &Serpent), Without<SerpentHead>>,
    time: Res<Time>,
) {
    let Some(target_position) = get_cursor_world_position(&q_windows, &q_camera) else {
        return;
    };

    for (mut transform, node, mut head) in &mut q_heads {
        let mut direction = target_position - transform.translation.truncate();
        if direction.length_squared() <= 100.0 {
            return;
        }

        transform.translation +=
            (direction.normalize_or_zero() * SERPENT_SPEED * time.delta_secs()).extend(0.0);
        transform.rotation = Quat::from_axis_angle(Vec3::Z, direction.to_angle());
    }
}
