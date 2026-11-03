use bevy::prelude::*;

use crate::{
    common::Transition,
    game::setup::Points,
    save::Save,
    screens::Screen,
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{NEON_PURPLE, NEON_WHITE, PURPLE},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), setup);
    app.add_systems(
        Update,
        (flicker_logo, breathe_logo).run_if(in_state(Screen::Title)),
    );
}

const LOGO_Y: f32 = 200.;
/// The best score ([048]) sits between the logo and the buttons.
const BEST_Y: f32 = 70.;
const PLAY_Y: f32 = -10.;
const SEED_Y: f32 = -94.;
const SETTINGS_Y: f32 = -178.;
/// WIKI and SKINS share a row, each half a button wide.
const HALF_ROW_Y: f32 = -262.;
const HALF_BUTTON: Vec2 = Vec2::new(127., 76.);
#[cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))]
const QUIT_Y: f32 = -346.;

/// The "RIX" box; its text is a child.
#[derive(Component)]
struct Logo;

fn setup(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let font_size = 119.;

    let shape = Shape::new(shape::rounded_rectangle(
        Vec2::new(font_size * 2., font_size * 1.5),
        10.,
    ))
    .fill(PURPLE)
    .stroke(NEON_PURPLE, 3.0);

    commands.spawn((
        Logo,
        DespawnOnExit(Screen::Title),
        shape,
        Transform::from_xyz(0., LOGO_Y, 0.),
        children![(
            Text2d::new("RIX"),
            TextFont {
                font: fonts.main.clone().into(),
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(Color::NONE),
            Transform::from_xyz(0., 0., 1.),
            Transition(Timer::from_seconds(2.6, TimerMode::Once)),
        )],
    ));

    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::button("PLAY", fonts.main.clone(), PLAY_Y),
        ))
        .observe(play);
    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::button("PLAY SEED", fonts.main.clone(), SEED_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Seed);
        });
    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::button("SETTINGS", fonts.main.clone(), SETTINGS_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Settings);
        });
    for (label, x, screen) in [
        ("WIKI", -66.5, Screen::Wiki),
        ("SKINS", 66.5, Screen::Skins),
    ] {
        commands
            .spawn((
                DespawnOnExit(Screen::Title),
                widget::button_at(
                    label,
                    fonts.main.clone(),
                    Vec2::new(x, HALF_ROW_Y),
                    HALF_BUTTON,
                ),
            ))
            .observe(
                move |_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
                    next.set(screen);
                },
            );
    }
    if save.best > 0 {
        commands.spawn((
            DespawnOnExit(Screen::Title),
            Text2d::new(format!("BEST {}", Points(save.best))),
            TextFont {
                font: fonts.main.clone().into(),
                font_size: FontSize::Px(26.),
                ..default()
            },
            TextColor(NEON_WHITE),
            Transform::from_xyz(0., BEST_Y, 1.),
        ));
    }
    // Phones and the web close apps their own way.
    #[cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))]
    commands
        .spawn((
            DespawnOnExit(Screen::Title),
            widget::button("QUIT", fonts.main.clone(), QUIT_Y),
        ))
        .observe(quit);
}

fn play(_: On<Activate>, mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Gameplay);
}

#[cfg(not(any(target_os = "ios", target_os = "android", target_family = "wasm")))]
fn quit(_: On<Activate>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

/// The logo text and outline flicker in.
fn flicker_logo(
    mut commands: Commands,
    time: Res<Time>,
    mut text_q: Query<(Entity, &mut Transition, &mut TextColor)>,
    mut shape_q: Query<&mut Shape, With<Logo>>,
) {
    for (entity, mut transition, mut text_color) in text_q.iter_mut() {
        transition.0.tick(time.delta());
        let t = transition.0.fraction();
        let easing = EasingCurve::new(0.0, 1.0, EaseFunction::BounceInOut);
        let alpha = easing.sample(t).unwrap();
        let mut current_color = NEON_WHITE;
        current_color.set_alpha(alpha);

        text_color.0 = current_color;
        for mut shape in &mut shape_q {
            if let Some(stroke) = &mut shape.stroke {
                stroke.color = current_color;
            }
        }

        if transition.0.is_finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

/// After the flicker, the logo grows and shrinks a little, slowly.
fn breathe_logo(time: Res<Time>, mut logo: Query<&mut Transform, With<Logo>>) {
    let scale = 1. + 0.025 * ops::sin(time.elapsed_secs() * 1.6);
    for mut transform in &mut logo {
        transform.scale = Vec3::splat(scale);
    }
}
