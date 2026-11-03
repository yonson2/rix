//! The auto-line ([046]): an arrow. The line in progress shoots straight on, the way it was
//! going, to the boundary of the free region, and claims. `user_input::handle_input` fires
//! the shot; a shot that would cross the line does not fire.

use bevy::prelude::*;

use crate::{
    shape::{self, Shape},
    theme::palette::{GREEN, NEON_GREEN},
};

/// The heading is the way the line went over its last this many units.
const HEADING_LEN: f32 = 40.;

/// A point on the line's heading past the far side of `free`.
pub fn shot_target(path: &[Vec2], free: &[Vec2]) -> Option<Vec2> {
    let end = *path.last()?;
    let back = path
        .iter()
        .rev()
        .find(|p| p.distance(end) >= HEADING_LEN)
        .or(path.first())?;
    let heading = (end - *back).try_normalize()?;
    let reach = free.iter().map(|v| v.distance(end)).fold(0., f32::max) + 1.;
    Some(end + heading * reach)
}

pub fn icon() -> Shape {
    let arrow = [
        Vec2::new(-13., 3.),
        Vec2::new(2., 3.),
        Vec2::new(2., 9.),
        Vec2::new(12., 0.),
        Vec2::new(2., -9.),
        Vec2::new(2., -3.),
        Vec2::new(-13., -3.),
    ];
    Shape::new(shape::polygon(&arrow))
        .fill(GREEN)
        .stroke(NEON_GREEN, 2.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shot_follows_the_last_40_units_past_the_free_region() {
        let free = [
            Vec2::new(-100., -100.),
            Vec2::new(100., -100.),
            Vec2::new(100., 100.),
            Vec2::new(-100., 100.),
        ];
        // A turn long ago does not count; the last 40 units go up and to the right.
        let path = [
            Vec2::new(-100., 0.),
            Vec2::new(-50., 0.),
            Vec2::new(-20., 30.),
            Vec2::new(0., 50.),
        ];
        let target = shot_target(&path, &free).unwrap();
        let heading = (target - path[3]).normalize();
        assert!(heading.distance(Vec2::ONE.normalize()) < 1e-4, "{heading}");
        assert!(target.x > 100. && target.y > 100.);

        let short = [Vec2::new(-100., 0.), Vec2::new(-90., 0.)];
        assert!(
            shot_target(&short, &free).unwrap().x > 100.,
            "a short line uses all of it"
        );
        assert_eq!(shot_target(&path[..1], &free), None, "no heading yet");
    }
}
