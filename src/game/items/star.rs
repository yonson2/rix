//! The Star ([069]): for a while nothing kills the player. An enemy or a bullet that
//! touches the line does no harm (an enemy passes through it: the line has no body), and
//! the line cycles the rainbow. A new star starts the time over.

use bevy::prelude::*;

use crate::{
    game::{AppSystems, PausableSystems, polygon::TempLineSegment, state::Phase},
    shape::{self, Shape},
    theme::palette::NEON_WHITE,
    utils::update_sprite_color,
};

pub const STAR_SECS: f32 = 6.;
/// Degrees of hue per second, and per world unit along the line.
const HUE_SPEED: f32 = 360.;
const HUE_PER_UNIT: f32 = 0.8;
/// Above 1 so the line blooms, as the neon palette does.
const GLOW: f32 = 1.8;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_star
            .run_if(resource_exists::<Star>)
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        rainbow_line
            .run_if(resource_exists::<Star>)
            .after(update_sprite_color)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(OnExit(Phase::Playing), |mut commands: Commands| {
        commands.remove_resource::<Star>();
    });
}

/// While it exists, the player cannot die.
#[derive(Resource, Deref, DerefMut)]
pub struct Star(Timer);

impl Default for Star {
    fn default() -> Self {
        Self(Timer::from_seconds(STAR_SECS, TimerMode::Once))
    }
}

fn tick_star(mut commands: Commands, time: Res<Time>, mut star: ResMut<Star>) {
    if star.tick(time.delta()).is_finished() {
        commands.remove_resource::<Star>();
    }
}

/// Draws only.
fn rainbow_line(
    time: Res<Time>,
    mut line: Query<(&mut Sprite, &Transform), With<TempLineSegment>>,
) {
    for (mut sprite, transform) in &mut line {
        sprite.color = rainbow(time.elapsed_secs(), transform.translation.truncate());
    }
}

/// A bright hue that runs along the line over time.
fn rainbow(secs: f32, at: Vec2) -> Color {
    let hue = (secs * HUE_SPEED + (at.x + at.y) * HUE_PER_UNIT).rem_euclid(360.);
    let c = Color::hsl(hue, 1., 0.6).to_srgba();
    Color::srgb(c.red * GLOW, c.green * GLOW, c.blue * GLOW)
}

pub fn icon() -> Shape {
    let points: Vec<Vec2> = (0..10)
        .map(|i| {
            let radius = if i % 2 == 0 { 14. } else { 6. };
            let angle = std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.;
            Vec2::new(ops::cos(angle), ops::sin(angle)) * radius
        })
        .collect();
    Shape::new(shape::polygon(&points))
        .fill(Color::WHITE)
        .stroke(NEON_WHITE, 2.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rainbow_moves_along_the_line_and_over_time() {
        let hue = |c: Color| Hsla::from(c).hue;
        let here = rainbow(0., Vec2::ZERO);
        assert_ne!(hue(here), hue(rainbow(0., Vec2::new(100., 0.))));
        assert_ne!(hue(here), hue(rainbow(0.25, Vec2::ZERO)));
        let c = here.to_srgba();
        assert!(c.red.max(c.green).max(c.blue) > 1., "it glows");
    }
}
