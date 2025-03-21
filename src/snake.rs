use bevy::{color::palettes::basic::PURPLE, prelude::*};
use rand::{Rng, thread_rng};

use crate::chain::{ChainHead, ChainNode, move_chain, rotate};

const SNAKE_SPEED: f32 = 500.;
const DIRECTION_CHANGE_INTERVAL: f32 = 0.5;
const WALL_DETECTION_DISTANCE: f32 = 200.0;
const SCREEN_EDGE_BUFFER: f32 = 50.0;

#[derive(Component)]
pub struct AutonomousMovement {
    pub timer: Timer,
    pub target_direction: Vec2,
    pub current_direction: Vec2,
    pub smoothing_factor: f32,
}

impl Default for AutonomousMovement {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(DIRECTION_CHANGE_INTERVAL, TimerMode::Repeating),
            target_direction: Vec2::new(0.0, 1.0),
            current_direction: Vec2::new(0.0, 1.0),
            smoothing_factor: 0.2,
        }
    }
}

pub struct SnakePlugin;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_snake)
            .add_systems(Update, move_snake.before(move_chain));
    }
}

fn move_snake(
    mut q_heads: Query<(
        &mut Transform,
        &ChainNode,
        &mut ChainHead,
        &mut AutonomousMovement,
    )>,
    q_nodes: Query<(&Transform, &ChainNode), Without<ChainHead>>,
    time: Res<Time>,
    q_windows: Query<&Window>,
) {
    let mut rng = thread_rng();

    let window = q_windows.single();
    let window_width = window.width();
    let window_height = window.height();
    let half_width = window_width / 2.0;
    let half_height = window_height / 2.0;

    for (mut transform, node, mut head, mut auto_move) in &mut q_heads {
        auto_move.timer.tick(time.delta());

        if auto_move.timer.just_finished() {
            let angle = rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI);
            auto_move.target_direction = Vec2::new(angle.cos(), angle.sin());
        }

        // Check for screen boundaries and adjust direction
        let current_pos = transform.translation.truncate();
        let mut boundary_avoidance = Vec2::ZERO;

        // Calculate distance to each screen edge
        let dist_to_right = half_width - current_pos.x;
        let dist_to_left = half_width + current_pos.x;
        let dist_to_top = half_height - current_pos.y;
        let dist_to_bottom = half_height + current_pos.y;

        // Apply repulsive forces when approaching screen edges
        if dist_to_right < WALL_DETECTION_DISTANCE {
            let repulsion_strength = (1.0 - (dist_to_right / WALL_DETECTION_DISTANCE)).powf(2.0);
            boundary_avoidance += Vec2::new(-1.0, 0.0) * repulsion_strength * 2.0;
        }

        if dist_to_left < WALL_DETECTION_DISTANCE {
            let repulsion_strength = (1.0 - (dist_to_left / WALL_DETECTION_DISTANCE)).powf(2.0);
            boundary_avoidance += Vec2::new(1.0, 0.0) * repulsion_strength * 2.0;
        }

        if dist_to_top < WALL_DETECTION_DISTANCE {
            let repulsion_strength = (1.0 - (dist_to_top / WALL_DETECTION_DISTANCE)).powf(2.0);
            boundary_avoidance += Vec2::new(0.0, -1.0) * repulsion_strength * 2.0;
        }

        if dist_to_bottom < WALL_DETECTION_DISTANCE {
            let repulsion_strength = (1.0 - (dist_to_bottom / WALL_DETECTION_DISTANCE)).powf(2.0);
            boundary_avoidance += Vec2::new(0.0, 1.0) * repulsion_strength * 2.0;
        }

        // If there's a significant avoidance vector, use it to influence the direction
        if boundary_avoidance.length_squared() > 0.01 {
            auto_move.target_direction =
                (auto_move.target_direction + boundary_avoidance * 2.0).normalize();
        }

        // Smooth the direction change
        auto_move.current_direction = auto_move
            .current_direction
            .lerp(auto_move.target_direction, auto_move.smoothing_factor)
            .normalize();

        // Apply chain constraints for smooth turning similar to move_snakes
        let mut direction = auto_move.current_direction;
        let next_entity = node.next;
        if let Some(next) = next_entity {
            if let Ok((next_transform, _)) = q_nodes.get(next) {
                let predicted_direction =
                    (transform.translation - next_transform.translation).truncate();
                let angle = predicted_direction
                    .angle_to(direction)
                    .to_degrees()
                    .clamp(-20., 20.);

                direction = rotate(predicted_direction, angle.to_radians());
            }
        }

        let next_pos = transform.translation
            + (direction.normalize_or_zero() * SNAKE_SPEED * time.delta_secs()).extend(0.0);

        // Check if the next position would be off-screen and clamp it if needed
        let clamped_x = next_pos.x.clamp(
            -half_width + SCREEN_EDGE_BUFFER,
            half_width - SCREEN_EDGE_BUFFER,
        );
        let clamped_y = next_pos.y.clamp(
            -half_height + SCREEN_EDGE_BUFFER,
            half_height - SCREEN_EDGE_BUFFER,
        );

        // Update head direction and position
        head.direction = direction;
        transform.translation = Vec3::new(clamped_x, clamped_y, next_pos.z);
        transform.rotation = Quat::from_axis_angle(Vec3::Z, direction.to_angle());
    }
}

fn spawn_snake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let snake_size = 20.;
    let length = 30;

    let mut prev_entity = None;

    for i in (0..length).rev() {
        let mut entity = commands.spawn((
            ChainNode {
                next: prev_entity,
                distance: snake_size,
                angle_threshold: (20.0f32).to_radians(),
            },
            Mesh2d(meshes.add(Circle::new(snake_size))),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
            Transform::from_xyz(0., -i as f32 * snake_size, 0.0 - i as f32),
        ));

        if i == 0 {
            entity
                .insert((ChainHead {
                    target: Vec2::new(0.0, 20.0),
                    direction: Vec2::new(0.0, 1.0),
                },))
                .insert(AutonomousMovement {
                    ..Default::default()
                });
        }

        entity.with_children(|parent| {
            parent.spawn((
                Mesh2d(meshes.add(Circle::new(snake_size + 4.))),
                MeshMaterial2d(materials.add(Color::BLACK)),
                Transform::from_xyz(0., 0., -0.1),
            ));
        });

        prev_entity = Some(entity.id());
    }
}
