use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use crate::{
    colors::YELLOW,
    rix::{RixComponent, enemies::Enemy, get_enemy_layer, setup::LevelInfo, wall::WALL_THICKNESS},
};

const ARROW_SIZE: f32 = 40.; // Half the size of square

pub struct ArrowPlugin;

impl Plugin for ArrowPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component)]
pub struct Arrow;

pub fn spawn_arrow(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut level_info: ResMut<LevelInfo>,
) {
    let position = generate_valid_position(*window, &mut level_info);

    // Points for an arrow pointing up (completely flipped)
    let points = [
        Vec2::new(0.3, -1.0),  // Top right corner of the tail
        Vec2::new(0.3, 0.0),   // Right side where tail meets body
        Vec2::new(1.0, 0.0),   // Right point of arrowhead
        Vec2::new(0.0, 1.5),   // Tip of the arrowhead (pointing up)
        Vec2::new(-1.0, 0.0),  // Left point of arrowhead
        Vec2::new(-0.3, 0.0),  // Left side where tail meets body
        Vec2::new(-0.3, -1.0), // Top left corner of the tail
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
        LinearDamping(0.3),
        Restitution::new(0.5).with_combine_rule(CoefficientCombine::Average),
        get_enemy_layer(),
    ));
}

fn generate_valid_position(window: &Window, level_info: &mut LevelInfo) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let half_size = ARROW_SIZE;
    let margin = WALL_THICKNESS * 5.;

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
