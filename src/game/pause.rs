//! Pause ([034]): the app going to the background, the window losing focus or Escape
//! pauses a run; a tap, a click or Escape resumes it. Virtual and physics time stop
//! together, so nothing moves while paused.

use avian2d::prelude::*;
use bevy::{
    input::{ButtonState, mouse::MouseButtonInput, touch::TouchPhase},
    prelude::*,
    window::{AppLifecycle, WindowFocused},
};

use crate::theme::{FontAssets, palette::BASE_GRAY};

use super::{state::IsPaused, wall::FIELD};

/// Above the HUD.
const OVERLAY_Z: f32 = 20.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            pause_on_signal.run_if(in_state(IsPaused::Running)),
            resume_on_input.run_if(in_state(IsPaused::Paused)),
        ),
    )
    .add_systems(
        OnEnter(IsPaused::Paused),
        (stop_time, drop_pending_input, spawn_overlay),
    )
    .add_systems(OnExit(IsPaused::Paused), (start_time, drop_pending_input));
}

fn pause_on_signal(
    mut lifecycle: MessageReader<AppLifecycle>,
    mut focus: MessageReader<WindowFocused>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<IsPaused>>,
) {
    let suspended = lifecycle
        .read()
        .any(|e| matches!(e, AppLifecycle::WillSuspend | AppLifecycle::Suspended));
    let unfocused = focus.read().any(|e| !e.focused);
    if suspended || unfocused || keys.just_pressed(KeyCode::Escape) {
        next.set(IsPaused::Paused);
    }
}

/// Resumes on the release, so the press does not reach the line drawing.
fn resume_on_input(
    mut clicks: MessageReader<MouseButtonInput>,
    mut touches: MessageReader<TouchInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<IsPaused>>,
) {
    let clicked = clicks
        .read()
        .any(|e| e.button == MouseButton::Left && e.state == ButtonState::Released);
    let tapped = touches.read().any(|e| e.phase == TouchPhase::Ended);
    if clicked || tapped || keys.just_pressed(KeyCode::Escape) {
        next.set(IsPaused::Running);
    }
}

fn stop_time(mut time: ResMut<Time<Virtual>>, mut physics: ResMut<Time<Physics>>) {
    time.pause();
    physics.pause();
}

fn start_time(mut time: ResMut<Time<Virtual>>, mut physics: ResMut<Time<Physics>>) {
    time.unpause();
    physics.unpause();
}

/// Pointer messages from before the switch must not resume the pause or start a line.
fn drop_pending_input(
    mut clicks: ResMut<Messages<MouseButtonInput>>,
    mut touches: ResMut<Messages<TouchInput>>,
) {
    clicks.clear();
    touches.clear();
}

fn spawn_overlay(mut commands: Commands, fonts: Res<FontAssets>) {
    let font = fonts.main.clone();
    commands.spawn((
        DespawnOnExit(IsPaused::Paused),
        Sprite::from_color(Color::BLACK.with_alpha(0.6), FIELD),
        Transform::from_xyz(0., 0., OVERLAY_Z),
    ));
    commands.spawn((
        DespawnOnExit(IsPaused::Paused),
        Text2d::new("PAUSED"),
        TextFont {
            font: font.into(),
            font_size: FontSize::Px(70.),
            ..default()
        },
        Transform::from_xyz(0., 20., OVERLAY_Z + 1.),
    ));
    commands.spawn((
        DespawnOnExit(IsPaused::Paused),
        Text2d::new("Tap to resume."),
        TextFont {
            font_size: FontSize::Px(25.),
            ..default()
        },
        TextColor(BASE_GRAY),
        Transform::from_xyz(0., -40., OVERLAY_Z + 1.),
    ));
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{
        input::InputPlugin, state::app::StatesPlugin, time::TimeUpdateStrategy,
        transform::TransformPlugin,
    };

    use super::*;
    use crate::{game::state::Phase, screens::Screen};

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            InputPlugin,
            StatesPlugin,
            TransformPlugin,
            PhysicsPlugins::default(),
            plugin,
        ))
        .init_asset::<Font>()
        .add_message::<AppLifecycle>()
        .add_message::<WindowFocused>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )))
        .init_state::<Screen>()
        .add_sub_state::<Phase>()
        .add_sub_state::<IsPaused>()
        .insert_state(Screen::Gameplay)
        .insert_resource(crate::theme::FontAssets {
            main: Handle::default(),
        });
        app.finish();
        app
    }

    fn x(app: &mut App, body: Entity) -> f32 {
        app.world().get::<Position>(body).unwrap().x
    }

    fn run(app: &mut App, frames: u32) {
        for _ in 0..frames {
            app.update();
        }
    }

    #[test]
    fn suspend_freezes_bodies_until_a_tap() {
        let mut app = app();
        let body = app
            .world_mut()
            .spawn((
                RigidBody::Dynamic,
                Collider::circle(5.),
                GravityScale(0.),
                LinearVelocity(Vec2::new(100., 0.)),
            ))
            .id();
        run(&mut app, 30);
        let moving = x(&mut app, body);
        assert!(moving > 0., "the body moves before the pause");

        app.world_mut().write_message(AppLifecycle::WillSuspend);
        run(&mut app, 2);
        assert_eq!(
            *app.world().resource::<State<IsPaused>>().get(),
            IsPaused::Paused
        );
        let paused_at = x(&mut app, body);
        run(&mut app, 120);
        assert_eq!(
            x(&mut app, body),
            paused_at,
            "nothing moves in 2 s of pause"
        );

        app.world_mut().write_message(TouchInput {
            phase: TouchPhase::Ended,
            position: Vec2::ZERO,
            window: Entity::PLACEHOLDER,
            force: None,
            id: 0,
        });
        run(&mut app, 2);
        assert_eq!(
            *app.world().resource::<State<IsPaused>>().get(),
            IsPaused::Running
        );
        run(&mut app, 30);
        assert!(
            x(&mut app, body) > paused_at,
            "the body moves after the tap"
        );
    }
}
