//! The settings screen ([048]): one row per setting, its name on the left and its value on
//! the right ([127]); a tap steps it (volumes and the CRT down by 20 % and from 0 back to
//! 100, switches on and off). The save keeps them. The pause menu shows the same panel
//! ([073]). MUSIC stays locked until the soundtrack opens ([089]): a gray row that says
//! when.

use bevy::{prelude::*, sprite::Anchor};

use super::Screen;
use crate::{
    save::{SOUNDTRACK_LEVEL, Save, step_down},
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, NEON_GRAY, NEON_GREEN, NEON_WHITE, NEON_YELLOW},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), setup)
        .add_systems(Update, relabel.run_if(resource_changed::<Save>));
}

const TITLE_Y: f32 = 330.;
const FIRST_Y: f32 = 230.;
const ROW_GAP: f32 = 80.;
const ROW_SIZE: Vec2 = Vec2::new(380., 64.);
/// The name and the value keep this far inside a row's sides.
const TEXT_X: f32 = 166.;
const TEXT_SIZE: f32 = 26.;
const BACK_Y: f32 = -340.;

/// A row's value text.
#[derive(Component)]
struct Value;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum Setting {
    Music,
    Sound,
    Crt,
    Shake,
    Haptics,
    Flashing,
}

const ROWS: [Setting; 6] = [
    Setting::Music,
    Setting::Sound,
    Setting::Crt,
    Setting::Shake,
    Setting::Haptics,
    Setting::Flashing,
];

impl Setting {
    /// MUSIC waits for the soundtrack to open ([089]).
    fn locked(self, save: &Save) -> bool {
        self == Setting::Music && !save.soundtrack_open()
    }

    fn name(self) -> &'static str {
        match self {
            Setting::Music => "MUSIC",
            Setting::Sound => "SOUND",
            Setting::Crt => "CRT",
            Setting::Shake => "SHAKE",
            Setting::Haptics => "HAPTICS",
            Setting::Flashing => "FLASH",
        }
    }

    /// The value and its color: a percent in yellow, ON in green, OFF in gray.
    fn value(self, save: &Save) -> (String, Color) {
        let settings = &save.settings;
        let percent = |value: u8| (format!("{value}%"), NEON_YELLOW);
        let switch = |on: bool| match on {
            true => ("ON".to_string(), NEON_GREEN),
            false => ("OFF".to_string(), NEON_GRAY),
        };
        match self {
            Setting::Music if self.locked(save) => {
                (format!("REACH LEVEL {SOUNDTRACK_LEVEL}"), BASE_GRAY)
            }
            Setting::Music => percent(settings.music),
            Setting::Sound => percent(settings.sound),
            Setting::Crt => percent(settings.crt),
            Setting::Shake => switch(settings.shake),
            Setting::Haptics => switch(settings.haptics),
            Setting::Flashing => switch(settings.flashing),
        }
    }

    fn step(self, save: &mut Save) {
        if self.locked(save) {
            return;
        }
        let settings = &mut save.settings;
        match self {
            Setting::Music => settings.music = step_down(settings.music),
            Setting::Sound => settings.sound = step_down(settings.sound),
            Setting::Crt => settings.crt = step_down(settings.crt),
            Setting::Shake => settings.shake ^= true,
            Setting::Haptics => settings.haptics ^= true,
            Setting::Flashing => settings.flashing ^= true,
        }
    }
}

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let root = (
        DespawnOnExit(Screen::Settings),
        Transform::default(),
        Visibility::default(),
    );
    let back = spawn_panel(&mut commands, &fonts.main, &save, root);
    commands
        .entity(back)
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

