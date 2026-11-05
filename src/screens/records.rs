//! The RECORDS screen ([111]): every medal with its rule, the earned ones lit.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::medals::Medal,
    save::Save,
    theme::{
        FontAssets,
        palette::{BASE_GRAY, NEON_CYAN, NEON_GRAY, NEON_WHITE, NEON_YELLOW},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Records), setup);
}

const TITLE_Y: f32 = 330.;
const COUNT_Y: f32 = 275.;
const FIRST_Y: f32 = 225.;
/// From one medal's name to the next.
const MEDAL_GAP: f32 = 62.;
/// From a medal's name down to its rule.
const RULE_GAP: f32 = 24.;
const BACK_Y: f32 = -340.;

fn label(font: &Handle<Font>, text: &str, size: f32, color: Color, y: f32) -> impl Bundle {
    (
        DespawnOnExit(Screen::Records),
        Text2d::new(text),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(0., y, 1.),
    )
}

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font = &fonts.main;
    let earned = |medal: Medal| save.medals.contains(&medal.id());
    let count = Medal::ALL.into_iter().filter(|&m| earned(m)).count();
    commands.spawn(label(font, "RECORDS", 54., NEON_WHITE, TITLE_Y));
    let count = format!("MEDALS {count} / {}", Medal::ALL.len());
    commands.spawn(label(font, &count, 22., NEON_CYAN, COUNT_Y));
    for (i, medal) in Medal::ALL.into_iter().enumerate() {
        let y = FIRST_Y - i as f32 * MEDAL_GAP;
        let (name, rule) = match earned(medal) {
            true => (NEON_YELLOW, NEON_WHITE),
            false => (NEON_GRAY, BASE_GRAY),
        };
        commands.spawn(label(font, medal.name(), 22., name, y));
        commands.spawn(label(font, medal.rule(), 16., rule, y - RULE_GAP));
    }
    commands
        .spawn((
            DespawnOnExit(Screen::Records),
            widget::button("BACK", font.clone(), BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_medals_fit_above_the_back_button() {
        let last = FIRST_Y - (Medal::ALL.len() - 1) as f32 * MEDAL_GAP - RULE_GAP;
        assert!(last > BACK_Y + 60., "the last rule sits at {last}");
    }
}
