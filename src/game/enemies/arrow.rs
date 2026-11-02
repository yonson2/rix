use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    game::{
        AppSystems, PausableSystems, enemies::Enemy, events::InputPosition, get_enemy_layer,
        setup::GameRng, state::Phase, user_input::UserPath,
    },
    shape::{self, Shape},
    theme::palette::{NEON_YELLOW, YELLOW},
    utils::generate_enemy_position,
};

const ARROW_SIZE: f32 = 20.;
const DRAWING_STROKE: f32 = 4.;
const NORMAL_STROKE: f32 = 2.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (point_arrow_to_cursor, handle_collision)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        light_arrow
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct Arrow;

pub fn spawn_arrow(mut commands: Commands, spatial_query: SpatialQuery, mut rng: ResMut<GameRng>) {
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

    let shape = Shape::new(shape::rounded_polygon(&points, 5.))
        .fill(YELLOW)
        .stroke(NEON_YELLOW, 4.);

    let collider = Collider::polyline(points.to_vec(), None);

    let position = generate_enemy_position(&spatial_query, &mut rng, &collider);

    commands.spawn((
        Arrow,
        Enemy,
        DespawnOnExit(Phase::Playing),
        shape,
        collider,
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(0.),
        // The polyline has no area, so no inertia of its own ([065]). Contacts never turn
        // the arrow (`point_arrow_to_cursor` does), so the inertia is never used.
        (Mass(3.0), AngularInertia(1.), LockedAxes::ROTATION_LOCKED),
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
    mut input_events: MessageReader<InputPosition>,
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
            let force = normalized_dir * 750.0 * time.delta_secs();

            let distance = direction.length();
            let speed_factor = (distance / 200.0).min(1.0);

            velocity.0 += force * speed_factor;
        }
    }
}

fn handle_collision(
    mut collision_events: MessageReader<CollisionStart>,
    arrow_query: Query<Entity, With<Arrow>>,
    mut velocities: Query<&mut LinearVelocity>,
    transforms: Query<&Transform>,
) {
    for CollisionStart {
        collider1: entity1,
        collider2: entity2,
        ..
    } in collision_events.read()
    {
        let entity1 = *entity1;
        let entity2 = *entity2;
        let both_arrows = arrow_query.contains(entity1) && arrow_query.contains(entity2);

        if both_arrows
            && let (Ok(transform1), Ok(transform2)) =
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

fn light_arrow(user_path: Res<UserPath>, mut query: Query<&mut Shape, With<Arrow>>) {
    if user_path.is_changed() {
        if user_path.is_drawing {
            for mut shape in query.iter_mut() {
                if let Some(stroke) = &mut shape.stroke {
                    stroke.width = DRAWING_STROKE;
                }
            }
        } else {
            for mut shape in query.iter_mut() {
                if let Some(stroke) = &mut shape.stroke {
                    stroke.width = NORMAL_STROKE;
                }
            }
        }
    }
}
