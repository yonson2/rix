//! The game over screen ([038]): world `Text2d` and neon buttons, so it gets the bloom and
//! the CRT pass as the title does.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        challenges,
        countdown::Countdown,
        death::DeathPicture,
        secrets::{self, Secret},
        seed,
        setup::{Points, Run, new_run, random_seed, retry_seed},
        skins,
    },
    save::{RunStats, Save, record_daily, record_run},
    share::{SHEET, Share},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, BURGUNDY, NEON_CYAN, NEON_WHITE, NEON_YELLOW},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), setup);
}

/// The RETRY button's height; the flow scene taps it ([026]).
pub const RETRY_Y: f32 = -90.;
const MENU_Y: f32 = -200.;
const SHARE_Y: f32 = -310.;
/// The best score, or "NEW BEST!" ([048]).
const BEST_Y: f32 = -12.;
/// A skin or a challenge that this game unlocked ([114], [123]), or the palindrome secret
/// ([119]).
const NOTICE_Y: f32 = 330.;
/// The challenge's name, over the title ([123]).
const NAME_Y: f32 = 272.;

/// It adds the run to the save ([048]).
fn setup(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    run: Res<Run>,
    played: Res<RunStats>,
    mut save: ResMut<Save>,
    clock: Option<Res<Countdown>>,
) {
    let games = save.stats.games();
    // Before `record_run`: the first run finds nothing (D27).
    let found = secrets::palindrome(run.points.0) && secrets::find(&mut save, Secret::Palindrome);
    let new_best = record_run(&mut save, &run, &played);
    record_daily(&mut save, run.seed, challenges::today());
    let unlocked = skins::unlocked_by_games(games, save.stats.games());
    let notices: Vec<String> = [
        skins::skin_notice(&unlocked),
        challenges::notice(games, save.stats.games()),
        found.then(|| secrets::notice(Secret::Palindrome)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let text = |text: String, size: f32, color: Color, y: f32| {
        (
            DespawnOnExit(Screen::GameOver),
            Text2d::new(text),
            TextFont {
                font: fonts.main.clone().into(),
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(0., y, 1.),
        )
    };
    if !notices.is_empty() {
        commands.spawn((
            text(notices.join("\n"), 22., NEON_YELLOW, NOTICE_Y),
            TextLayout::justify(Justify::Center),
        ));
    }
    if let Some(name) = challenges::name(run.seed) {
        commands.spawn(text(name, 24., NEON_CYAN, NAME_Y));
    }
    let (title, color) = title(clock.as_deref());
    commands.spawn(text(title.into(), 64., color, 220.));
    commands.spawn(text(run.points.to_string(), 54., NEON_WHITE, 130.));
    let stats = format!("LEVEL {}   KILLS {}", run.level, run.kill_count);
    commands.spawn(text(stats, 24., Color::WHITE, 70.));
    commands.spawn(text(
        format!("SEED {}", seed::code(run.seed)),
        22.,
        BASE_GRAY,
        32.,
    ));
    commands.spawn(if new_best {
        text("NEW BEST!".into(), 26., NEON_YELLOW, BEST_Y)
    } else {
        text(
            format!("BEST {}", Points(save.best_of(run.seed))),
            22.,
            BASE_GRAY,
            BEST_Y,
        )
    });

    commands
        .spawn((
            DespawnOnExit(Screen::GameOver),
            widget::button("RETRY", fonts.main.clone(), RETRY_Y),
        ))
        .observe(retry);
    commands
        .spawn((
            DespawnOnExit(Screen::GameOver),
            widget::button("MENU", fonts.main.clone(), MENU_Y),
        ))
        .observe(menu);
    // Off phones a share is a copy ([067]).
    let label = if SHEET { "SHARE" } else { "COPY" };
    commands
        .spawn((
            DespawnOnExit(Screen::GameOver),
            widget::button(label, fonts.main.clone(), SHARE_Y),
        ))
        .observe(share);
}

/// "TIME!" when a countdown challenge ended on its clock ([122]), else "YOU DIED!".
fn title(clock: Option<&Countdown>) -> (&'static str, Color) {
    match clock {
        Some(clock) if clock.is_up() => ("TIME!", NEON_YELLOW),
        _ => ("YOU DIED!", BURGUNDY),
    }
}

/// The score, the level (or the challenge, [123]) and the seed, so a friend can play the
/// same run ([095]).
fn share_text(run: &Run) -> String {
    let on = match challenges::name(run.seed) {
        Some(name) => name,
        None => format!("level {}", run.level),
    };
    format!(
        "I scored {} on {on} in Rix! Seed {}",
        run.points,
        seed::code(run.seed)
    )
}

/// On phones the picture of the death goes with the text ([095]).
fn share(
    activate: On<Activate>,
    mut commands: Commands,
    run: Res<Run>,
    picture: Option<Res<DeathPicture>>,
    children: Query<&Children>,
    mut labels: Query<&mut Text2d>,
) {
    commands.trigger(Share {
        text: share_text(&run),
        picture: picture.map(|picture| picture.0.clone()),
    });
    if SHEET {
        return;
    }
    for child in children.iter_descendants(activate.entity) {
        if let Ok(mut label) = labels.get_mut(child) {
            label.0 = "COPIED".into();
        }
    }
}

/// A challenge plays again ([123]).
fn retry(
    _: On<Activate>,
    mut commands: Commands,
    run: Res<Run>,
    mut next: ResMut<NextState<Screen>>,
) {
    new_run(&mut commands, retry_seed(run.seed));
    next.set(Screen::Gameplay);
}

/// The title holds a fresh run, so PLAY starts it.
fn menu(_: On<Activate>, mut commands: Commands, mut next: ResMut<NextState<Screen>>) {
    new_run(&mut commands, random_seed());
    next.set(Screen::Title);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_share_tells_the_score_the_level_and_the_seed() {
        let mut run = Run::with_seed(7);
        run.points = Points(12_345);
        run.level = 4;
        assert_eq!(
            share_text(&run),
            format!("I scored 12,345 on level 4 in Rix! Seed {}", seed::code(7))
        );
        let ball_pit = &challenges::CHALLENGES[1];
        run.seed = ball_pit.seed();
        assert_eq!(
            share_text(&run),
            format!(
                "I scored 12,345 on BALL PIT in Rix! Seed {}",
                seed::code(ball_pit.seed())
            )
        );
        assert_eq!(
            seed::find(&share_text(&run)),
            Some(seed::code(run.seed).replace('-', ""))
        );
    }

    #[test]
    fn the_title_says_time_only_when_the_clock_ran_out() {
        assert_eq!(title(None).0, "YOU DIED!");
        let mut clock = Countdown::new(60.);
        assert_eq!(title(Some(&clock)).0, "YOU DIED!", "a death with time left");
        assert!(clock.tick(std::time::Duration::from_secs(60)));
        assert_eq!(title(Some(&clock)).0, "TIME!");
    }

    #[test]
    fn a_run_that_ends_on_a_palindrome_finds_its_secret() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(FontAssets {
            main: Handle::default(),
        });
        world.init_resource::<RunStats>();
        let mut save = Save::default();
        save.stats.runs = 1;
        world.insert_resource(save);
        let mut run = Run::with_seed(1);
        run.points = Points(12_321);
        world.insert_resource(run);
        world.run_system_once(setup).unwrap();
        assert_eq!(world.resource::<Save>().secrets, ["Palindrome"]);
        let notice = secrets::notice(Secret::Palindrome);
        assert!(world.query::<&Text2d>().iter(&world).any(|t| t.0 == notice));
    }
}
