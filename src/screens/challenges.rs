//! The CHALLENGES screen ([123]): one row per listed challenge with its name, its rule and
//! the local best. A tap on an open row plays it; a locked row says when it opens.

use bevy::{prelude::*, sprite::Anchor};

use super::Screen;
use crate::{
    game::{
        challenges::{CHALLENGES, Challenge},
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

const TITLE_Y: f32 = 330.;
const FIRST_Y: f32 = 272.;
const ROW_GAP: f32 = 58.;
/// 52 high: about 45 pt on a 390 pt wide phone, over the 44 pt minimum.
const ROW_SIZE: Vec2 = Vec2::new(400., 52.);
/// The texts keep this far inside a row's sides.
const TEXT_X: f32 = 188.;
const NAME_DY: f32 = 9.;
const ABOUT_DY: f32 = -12.;
const BACK_Y: f32 = -340.;

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

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font = &fonts.main;
    commands.spawn((
        DespawnOnExit(Screen::Challenges),
        label(font, "CHALLENGES", 54., NEON_WHITE),
        Transform::from_xyz(0., TITLE_Y, 1.),
    ));
    let games = save.stats.games();
    for (i, challenge) in CHALLENGES.iter().enumerate() {
        let at = Vec2::new(0., FIRST_Y - i as f32 * ROW_GAP);
        let open = games >= challenge.opens_at;
        let mut row = match open {
            true => {
                let mut row = commands.spawn((
                    DespawnOnExit(Screen::Challenges),
                    Row(challenge.seed()),
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
        let (name, text) = match open {
            true => (NEON_YELLOW, NEON_WHITE),
            false => (NEON_GRAY, BASE_GRAY),
        };
        let best = save.best_of(challenge.seed());
        row.with_children(|row| {
            row.spawn((
                label(font, challenge.name, 20., name),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-TEXT_X, NAME_DY, 0.1),
            ));
            row.spawn((
                label(font, about(challenge, games), 15., text),
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
        let last = FIRST_Y - (CHALLENGES.len() - 1) as f32 * ROW_GAP - ROW_SIZE.y / 2.;
        assert!(last > BACK_Y + 50., "the last row ends at {last}");
    }

    #[test]
    fn a_locked_row_says_when_it_opens() {
        let swarm = &CHALLENGES[4];
        assert_eq!(about(swarm, 12), "OPENS AT 15 GAMES (12 / 15)");
        assert_eq!(about(swarm, 15), "3X ENEMIES");
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
        assert_eq!(rows.len(), 3, "a new player has three open");
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
