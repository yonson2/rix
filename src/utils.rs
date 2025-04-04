use avian2d::prelude::*;
use bevy::prelude::*;

use crate::rix::wall::{WALL_THICKNESS, WALL_THICKNESS_TOP};

#[derive(Component)]
pub struct ColorTransition {
    pub timer: Timer,
    pub from: Color,
    pub to: Color,
}

pub fn update_mesh_color<T: Component>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ColorTransition, &MeshMaterial2d<ColorMaterial>), With<T>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transition, mesh_material) in &mut query {
        transition.timer.tick(time.delta());

        let t = transition.timer.fraction();

        let factor = if t < 0.5 {
            t * 2.0 // Scale 0.0-0.5 to 0.0-1.0
        } else {
            (1.0 - t) * 2.0 // Scale 0.5-1.0 to 1.0-0.0
        };

        let current_color = transition.from.mix(&transition.to, factor);

        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.color = current_color;
        }

        if transition.timer.finished() && transition.timer.mode() == TimerMode::Once {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}

//TODO: Deduplicate
pub fn update_sprite_color(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ColorTransition, &mut Sprite)>,
) {
    for (entity, mut transition, mut sprite) in &mut query {
        transition.timer.tick(time.delta());

        let t = transition.timer.fraction();
        let factor = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };

        let current_color = transition.from.mix(&transition.to, factor);

        sprite.color = current_color;

        if transition.timer.finished() && transition.timer.mode() == TimerMode::Once {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}

pub fn generate_enemy_position(
    window: &Window,
    spatial_query: SpatialQuery,
    rng: &mut fastrand::Rng,
    collider: &Collider,
) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let margin_horizontal = WALL_THICKNESS * 2.; //NOTE: the *2. might be because of scale_factor
    let margin_top = WALL_THICKNESS_TOP * 2.;
    let margin_bottom = WALL_THICKNESS * 2.;
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    let min_x = -half_width + margin_horizontal;
    let max_x = half_width - margin_horizontal;
    let min_y = -half_height + margin_bottom;
    let max_y = half_height - margin_top;

    let mut position = Vec2::ZERO;
    loop {
        let x = min_x + rng.f32() * (max_x - min_x);
        let y = min_y + rng.f32() * (max_y - min_y);

        let intersections = spatial_query.shape_intersections(
            collider,
            Vec2::new(x, y),
            0.,
            &SpatialQueryFilter::default(),
        );
        if intersections.is_empty() {
            position.x = x;
            position.y = y;
            break;
        }
    }

    position
}

pub fn generate_item_position(window: &Window, rng: &mut fastrand::Rng) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    Vec2::new(
        WALL_THICKNESS + (rng.f32() * half_width - WALL_THICKNESS),
        100. + half_height + (WALL_THICKNESS_TOP - WALL_THICKNESS), // Adjust for the taller top wall
    )
}
