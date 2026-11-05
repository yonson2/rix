//! The RECORDS screen ([111], [113]): the lifetime totals, the next look and the next
//! challenge ([123]) that games played unlock, and every medal with its rule, the earned
//! ones lit.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{challenges, medals::Medal, setup::Points, skins},
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
/// The totals in two columns, from the top.
const TOTALS_Y: f32 = 278.;
const TOTALS_GAP: f32 = 26.;
const COLUMN_X: f32 = 108.;
const NEXT_Y: f32 = 172.;
const NEXT_CHALLENGE_Y: f32 = 150.;
const COUNT_Y: f32 = 114.;
const FIRST_Y: f32 = 78.;
/// From one medal's name to the next.
const MEDAL_GAP: f32 = 46.;
/// From a medal's name down to its rule.
const RULE_GAP: f32 = 20.;
const BACK_Y: f32 = -340.;

fn label(font: &Handle<Font>, text: &str, size: f32, color: Color, y: f32) -> impl Bundle {
    label_at(font, text, size, color, Vec2::new(0., y))
}

fn label_at(font: &Handle<Font>, text: &str, size: f32, color: Color, at: Vec2) -> impl Bundle {
    (
        DespawnOnExit(Screen::Records),
        Text2d::new(text),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(at.extend(1.)),
    )
}

/// Play time as hours and minutes, or minutes and seconds under an hour.
fn play_time(seconds: f64) -> String {
    let total = seconds as u64;
    let (hours, minutes, secs) = (total / 3600, total / 60 % 60, total % 60);
    match hours {
        0 => format!("{minutes}M {secs:02}S"),
        _ => format!("{hours}H {minutes:02}M"),
    }
}

/// The totals, row by row, two to a row.
fn totals(save: &Save) -> [[String; 2]; 4] {
    let n = |count: u64| Points(count.into()).to_string();
    let stats = &save.stats;
    [
        [
            format!("BEST {}", Points(save.best)),
            format!("BEST LEVEL {}", save.best_level),
        ],
        [
            format!("GAMES {}", n(stats.games())),
            format!("CLAIMS {}", n(stats.claims)),
        ],
        [
            format!("KILLS {}", n(stats.kills)),
            format!("LEVELS {}", n(stats.levels)),
        ],
        [format!("TIME {}", play_time(stats.seconds)), String::new()],
    ]
}

/// The next look that games played unlock, with the progress to it.
fn next_unlock(save: &Save) -> Option<String> {
    let games = save.stats.games();
    let (name, at) = skins::next_by_games(games)?;
    Some(format!("NEXT: {name} ({games} / {at} GAMES)"))
}

/// The next challenge that games played open ([123]).
fn next_challenge(save: &Save) -> Option<String> {
    let games = save.stats.games();
    let next = challenges::next_by_games(games)?;
    Some(format!(
        "NEXT CHALLENGE: {} ({games} / {})",
        next.name, next.opens_at
    ))
}

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font = &fonts.main;
    let earned = |medal: Medal| save.medals.contains(&medal.id());
    let count = Medal::ALL.into_iter().filter(|&m| earned(m)).count();
    commands.spawn(label(font, "RECORDS", 54., NEON_WHITE, TITLE_Y));
    for (row, pair) in totals(&save).iter().enumerate() {
        let y = TOTALS_Y - row as f32 * TOTALS_GAP;
        // A row of one sits in the middle.
        let xs = match pair[1].is_empty() {
            true => [0., 0.],
            false => [-COLUMN_X, COLUMN_X],
        };
        for (text, x) in pair.iter().zip(xs) {
            commands.spawn(label_at(font, text, 20., NEON_WHITE, Vec2::new(x, y)));
        }
    }
    if let Some(next) = next_unlock(&save) {
        commands.spawn(label(font, &next, 18., NEON_YELLOW, NEXT_Y));
    }
    if let Some(next) = next_challenge(&save) {
        commands.spawn(label(font, &next, 18., NEON_YELLOW, NEXT_CHALLENGE_Y));
    }
    let count = format!("MEDALS {count} / {}", Medal::ALL.len());
    commands.spawn(label(font, &count, 22., NEON_CYAN, COUNT_Y));
    for (i, medal) in Medal::ALL.into_iter().enumerate() {
        let y = FIRST_Y - i as f32 * MEDAL_GAP;
        let (name, rule) = match earned(medal) {
            true => (NEON_YELLOW, NEON_WHITE),
            false => (NEON_GRAY, BASE_GRAY),
        };
        commands.spawn(label(font, medal.name(), 20., name, y));
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
        let totals_end = TOTALS_Y - 3. * TOTALS_GAP;
        assert!(totals_end > NEXT_Y + 20. && NEXT_CHALLENGE_Y > COUNT_Y + 20.);
    }

    #[test]
    fn the_totals_and_the_next_unlock_read_from_the_save() {
        let mut save = Save {
            best: 12_345,
            best_level: 9,
            ..default()
        };
        save.stats.games = Some(18);
        save.stats.kills = 1_203;
        save.stats.seconds = 3. * 3600. + 12. * 60. + 5.;
        let rows = totals(&save);
        assert_eq!(rows[0], ["BEST 12,345", "BEST LEVEL 9"]);
        assert_eq!(rows[2][0], "KILLS 1,203");
        assert_eq!(rows[3][0], "TIME 3H 12M");
        assert_eq!(play_time(65.4), "1M 05S");
        assert_eq!(
            next_unlock(&save).as_deref(),
            Some("NEXT: SHOCKWAVE KILL (18 / 25 GAMES)")
        );
        assert_eq!(
            next_challenge(&save).as_deref(),
            Some("NEXT CHALLENGE: BULLET HELL (18 / 20)")
        );
        save.stats.games = Some(1000);
        assert_eq!(next_unlock(&save), None, "all open");
        assert_eq!(next_challenge(&save), None);
    }
}
