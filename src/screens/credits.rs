//! The credits ([052]): who made Rix and what it uses. `CREDITS.md` has the same list with
//! the sources; `assets/licenses/` has the licence texts that ship with the app.

use bevy::prelude::*;

use super::Screen;
use crate::theme::{
    FontAssets,
    palette::{NEON_CYAN, NEON_WHITE},
    widget::{self, Activate},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), setup);
}

const TITLE_Y: f32 = 330.;
const FIRST_Y: f32 = 265.;
const HEADING_GAP: f32 = 30.;
const LINE_GAP: f32 = 24.;
const SECTION_GAP: f32 = 16.;
const BACK_Y: f32 = -340.;

/// Each section: a heading and its lines.
const SECTIONS: [(&str, &[&str]); 5] = [
    ("GAME", &["PEDRO"]),
    ("INSPIRED BY", &["QIX (TAITO, 1981)", "XONIX (1984)"]),
    (
        "FONT",
        &["\"BALATRO\" BY LOCALTHUNK", "SIL OPEN FONT LICENSE 1.1"],
    ),
    (
        "SOUND",
        &["EFFECTS MADE WITH ELEVENLABS", "MUSIC MADE WITH SUNO"],
    ),
    (
        "CODE",
        &[
            "BUILT WITH BEVY AND RUST",
            "LICENCES: GITHUB.COM/YONSON2/RIX",
        ],
    ),
];

fn label(font: &Handle<Font>, text: &str, size: f32, color: Color, y: f32) -> impl Bundle {
    (
        DespawnOnExit(Screen::Credits),
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

fn setup(mut commands: Commands, fonts: Res<FontAssets>) {
    let font = &fonts.main;
    commands.spawn(label(font, "CREDITS", 54., NEON_WHITE, TITLE_Y));
    let mut y = FIRST_Y;
    for (heading, lines) in SECTIONS {
        commands.spawn(label(font, heading, 22., NEON_CYAN, y));
        y -= HEADING_GAP;
        for line in lines {
            commands.spawn(label(font, line, 18., NEON_WHITE, y));
            y -= LINE_GAP;
        }
        y -= SECTION_GAP;
    }
    commands
        .spawn((
            DespawnOnExit(Screen::Credits),
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
    fn the_credits_fit_above_the_back_button() {
        let lines: usize = SECTIONS.iter().map(|(_, lines)| lines.len()).sum();
        let last =
            FIRST_Y - SECTIONS.len() as f32 * (HEADING_GAP + SECTION_GAP) - lines as f32 * LINE_GAP;
        assert!(last > BACK_Y + 40., "the last line ends at {last}");
    }
}
