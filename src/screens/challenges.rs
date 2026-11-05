//! The CHALLENGES screen ([123]): the daily challenge ([124]) with its days in a row, then
//! one row per listed challenge, each with its name, its rule and the local best. A tap on
//! an open row plays it; a locked row says when it opens.

use bevy::{prelude::*, sprite::Anchor};

use super::Screen;
use crate::{
    game::{
        challenges::{self, CHALLENGES, Challenge},
        setup::{Points, new_run},
    },
    save::Save,
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, NEON_GRAY, NEON_WHITE, NEON_YELLOW},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Challenges), setup);
}

const TITLE_Y: f32 = 340.;
const FIRST_Y: f32 = 284.;
const ROW_GAP: f32 = 56.;
/// 52 high: about 45 pt on a 390 pt wide phone, over the 44 pt minimum.
const ROW_SIZE: Vec2 = Vec2::new(400., 52.);
/// The texts keep this far inside a row's sides.
const TEXT_X: f32 = 188.;
const NAME_DY: f32 = 9.;
const ABOUT_DY: f32 = -12.;
const BACK_Y: f32 = -345.;

/// The seed that an open row plays.
#[derive(Component)]
struct Row(u64);

fn label(font: &Handle<Font>, text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text2d::new(text),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

/// The row's second line: the rule, or the games that open it.
fn about(challenge: &Challenge, games: u64) -> String {
    match games >= challenge.opens_at {
        true => challenge.about.into(),
        false => format!(
            "OPENS AT {at} GAMES ({games} / {at})",
            at = challenge.opens_at
        ),
    }
}

/// What a row shows and plays.
struct Entry {
    name: String,
    about: String,
    seed: u64,
    open: bool,
    /// The days in a row: the daily only.
    streak: u64,
}

/// The daily challenge of `today`, open from the start, then the list.
fn entries(save: &Save, today: u64) -> Vec<Entry> {
    let games = save.stats.games();
    let daily = challenges::daily(today);
    let daily = Entry {
        name: format!("DAILY {}", daily.name),
        about: daily.about.into(),
        seed: challenges::daily_seed(today),
        open: true,
        streak: save.daily.streak_on(today),
    };
    let listed = CHALLENGES.iter().map(|c| Entry {
        name: c.name.into(),
        about: about(c, games),
        seed: c.seed(),
        open: games >= c.opens_at,
        streak: 0,
    });
    std::iter::once(daily).chain(listed).collect()
}

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font = &fonts.main;
    commands.spawn((
        DespawnOnExit(Screen::Challenges),
        label(font, "CHALLENGES", 54., NEON_WHITE),
        Transform::from_xyz(0., TITLE_Y, 1.),
    ));
    for (i, entry) in entries(&save, challenges::today()).into_iter().enumerate() {
        let at = Vec2::new(0., FIRST_Y - i as f32 * ROW_GAP);
        let mut row = match entry.open {
            true => {
                let mut row = commands.spawn((
                    DespawnOnExit(Screen::Challenges),
                    Row(entry.seed),
                    widget::button_at("", font.clone(), at, ROW_SIZE),
                ));
                row.observe(play);
                row
            }
            false => commands.spawn((
                DespawnOnExit(Screen::Challenges),
                Shape::new(shape::rounded_rectangle(ROW_SIZE, 12.))
                    .fill(BASE_GRAY.with_alpha(0.12))
                    .stroke(NEON_GRAY, 3.),
                Transform::from_translation(at.extend(1.)),
            )),
        };
        let (name, text) = match entry.open {
            true => (NEON_YELLOW, NEON_WHITE),
            false => (NEON_GRAY, BASE_GRAY),
        };
        let best = save.best_of(entry.seed);
        row.with_children(|row| {
            row.spawn((
                label(font, entry.name, 20., name),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-TEXT_X, NAME_DY, 0.1),
            ));
            row.spawn((
                label(font, entry.about, 15., text),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-TEXT_X, ABOUT_DY, 0.1),
            ));
            if best > 0 {
                row.spawn((
                    label(font, format!("BEST {}", Points(best)), 16., text),
                    Anchor::CENTER_RIGHT,
                    Transform::from_xyz(TEXT_X, NAME_DY, 0.1),
                ));
            }
            if entry.streak > 0 {
                row.spawn((
                    label(font, format!("STREAK {}", entry.streak), 15., name),
                    Anchor::CENTER_RIGHT,
                    Transform::from_xyz(TEXT_X, ABOUT_DY, 0.1),
                ));
            }
        });
    }
    commands
        .spawn((
            DespawnOnExit(Screen::Challenges),
            widget::button("BACK", font.clone(), BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

fn play(
    activate: On<Activate>,
    rows: Query<&Row>,
    mut commands: Commands,
    mut next: ResMut<NextState<Screen>>,
) {
    if let Ok(row) = rows.get(activate.entity) {
        new_run(&mut commands, row.0);
        next.set(Screen::Gameplay);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{rules::Rules, setup::Run};
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn the_rows_fit_above_the_back_button() {
        // The daily and the list; the BACK button is 76 high.
        let last = FIRST_Y - CHALLENGES.len() as f32 * ROW_GAP - ROW_SIZE.y / 2.;
        assert!(last > BACK_Y + 38. + 3., "the last row ends at {last}");
    }

    #[test]
    fn a_locked_row_says_when_it_opens() {
        let swarm = &CHALLENGES[4];
        assert_eq!(about(swarm, 12), "OPENS AT 15 GAMES (12 / 15)");
        assert_eq!(about(swarm, 15), "3X ENEMIES");
    }

    #[test]
    fn the_daily_comes_first_open_with_its_days_in_a_row() {
        let day = 20_397;
        let mut save = Save::default();
        save.daily.day = day - 1;
        save.daily.streak = 3;
        let rows = entries(&save, day);
        assert_eq!(rows.len(), CHALLENGES.len() + 1);
        let daily = &rows[0];
        assert_eq!(daily.seed, challenges::daily_seed(day));
        assert_eq!(daily.name, format!("DAILY {}", challenges::daily(day).name));
        assert!(daily.open, "also when its rule set is locked");
        assert_eq!(daily.streak, 3);
        assert!(rows[1..].iter().all(|row| row.streak == 0));
        assert_eq!(entries(&save, day + 2)[0].streak, 0, "a day missed");
    }

    #[test]
    fn a_tap_on_an_open_row_plays_its_challenge() {
        let mut world = World::new();
        world.insert_resource(FontAssets {
            main: Handle::default(),
        });
        world.init_resource::<Save>();
        world.init_resource::<NextState<Screen>>();
        world.run_system_once(setup).unwrap();
        let rows: Vec<(Entity, u64)> = world
            .query::<(Entity, &Row)>()
            .iter(&world)
            .map(|(entity, row)| (entity, row.0))
            .collect();
        assert_eq!(rows.len(), 4, "a new player has the daily and three open");
        let ball_pit = CHALLENGES[1].seed();
        let (entity, _) = rows
            .into_iter()
            .find(|&(_, seed)| seed == ball_pit)
            .unwrap();
        world.trigger(Activate { entity });
        world.flush();
        assert_eq!(world.resource::<Run>().seed, ball_pit);
        assert_eq!(*world.resource::<Rules>(), CHALLENGES[1].rules);
        assert!(matches!(
            *world.resource::<NextState<Screen>>(),
            NextState::Pending(Screen::Gameplay)
        ));
    }
}
