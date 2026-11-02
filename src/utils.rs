use avian2d::prelude::*;
use bevy::prelude::*;
use fastrand::Rng;

use crate::{
    rix::wall::{WALL_THICKNESS, WALL_THICKNESS_TOP},
    shape::Shape,
};

#[derive(Component)]
pub struct ColorTransition {
    pub timer: Timer,
    pub from: Color,
    pub to: Color,
}

#[derive(Component)]
pub struct FadeShape {
    pub timer: Timer,
    pub direction: Option<Vec2>,
}

impl FadeShape {
    pub fn from_seconds(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            direction: None,
        }
    }

    pub fn with_direction(duration: f32, seed: f32) -> Self {
        let angle = seed * std::f32::consts::TAU;
        let direction = Vec2::new(angle.cos(), angle.sin());

        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            direction: Some(direction),
        }
    }
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

        let factor = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };

        let current_color = transition.from.mix(&transition.to, factor);

        if let Some(mut material) = materials.get_mut(&mesh_material.0) {
            material.color = current_color;
        }

        if transition.timer.is_finished() && transition.timer.mode() == TimerMode::Once {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}

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

        if transition.timer.is_finished() && transition.timer.mode() == TimerMode::Once {
            commands.entity(entity).remove::<ColorTransition>();
        }
    }
}

pub fn update_shape_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FadeShape, &mut Shape, &mut Transform)>,
) {
    for (entity, mut fade, mut shape, mut transform) in &mut query {
        fade.timer.tick(time.delta());

        let progress = fade.timer.fraction();
        let alpha = 1.0 - progress;

        if let Some(fill) = &mut shape.fill {
            fill.set_alpha(alpha);
        }

        if let Some(stroke) = &mut shape.stroke {
            stroke.color.set_alpha(alpha);
        }

        if let Some(direction) = fade.direction {
            let speed_factor = 100.0 * progress * progress;
            let movement = direction * speed_factor * time.delta_secs();

            transform.translation.x += movement.x;
            transform.translation.y += movement.y;

            let scale_factor = 1.0 + progress * 0.5;
            transform.scale = Vec3::splat(scale_factor);
        }

        if fade.timer.is_finished() && fade.timer.mode() == TimerMode::Once {
            if let Ok(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

pub fn generate_enemy_position(
    // Renamed slightly for clarity
    window: &Window,
    spatial_query: &SpatialQuery,
    rng: &mut Rng,
    enemy_collider: &Collider,
) -> Vec2 {
    // Returns Vec2 directly
    let (width, height) = (window.width(), window.height());
    let margin_horizontal = WALL_THICKNESS * 2.;
    let margin_top = WALL_THICKNESS_TOP * 2.;
    let margin_bottom = WALL_THICKNESS * 2.;
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    // Define the spawnable area boundaries
    let min_x = -half_width + margin_horizontal;
    let max_x = half_width - margin_horizontal;
    let min_y = -half_height + margin_bottom;
    let max_y = half_height - margin_top;

    // Basic check to prevent panic/infinite loop if bounds are initially invalid
    if min_x >= max_x || min_y >= max_y {
        error!(
            "Spawn area is invalid (min >= max)! Check window dimensions/margins. Cannot guarantee finding a position. Returning Vec2::ZERO as fallback."
        );
        // Consider panicking here if this state should truly be impossible:
        // panic!("Invalid spawn area configuration!");
        return Vec2::ZERO;
    }

    // Loop indefinitely
    let mut attemps = 10000;
    let (mut x, mut y) = (0., 0.);
    while attemps > 0 {
        attemps -= 1;
        // Generate a random candidate position
        // Ensure max > min which we checked above
        x = rng.f32() * (max_x - min_x) + min_x;
        y = rng.f32() * (max_y - min_y) + min_y;
        let candidate_position = Vec2::new(x, y);

        // Define the spatial query filter. You might customize this.
        let query_filter = SpatialQueryFilter::default();

        // Check for intersections at the candidate position
        let intersections = spatial_query.shape_intersections(
            enemy_collider,
            candidate_position,
            f32::default(),
            &query_filter,
        );

        // If the returned vector is empty, no intersections were found!
        if intersections.is_empty() {
            // Found a valid position, return it and exit the loop/function
            return candidate_position;
        }
    }
    warn!("Valid position not found, returning a random one");
    Vec2::new(x, y)
}

pub fn generate_item_position(window: &Window, rng: &mut fastrand::Rng) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    Vec2::new(
        WALL_THICKNESS + (rng.f32() * half_width - WALL_THICKNESS),
        100. + half_height + (WALL_THICKNESS_TOP - WALL_THICKNESS),
    )
}
