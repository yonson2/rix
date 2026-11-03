//! The game over screen ([038]): world `Text2d` and neon buttons, so it gets the bloom and
//! the CRT pass as the title does.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::{
        death::DeathPicture,
        seed,
        setup::{Points, Run, new_run, random_seed},
    },
    save::{RunStats, Save, record_run},
    share::{SHEET, Share},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, BURGUNDY, NEON_WHITE, NEON_YELLOW},
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

/// It adds the run to the save ([048]).
fn setup(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    run: Res<Run>,
    played: Res<RunStats>,
    mut save: ResMut<Save>,
) {
    let new_best = record_run(&mut save, &run, &played);
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
    commands.spawn(text("YOU DIED!".into(), 64., BURGUNDY, 220.));
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
            format!("BEST {}", Points(save.best)),
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

/// The score, the level and the seed, so a friend can play the same run ([095]).
fn share_text(run: &Run) -> String {
    format!(
        "I scored {} on level {} in Rix! Seed {}",
        run.points,
        run.level,
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

fn retry(_: On<Activate>, mut commands: Commands, mut next: ResMut<NextState<Screen>>) {
    new_run(&mut commands, random_seed());
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
    }
}
