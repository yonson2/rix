use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use crate::{
    colors::{NEON_YELLOW, YELLOW},
    rix::{
        RixComponent, enemies::Enemy, events::InputPosition, get_enemy_layer, setup::LevelInfo,
        state::IsPaused, user_input::UserPath,
    },
    utils::generate_enemy_position,
};

const ARROW_SIZE: f32 = 20.;
const DRAWING_STROKE: f32 = 4.;
const NORMAL_STROKE: f32 = 2.;

pub struct ArrowPlugin;

impl Plugin for ArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (point_arrow_to_cursor, handle_collision, light_arrow)
                .run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
pub struct Arrow;

pub fn spawn_arrow(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    spatial_query: SpatialQuery,
    mut level_info: ResMut<LevelInfo>,
) {
    let points = [
        Vec2::new(0.3, -1.0),
        Vec2::new(0.3, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.5),
        Vec2::new(-1.0, 0.0),
        Vec2::new(-0.3, 0.0),
        Vec2::new(-0.3, -1.0),
    ]
    .map(|x| x * ARROW_SIZE);

    let rounded_polygon = shapes::RoundedPolygon {
        points: points.to_vec(),
        radius: 5.,
        closed: true,
    };

    let stroke = Stroke::new(NEON_YELLOW, 4.);
    let shape = ShapeBuilder::with(&rounded_polygon)
        .fill(YELLOW)
        .stroke(stroke)
        .build();

    let collider = Collider::polyline(points.to_vec(), None);

    let position = generate_enemy_position(*window, &spatial_query, &mut level_info.rng, &collider);

    commands.spawn((
        Arrow,
        Enemy,
        RixComponent,
        shape,
        collider,
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(0.),
        Mass(3.0),
        LinearVelocity(Vec2::ZERO),
        LinearDamping(0.222),
        Friction::new(0.2),
        Restitution::new(0.95).with_combine_rule(CoefficientCombine::Average),
        ArrowRotation::default(),
        get_enemy_layer(),
    ));
}

#[derive(Component, Default)]
struct ArrowRotation {
    target_angle: f32,
    rotation_speed: f32,
}

fn point_arrow_to_cursor(
    mut arrow_query: Query<(&mut Transform, &mut LinearVelocity, &mut ArrowRotation), With<Arrow>>,
    mut input_events: EventReader<InputPosition>,
    time: Res<Time>,
) {
    let Some(cursor_position) = input_events.read().last() else {
        return;
    };

    for (mut transform, mut velocity, mut arrow_rotation) in arrow_query.iter_mut() {
        let arrow_pos = transform.translation.truncate();
        let direction = cursor_position.0 - arrow_pos;

        if direction.length_squared() > 0.001 {
            let target_angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
            arrow_rotation.target_angle = target_angle;

            let current_angle = transform.rotation.to_euler(EulerRot::ZYX).0;

            let mut angle_diff = arrow_rotation.target_angle - current_angle;
            while angle_diff > std::f32::consts::PI {
                angle_diff -= std::f32::consts::TAU;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += std::f32::consts::TAU;
            }

            let max_rotation_speed = 15.0;
            let target_rotation_speed = angle_diff.abs() * 3.0;

            arrow_rotation.rotation_speed = arrow_rotation.rotation_speed * 0.9
                + target_rotation_speed.min(max_rotation_speed) * 0.1;

            let rotation_amount =
                arrow_rotation.rotation_speed * angle_diff.signum() * time.delta_secs();
            let new_angle = current_angle + rotation_amount;

            transform.rotation = Quat::from_rotation_z(new_angle);

            let normalized_dir = direction.normalize();
            let force = normalized_dir * 500.0 * time.delta_secs();

            let distance = direction.length();
            let speed_factor = (distance / 200.0).min(1.0);

            velocity.0 += force * speed_factor;
        }
    }
}

fn handle_collision(
    mut collision_events: EventReader<Collision>,
    arrow_query: Query<Entity, With<Arrow>>,
    mut velocities: Query<&mut LinearVelocity>,
    transforms: Query<&Transform>,
) {
    for Collision(contacts) in collision_events.read() {
        let entity1 = contacts.entity1;
        let entity2 = contacts.entity2;
        let both_arrows = arrow_query.contains(entity1) && arrow_query.contains(entity2);

        if both_arrows {
            if let (Ok(transform1), Ok(transform2)) =
                (transforms.get(entity1), transforms.get(entity2))
            {
                let pos1 = transform1.translation.truncate();
                let pos2 = transform2.translation.truncate();

                let direction = (pos1 - pos2).normalize_or_zero();

                if let Ok(mut velocity1) = velocities.get_mut(entity1) {
                    velocity1.0 += direction * 80.0;
                }

                if let Ok(mut velocity2) = velocities.get_mut(entity2) {
                    velocity2.0 -= direction * 80.0;
                }
            }
        }
    }
}

fn light_arrow(user_path: Res<UserPath>, mut query: Query<&mut Shape, With<Arrow>>) {
    if user_path.is_changed() {
        if user_path.is_drawing {
            for mut shape in query.iter_mut() {
                shape.stroke = shape.stroke.map(|mut s| {
                    s.options.line_width = DRAWING_STROKE;
                    s
                });
            }
        } else {
            for mut shape in query.iter_mut() {
                shape.stroke = shape.stroke.map(|mut s| {
                    s.options.line_width = NORMAL_STROKE;
                    s
                });
            }
        }
    }
}
