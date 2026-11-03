//! World-space widgets ([036]). The game camera draws them, so they get the bloom and the
//! CRT pass; Bevy UI nodes draw after the CRT pass.

use bevy::{input::touch::Touches, prelude::*, window::PrimaryWindow};

use super::palette::{BASE_GRAY, CYAN, NEON_CYAN, NEON_WHITE};
use crate::shape::{self, Shape};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, press_buttons)
        .add_observer(cross_out);
}

/// 260 x 76 world units: about 230 x 67 pt on a 390 pt wide phone, over the 44 pt minimum.
const BUTTON_SIZE: Vec2 = Vec2::new(260., 76.);
const PRESSED_SCALE: f32 = 0.94;
/// The line across a locked button keeps this far inside its corners.
const LOCK_INSET: f32 = 8.;
const LOCK_LINE: f32 = 3.;
/// Gray, but light enough to read on the dark field.
const LOCKED_GRAY: Color = Color::srgb(0.52, 0.57, 0.6);
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
    styled(label, font, position, size, [CYAN, NEON_CYAN, NEON_WHITE])
}

/// A gray button with a line across it, as the bar of a no sign ([091]). A tap still
/// triggers [`Activate`].
pub fn locked_button(label: &str, font: Handle<Font>, y: f32) -> impl Bundle {
    (
        Locked,
        styled(
            label,
            font,
            Vec2::new(0., y),
            BUTTON_SIZE,
            [BASE_GRAY, LOCKED_GRAY, LOCKED_GRAY],
        ),
    )
}

/// A button from [`locked_button`].
#[derive(Component)]
pub struct Locked;

fn styled(
    label: &str,
    font: Handle<Font>,
    position: Vec2,
    size: Vec2,
    [fill, line, text]: [Color; 3],
) -> impl Bundle {
    (
        NeonButton { size },
        Shape::new(shape::rounded_rectangle(size, 12.))
            .fill(fill.with_alpha(0.12))
            .stroke(line, 3.),
        Transform::from_translation(position.extend(1.)),
        children![(
            Text2d::new(label),
            TextFont {
                font: font.into(),
                font_size: FontSize::Px(label_size(label, size)),
                ..default()
            },
            TextColor(text),
            Transform::from_xyz(0., 0., 0.1),
        )],
    )
}

/// 45 % of the button's height, at most 34, and small enough that the label fits the
/// button's width ([052]: CREDITS in a half-width button).
fn label_size(label: &str, size: Vec2) -> f32 {
    let fit = (size.x - 2. * LABEL_PAD) / (GLYPH_WIDTH * label.chars().count() as f32);
    (size.y * 0.45).min(34.).min(fit)
}

fn cross_out(add: On<Add<Locked>>, buttons: Query<&NeonButton>, mut commands: Commands) {
    let Ok(button) = buttons.get(add.entity) else {
        return;
    };
    let corner = button.size / 2. - LOCK_INSET;
    commands.entity(add.entity).with_child((
        Sprite::from_color(LOCKED_GRAY, Vec2::new(2. * corner.length(), LOCK_LINE)),
        Transform::from_xyz(0., 0., 0.2)
            .with_rotation(Quat::from_rotation_z(ops::atan2(corner.y, corner.x))),
    ));
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

    #[test]
    fn a_locked_button_gets_a_line_from_corner_to_corner() {
        let mut world = World::new();
        world.add_observer(cross_out);
        let button = world
            .spawn(locked_button("MUSIC", Handle::default(), 0.))
            .id();
        world.flush();
        let mut lines = world.query::<(&Sprite, &Transform, &ChildOf)>();
        let (sprite, transform, parent) = lines.single(&world).unwrap();
        assert_eq!(parent.parent(), button);
        assert_eq!(sprite.color, LOCKED_GRAY);
        let along = transform.rotation * Vec3::X * sprite.custom_size.unwrap().x / 2.;
        let corner = (BUTTON_SIZE / 2. - LOCK_INSET).extend(0.);
        assert!(along.distance(corner) < 0.01, "{along} reaches {corner}");
    }
}
