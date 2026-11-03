//! World-space widgets ([036]). The game camera draws them, so they get the bloom and the
//! CRT pass; Bevy UI nodes draw after the CRT pass.

use bevy::{input::touch::Touches, prelude::*, window::PrimaryWindow};

use super::palette::{CYAN, NEON_CYAN, NEON_WHITE};
use crate::shape::{self, Shape};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, press_buttons);
}

/// 260 x 76 world units: about 230 x 67 pt on a 390 pt wide phone, over the 44 pt minimum.
const BUTTON_SIZE: Vec2 = Vec2::new(260., 76.);
const PRESSED_SCALE: f32 = 0.94;

/// A neon box with a label. A press and a release inside it trigger [`Activate`] on it.
#[derive(Component)]
pub struct NeonButton {
    size: Vec2,
}

/// A [`NeonButton`] was pressed and released.
#[derive(EntityEvent)]
pub struct Activate {
    pub entity: Entity,
}

/// A button with `label`, centered at height `y`.
pub fn button(label: &str, font: Handle<Font>, y: f32) -> impl Bundle {
    (
        NeonButton { size: BUTTON_SIZE },
        Shape::new(shape::rounded_rectangle(BUTTON_SIZE, 12.))
            .fill(CYAN.with_alpha(0.12))
            .stroke(NEON_CYAN, 3.),
        Transform::from_xyz(0., y, 1.),
        children![(
            Text2d::new(label),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(34.),
                ..default()
            },
            TextColor(NEON_WHITE),
            Transform::from_xyz(0., 0., 0.1),
        )],
    )
}

fn press_buttons(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut buttons: Query<(Entity, &NeonButton, &GlobalTransform, &mut Transform)>,
    mut pressed: Local<Option<Entity>>,
) {
    let (camera, camera_transform) = *camera;
    let cursor = window.cursor_position();
    let press = (touches.iter_just_pressed().next().map(|t| t.position()))
        .or(cursor.filter(|_| mouse.just_pressed(MouseButton::Left)));
    let release = (touches.iter_just_released().next().map(|t| t.position()))
        .or(cursor.filter(|_| mouse.just_released(MouseButton::Left)));
    let hit = |screen: Vec2| {
        let p = camera.viewport_to_world_2d(camera_transform, screen).ok()?;
        buttons
            .iter()
            .find(|(_, button, transform, _)| {
                let offset = p - transform.translation().truncate();
                offset.abs().cmple(button.size / 2.).all()
            })
            .map(|(entity, ..)| entity)
    };
    if let Some(entity) = track(&mut pressed, press, release, hit) {
        commands.trigger(Activate { entity });
    }

    for (entity, _, _, mut transform) in &mut buttons {
        let scale = Vec3::splat(if *pressed == Some(entity) {
            PRESSED_SCALE
        } else {
            1.
        });
        if transform.scale != scale {
            transform.scale = scale;
        }
    }
}

/// The button a press lands on stays pressed until the release; a release on the same
/// button activates it.
fn track(
    pressed: &mut Option<Entity>,
    press: Option<Vec2>,
    release: Option<Vec2>,
    hit: impl Fn(Vec2) -> Option<Entity>,
) -> Option<Entity> {
    if let Some(p) = press {
        *pressed = hit(p);
    }
    let p = release?;
    pressed.take().filter(|&entity| hit(p) == Some(entity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_and_a_release_on_the_same_button_activate_it() {
        let (a, b) = (
            Entity::from_raw_u32(1).unwrap(),
            Entity::from_raw_u32(2).unwrap(),
        );
        // Button `a` left of x = 0, `b` right of it.
        let hit = |p: Vec2| Some(if p.x < 0. { a } else { b });
        let (left, right) = (Some(Vec2::new(-1., 0.)), Some(Vec2::new(1., 0.)));
        let mut pressed = None;

        assert_eq!(track(&mut pressed, left, None, hit), None);
        assert_eq!(pressed, Some(a), "a stays pressed until the release");
        assert_eq!(track(&mut pressed, None, left, hit), Some(a));
        assert_eq!(pressed, None);

        assert_eq!(track(&mut pressed, left, right, hit), None, "released on b");
        assert_eq!(
            track(&mut pressed, None, left, hit),
            None,
            "no press before"
        );
        assert_eq!(
            track(&mut pressed, left, left, hit),
            Some(a),
            "a tap in one frame"
        );
    }
}
