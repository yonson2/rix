//! The settings screen ([048]): one button per setting; a tap steps it (volumes and the
//! CRT down by 20 % and from 0 back to 100, switches on and off). The save keeps them.

use bevy::prelude::*;

use super::Screen;
use crate::{
    save::{Save, Settings, step_down},
    theme::{
        FontAssets,
        palette::NEON_WHITE,
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), setup)
        .add_systems(
            Update,
            relabel
                .run_if(in_state(Screen::Settings))
                .run_if(resource_changed::<Save>),
        );
}

const TITLE_Y: f32 = 330.;
const FIRST_Y: f32 = 230.;
const ROW_GAP: f32 = 90.;
const BACK_Y: f32 = -340.;

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
    fn label(self, settings: &Settings) -> String {
        let percent = |name: &str, value: u8| format!("{name} {value}%");
        let switch = |name: &str, on: bool| format!("{name} {}", if on { "ON" } else { "OFF" });
        match self {
            Setting::Music => percent("MUSIC", settings.music),
            Setting::Sound => percent("SOUND", settings.sound),
            Setting::Crt => percent("CRT", settings.crt),
            Setting::Shake => switch("SHAKE", settings.shake),
            Setting::Haptics => switch("HAPTICS", settings.haptics),
            Setting::Flashing => switch("FLASH", settings.flashing),
        }
    }

    fn step(self, settings: &mut Settings) {
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
    commands.spawn((
        DespawnOnExit(Screen::Settings),
        Text2d::new("SETTINGS"),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(54.),
            ..default()
        },
        TextColor(NEON_WHITE),
        Transform::from_xyz(0., TITLE_Y, 1.),
    ));
    for (row, setting) in ROWS.into_iter().enumerate() {
        let y = FIRST_Y - row as f32 * ROW_GAP;
        commands
            .spawn((
                DespawnOnExit(Screen::Settings),
                setting,
                widget::button(&setting.label(&save.settings), fonts.main.clone(), y),
            ))
            .observe(step);
    }
    commands
        .spawn((
            DespawnOnExit(Screen::Settings),
            widget::button("BACK", fonts.main.clone(), BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

fn step(activate: On<Activate>, settings: Query<&Setting>, mut save: ResMut<Save>) {
    if let Ok(setting) = settings.get(activate.entity) {
        setting.step(&mut save.settings);
    }
}

fn relabel(save: Res<Save>, rows: Query<(&Setting, &Children)>, mut labels: Query<&mut Text2d>) {
    for (setting, children) in &rows {
        for child in children {
            if let Ok(mut label) = labels.get_mut(*child) {
                label.0 = setting.label(&save.settings);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_setting_steps_and_shows_its_value() {
        let mut settings = Settings::default();
        let labels = |settings: &Settings| ROWS.map(|row| row.label(settings));
        assert_eq!(
            labels(&settings),
            [
                "MUSIC 60%",
                "SOUND 100%",
                "CRT 100%",
                "SHAKE ON",
                "HAPTICS ON",
                "FLASH ON"
            ]
        );
        for row in ROWS {
            row.step(&mut settings);
        }
        assert_eq!(
            labels(&settings),
            [
                "MUSIC 40%",
                "SOUND 80%",
                "CRT 80%",
                "SHAKE OFF",
                "HAPTICS OFF",
                "FLASH OFF"
            ]
        );
    }
}