/// The title, one row per setting and BACK, as children of `root`: it holds their
/// lifetime and height. Returns BACK, for the caller's observer.
pub fn spawn_panel(
    commands: &mut Commands,
    font: &Handle<Font>,
    save: &Save,
    root: impl Bundle,
) -> Entity {
    let mut back = Entity::PLACEHOLDER;
    commands.spawn(root).with_children(|panel| {
        panel.spawn((
            Text2d::new("SETTINGS"),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(54.),
                ..default()
            },
            TextColor(NEON_WHITE),
            Transform::from_xyz(0., TITLE_Y, 1.),
        ));
        let text = |text: &str, color: Color| {
            (
                Text2d::new(text),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(TEXT_SIZE),
                    ..default()
                },
                TextColor(color),
            )
        };
        for (row, setting) in ROWS.into_iter().enumerate() {
            let at = Vec2::new(0., FIRST_Y - row as f32 * ROW_GAP);
            let (value, color) = setting.value(save);
            let (mut row, name) = match setting.locked(save) {
                false => (
                    panel.spawn((setting, widget::box_button(at, ROW_SIZE))),
                    NEON_WHITE,
                ),
                true => (
                    panel.spawn((
                        setting,
                        Shape::new(shape::rounded_rectangle(ROW_SIZE, 12.))
                            .fill(BASE_GRAY.with_alpha(0.12))
                            .stroke(NEON_GRAY, 3.),
                        Transform::from_translation(at.extend(1.)),
                    )),
                    NEON_GRAY,
                ),
            };
            row.observe(step).with_children(|row| {
                row.spawn((
                    text(setting.name(), name),
                    Anchor::CENTER_LEFT,
                    Transform::from_xyz(-TEXT_X, 0., 0.1),
                ));
                row.spawn((
                    Value,
                    text(&value, color),
                    Anchor::CENTER_RIGHT,
                    Transform::from_xyz(TEXT_X, 0., 0.1),
                ));
            });
        }
        back = panel
            .spawn(widget::button("BACK", font.clone(), BACK_Y))
            .id();
    });
    back
}

fn step(activate: On<Activate>, settings: Query<&Setting>, mut save: ResMut<Save>) {
    if let Ok(setting) = settings.get(activate.entity) {
        setting.step(&mut save);
    }
}

fn relabel(
    save: Res<Save>,
    rows: Query<(&Setting, &Children)>,
    mut values: Query<(&mut Text2d, &mut TextColor), With<Value>>,
) {
    for (setting, children) in &rows {
        for child in children {
            if let Ok((mut text, mut text_color)) = values.get_mut(*child) {
                (text.0, text_color.0) = setting.value(&save);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_setting_steps_and_shows_its_value() {
        let mut save = Save {
            best_level: SOUNDTRACK_LEVEL,
            ..default()
        };
        save.settings.music = 60;
        let values = |save: &Save| ROWS.map(|row| row.value(save).0);
        assert_eq!(values(&save), ["60%", "100%", "100%", "ON", "ON", "ON"]);
        for row in ROWS {
            row.step(&mut save);
        }
        assert_eq!(values(&save), ["40%", "80%", "80%", "OFF", "OFF", "OFF"]);
        assert_eq!(Setting::Shake.value(&save).1, NEON_GRAY, "OFF is gray");
    }

    #[test]
    fn music_is_locked_until_the_soundtrack_opens_then_starts_off() {
        let mut save = Save::default();
        assert!(Setting::Music.locked(&save));
        assert_eq!(Setting::Music.value(&save).0, "REACH LEVEL 3");
        Setting::Music.step(&mut save);
        assert_eq!(save.settings.music, 0, "a tap does nothing while locked");

        save.best_level = SOUNDTRACK_LEVEL;
        assert!(!Setting::Music.locked(&save));
        assert_eq!(Setting::Music.value(&save).0, "0%");
        Setting::Music.step(&mut save);
        assert_eq!(Setting::Music.value(&save).0, "100%");
    }

    #[test]
    fn only_a_locked_music_row_takes_no_tap() {
        let mut world = World::new();
        spawn_panel(
            &mut world.commands(),
            &Handle::default(),
            &Save::default(),
            (),
        );
        world.flush();
        let mut rows = world.query::<(&Setting, Has<widget::NeonButton>)>();
        for (setting, button) in rows.iter(&world) {
            assert_eq!(button, *setting != Setting::Music, "{setting:?}");
        }
    }
}
