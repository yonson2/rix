use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use crate::{
    colors::YELLOW,
    rix::{
        RixComponent, enemies::Enemy, events::InputPosition, get_enemy_layer, setup::LevelInfo,
        state::IsPaused, wall::WALL_THICKNESS,
    },
};

const ARROW_SIZE: f32 = 40.;

pub struct ArrowPlugin;

impl Plugin for ArrowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            point_arrow_to_cursor.run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
pub struct Arrow;

pub fn spawn_arrow(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut level_info: ResMut<LevelInfo>,
) {
    let position = generate_valid_position(*window, &mut level_info);

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

    let shape = ShapeBuilder::with(&rounded_polygon).fill(YELLOW).build();

    commands.spawn((
        Arrow,
        Enemy,
        RixComponent,
        shape,
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(0.),
        Mass(3.0),
        Collider::polyline(points.to_vec(), None),
        LinearVelocity(Vec2::ZERO),
        LinearDamping(0.222),
        Restitution::new(0.7).with_combine_rule(CoefficientCombine::Average),
        ArrowRotation::default(),
        get_enemy_layer(),
    ));
}

fn generate_valid_position(window: &Window, level_info: &mut LevelInfo) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let half_size = ARROW_SIZE;
    let margin = WALL_THICKNESS * 10.;

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let min_x = -half_width + margin + half_size;
    let max_x = half_width - margin - half_size;
    let min_y = -half_height + margin + half_size;
    let max_y = half_height - margin - half_size;
    let x = min_x + level_info.rng.f32() * (max_x - min_x);
    let y = min_y + level_info.rng.f32() * (max_y - min_y);

    Vec2::new(x, y)
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
