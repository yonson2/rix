//! The game over screen ([038]): world `Text2d` and neon buttons, so it gets the bloom and
//! the CRT pass as the title does.

use bevy::prelude::*;

use super::Screen;
use crate::{
    game::setup::{Run, new_run, random_seed},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, BURGUNDY, NEON_WHITE},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), setup);
}

/// The RETRY button's height; the flow scene taps it ([026]).
pub const RETRY_Y: f32 = -90.;
const MENU_Y: f32 = -200.;

fn setup(mut commands: Commands, fonts: Res<FontAssets>, run: Res<Run>) {
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
    commands.spawn(text(format!("SEED {}", run.seed), 18., BASE_GRAY, 32.));

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
