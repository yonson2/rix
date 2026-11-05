//! World-space widgets ([036]). The game camera draws them, so they get the bloom and the
//! CRT pass; Bevy UI nodes draw after the CRT pass. The tiers ([127]): a primary button
//! (filled, its outline glows slowly), the outlined button, a box button whose caller adds
//! its content, and a text button with no box.

use bevy::{input::touch::Touches, prelude::*, window::PrimaryWindow};

use super::palette::{CYAN, NEON_CYAN, NEON_GRAY, NEON_WHITE, SUPER_NEON_CYAN};
use crate::shape::{self, Shape};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (press_buttons, glow_primary));
}

/// 260 x 76 world units: about 230 x 67 pt on a 390 pt wide phone, over the 44 pt minimum.
const BUTTON_SIZE: Vec2 = Vec2::new(260., 76.);
const PRESSED_SCALE: f32 = 0.94;
const CORNER: f32 = 12.;
const LINE: f32 = 3.;
const FILL_ALPHA: f32 = 0.12;
/// A primary button is filled more and its outline is thicker.
const PRIMARY_FILL_ALPHA: f32 = 0.38;
const PRIMARY_LINE: f32 = 4.;
const PRIMARY_LABEL: f32 = 44.;
/// One slow glow of a primary outline, in seconds.
const GLOW_SECS: f32 = 2.4;
const TEXT_LABEL: f32 = 20.;
/// A glyph of the game font is about this share of the font size wide.
const GLYPH_WIDTH: f32 = 0.6;
/// The space between a label and its button's sides.
const LABEL_PAD: f32 = 12.;

/// A neon box with a label. A press and a release inside it trigger [`Activate`] on it.
#[derive(Component)]
pub struct NeonButton {
    size: Vec2,
}

impl NeonButton {
    /// Whether the world point `p` is on the button.
    pub fn contains(&self, transform: &GlobalTransform, p: Vec2) -> bool {
        let offset = p - transform.translation().truncate();
        offset.abs().cmple(self.size / 2.).all()
    }
}

/// A [`NeonButton`] was pressed and released.
#[derive(EntityEvent)]
pub struct Activate {
    pub entity: Entity,
}

/// A button with `label`, centered at height `y`.
pub fn button(label: &str, font: Handle<Font>, y: f32) -> impl Bundle {
    button_at(label, font, Vec2::new(0., y), BUTTON_SIZE)
}

/// A button of `size` with `label`, centered at `position` ([067]: the seed keypad).
pub fn button_at(label: &str, font: Handle<Font>, position: Vec2, size: Vec2) -> impl Bundle {
    styled(label, font, position, size)
}

/// The screen's main action ([127]): filled, a large label, an outline that glows slowly.
pub fn primary_button(label: &str, font: Handle<Font>, position: Vec2, size: Vec2) -> impl Bundle {
    (
        Primary,
        NeonButton { size },
        Shape::new(shape::rounded_rectangle(size, CORNER))
            .fill(CYAN.with_alpha(PRIMARY_FILL_ALPHA))
            .stroke(NEON_CYAN, PRIMARY_LINE),
        Transform::from_translation(position.extend(1.)),
        children![label_text(label, font, PRIMARY_LABEL, NEON_WHITE)],
    )
}

/// A button from [`primary_button`].
#[derive(Component)]
struct Primary;

/// An outlined button with no label: the caller adds its content as children ([127]).
pub fn box_button(position: Vec2, size: Vec2) -> impl Bundle {
    (
        NeonButton { size },
        outline(size),
        Transform::from_translation(position.extend(1.)),
    )
}

/// A label with no box, for the least used actions ([127]); a tap on `size` around it
/// triggers [`Activate`].
pub fn text_button(label: &str, font: Handle<Font>, position: Vec2, size: Vec2) -> impl Bundle {
    (
        NeonButton { size },
        Transform::from_translation(position.extend(1.)),
        Visibility::default(),
        children![label_text(label, font, TEXT_LABEL, NEON_GRAY)],
    )
}

fn outline(size: Vec2) -> Shape {
    Shape::new(shape::rounded_rectangle(size, CORNER))
        .fill(CYAN.with_alpha(FILL_ALPHA))
        .stroke(NEON_CYAN, LINE)
}

fn label_text(label: &str, font: Handle<Font>, size: f32, color: Color) -> impl Bundle {
    (
        Text2d::new(label),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(0., 0., 0.1),
    )
}

fn styled(label: &str, font: Handle<Font>, position: Vec2, size: Vec2) -> impl Bundle {
    (
        NeonButton { size },
        outline(size),
        Transform::from_translation(position.extend(1.)),
        children![label_text(label, font, label_size(label, size), NEON_WHITE)],
    )
}

/// 45 % of the button's height, at most 34, and small enough that the label fits the
/// button's width ([052]: CREDITS in a half-width button).
fn label_size(label: &str, size: Vec2) -> f32 {
    let fit = (size.x - 2. * LABEL_PAD) / (GLYPH_WIDTH * label.chars().count() as f32);
    (size.y * 0.45).min(34.).min(fit)
}

/// Real time: the pause menu's RESUME glows while the game stands still.
fn glow_primary(time: Res<Time<Real>>, mut shapes: Query<&mut Shape, With<Primary>>) {
    let wave = 0.5 - 0.5 * ops::cos(time.elapsed_secs() * std::f32::consts::TAU / GLOW_SECS);
    let color = NEON_CYAN.mix(&SUPER_NEON_CYAN, wave);
    for mut shape in &mut shapes {
        if let Some(stroke) = &mut shape.stroke {
            stroke.color = color;
        }
    }
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
            .find(|(_, button, transform, _)| button.contains(transform, p))
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

    #[test]
    fn a_long_label_shrinks_to_fit_its_button() {
        let half = Vec2::new(123., 76.);
        assert_eq!(label_size("WIKI", half), 34.);
        let credits = label_size("CREDITS", half);
        assert!(credits < 34., "{credits}");
        assert!(credits * GLYPH_WIDTH * 7. <= half.x - 2. * LABEL_PAD);
        assert_eq!(label_size("PLAY SEED", BUTTON_SIZE), 34.);
    }
}
