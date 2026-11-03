//! Pause ([034]): the app going to the background, the window losing focus, Escape,
//! Android Back ([079]) or the HUD's pause button ([073]) pauses a run. The pause menu
//! resumes, restarts, shows the settings or goes to the title; Escape resumes too, a tap
//! elsewhere does nothing. Virtual and physics time stop together, so nothing moves while
//! paused.

use avian2d::prelude::*;
use bevy::{
    input::mouse::MouseButtonInput,
    prelude::*,
    window::{AppLifecycle, WindowEvent, WindowFocused},
};

use crate::{
    save::Save,
    screens::{Screen, settings},
    theme::{
        FontAssets,
        palette::BASE_GRAY,
        widget::{self, Activate},
    },
};

use super::{
    seed,
    setup::{Run, new_run, random_seed},
    state::{IsPaused, PausePanel},
    wall::Field,
};

/// Above the HUD.
const OVERLAY_Z: f32 = 20.;
const PAUSED_Y: f32 = 210.;
const SEED_Y: f32 = 140.;
const FIRST_BUTTON_Y: f32 = 40.;
const BUTTON_GAP: f32 = 90.;

/// The pause menu's buttons ([073]).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
enum PauseButton {
    Resume,
    Restart,
    Settings,
    Menu,
}

const MENU: [(PauseButton, &str); 4] = [
    (PauseButton::Resume, "RESUME"),
    (PauseButton::Restart, "RESTART"),
    (PauseButton::Settings, "SETTINGS"),
    (PauseButton::Menu, "MENU"),
];

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            pause_on_signal.run_if(in_state(IsPaused::Running)),
            resume_on_key.run_if(in_state(IsPaused::Paused)),
        ),
    )
    .add_systems(
        OnEnter(IsPaused::Paused),
        (stop_time, drop_pending_input, spawn_overlay),
    )
    .add_systems(OnEnter(PausePanel::Menu), spawn_menu)
    .add_systems(OnEnter(PausePanel::Settings), spawn_settings)
    .add_systems(
        OnEnter(Screen::Restart),
        |mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Gameplay);
        },
    )
    .add_systems(OnExit(IsPaused::Paused), (start_time, drop_pending_input));
    #[cfg(target_os = "android")]
    app.add_plugins(back::plugin);
}

/// Android Back ([079]): `MainActivity` asks the game first. Back pauses a running game;
/// paused or on another screen, the activity moves to the background.
#[cfg(any(target_os = "android", test))]
mod back {
    use std::{
        ffi::c_void,
        sync::atomic::{AtomicU8, Ordering::SeqCst},
    };

    use bevy::prelude::*;

    use crate::game::state::IsPaused;

    const AWAY: u8 = 0;
    const RUNNING: u8 = 1;
    const PRESSED: u8 = 2;

    /// The Java UI thread and the game thread both write it.
    static STATE: AtomicU8 = AtomicU8::new(AWAY);

    pub(super) fn plugin(app: &mut App) {
        app.add_systems(OnEnter(IsPaused::Running), || STATE.store(RUNNING, SeqCst))
            .add_systems(OnExit(IsPaused::Running), || STATE.store(AWAY, SeqCst))
            .add_systems(Update, pause.run_if(in_state(IsPaused::Running)));
    }

    fn pause(mut next: ResMut<NextState<IsPaused>>) {
        if STATE.load(SeqCst) == PRESSED {
            next.set(IsPaused::Paused);
        }
    }

    /// `MainActivity.onBackInPlay` (JNI, on the Java UI thread): 1 when the game takes Back.
    #[unsafe(export_name = "Java_top_cosas_rix_MainActivity_onBackInPlay")]
    pub(super) extern "C" fn on_back_in_play(_env: *mut c_void, _class: *mut c_void) -> u8 {
        let taken = STATE.compare_exchange(RUNNING, PRESSED, SeqCst, SeqCst);
        matches!(taken, Ok(_) | Err(PRESSED)).into()
    }
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

fn resume_on_key(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<IsPaused>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(IsPaused::Running);
    }
}

pub(super) fn stop_time(mut time: ResMut<Time<Virtual>>, mut physics: ResMut<Time<Physics>>) {
    time.pause();
    physics.pause();
}

pub(super) fn start_time(mut time: ResMut<Time<Virtual>>, mut physics: ResMut<Time<Physics>>) {
    time.unpause();
    physics.unpause();
}

/// Pointer messages from before the switch must not resume the pause or start a line.
fn drop_pending_input(
    mut clicks: ResMut<Messages<MouseButtonInput>>,
    mut touches: ResMut<Messages<TouchInput>>,
    mut window_events: ResMut<Messages<WindowEvent>>,
) {
    clicks.clear();
    touches.clear();
    window_events.clear();
}

fn spawn_overlay(mut commands: Commands, field: Res<Field>) {
    commands.spawn((
        DespawnOnExit(IsPaused::Paused),
        Sprite::from_color(Color::BLACK.with_alpha(0.85), field.0),
        Transform::from_xyz(0., 0., OVERLAY_Z),
    ));
}

/// A panel's root, above the dim sprite; the panel is its children.
fn panel_root(scope: PausePanel) -> impl Bundle {
    (
        DespawnOnExit(scope),
        Transform::from_xyz(0., 0., OVERLAY_Z + 1.),
        Visibility::default(),
    )
}

fn spawn_menu(mut commands: Commands, fonts: Res<FontAssets>, run: Res<Run>) {
    let font = fonts.main.clone();
    let text = |text: String, size: f32, color: Color, y: f32| {
        (
            Text2d::new(text),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(0., y, 0.),
        )
    };
    commands
        .spawn(panel_root(PausePanel::Menu))
        .with_children(|menu| {
            menu.spawn(text("PAUSED".into(), 70., Color::WHITE, PAUSED_Y));
            // The seed of this run ([067]), to note it down or play it again.
            let code = format!("SEED {}", seed::code(run.seed));
            menu.spawn(text(code, 22., BASE_GRAY, SEED_Y));
            for (row, (button, label)) in MENU.into_iter().enumerate() {
                let y = FIRST_BUTTON_Y - row as f32 * BUTTON_GAP;
                menu.spawn((button, widget::button(label, font.clone(), y)))
                    .observe(press);
            }
        });
}

/// RESTART and MENU start a fresh run, as the game over buttons do; the left run is not
/// recorded.
fn press(
    activate: On<Activate>,
    buttons: Query<&PauseButton>,
    mut commands: Commands,
    mut paused: ResMut<NextState<IsPaused>>,
    mut panel: ResMut<NextState<PausePanel>>,
    mut screen: ResMut<NextState<Screen>>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    match button {
        PauseButton::Resume => paused.set(IsPaused::Running),
        PauseButton::Restart => {
            new_run(&mut commands, random_seed());
            screen.set(Screen::Restart);
        }
        PauseButton::Settings => panel.set(PausePanel::Settings),
        PauseButton::Menu => {
            new_run(&mut commands, random_seed());
            screen.set(Screen::Title);
        }
    }
}

fn spawn_settings(mut commands: Commands, fonts: Res<FontAssets>, save: Res<Save>) {
    let root = panel_root(PausePanel::Settings);
    let back = settings::spawn_panel(&mut commands, &fonts.main, &save, root);
    commands.entity(back).observe(
        |_: On<Activate>, mut panel: ResMut<NextState<PausePanel>>| {
            panel.set(PausePanel::Menu);
        },
    );
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{
        input::{InputPlugin, touch::TouchPhase},
        state::app::StatesPlugin,
        time::TimeUpdateStrategy,
        transform::TransformPlugin,
    };

    use super::*;
    use crate::{game::state::Phase, theme::widget::NeonButton};

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
        .add_message::<WindowEvent>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1. / 60.,
        )))
        .init_state::<Screen>()
        .add_sub_state::<Phase>()
        .add_sub_state::<IsPaused>()
        .add_sub_state::<PausePanel>()
        .insert_state(Screen::Gameplay)
        .init_resource::<Field>()
        .init_resource::<Save>()
        .insert_resource(Run::with_seed(1))
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

    /// Presses the button labelled `label`, then runs two frames.
    fn tap(app: &mut App, label: &str) {
        let world = app.world_mut();
        let mut buttons = world.query_filtered::<(Entity, &Children), With<NeonButton>>();
        let world_ref: &World = world;
        let entity = buttons
            .iter(world_ref)
            .find(|(_, children)| {
                children
                    .iter()
                    .any(|c| world_ref.get::<Text2d>(c).is_some_and(|t| t.0 == label))
            })
            .map(|(entity, _)| entity)
            .unwrap_or_else(|| panic!("no {label} button"));
        world.trigger(Activate { entity });
        run(app, 2);
    }

    fn pause(app: &mut App) {
        let mut next = app.world_mut().resource_mut::<NextState<IsPaused>>();
        next.set(IsPaused::Paused);
        run(app, 2);
    }

    fn paused(app: &App) -> Option<IsPaused> {
        app.world()
            .get_resource::<State<IsPaused>>()
            .map(|s| *s.get())
    }

    fn screen(app: &App) -> Screen {
        *app.world().resource::<State<Screen>>().get()
    }

    fn time_runs(app: &App) -> bool {
        !app.world().resource::<Time<Virtual>>().is_paused()
            && !app.world().resource::<Time<Physics>>().is_paused()
    }

    #[test]
    fn suspend_freezes_bodies_until_resume() {
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
            paused(&app),
            Some(IsPaused::Paused),
            "a tap elsewhere ([073])"
        );
        tap(&mut app, "RESUME");
        assert_eq!(paused(&app), Some(IsPaused::Running));
        run(&mut app, 30);
        assert!(x(&mut app, body) > paused_at, "the body moves after RESUME");
    }

    #[test]
    fn the_menu_shows_settings_restarts_and_quits_with_time_running() {
        let mut app = app();
        run(&mut app, 2);
        app.world_mut().resource_mut::<Run>().level = 3;

        pause(&mut app);
        assert!(!time_runs(&app));
        tap(&mut app, "SETTINGS");
        assert_eq!(
            *app.world().resource::<State<PausePanel>>().get(),
            PausePanel::Settings
        );
        tap(&mut app, "SOUND 100%");
        assert_eq!(app.world().resource::<Save>().settings.sound, 80);
        tap(&mut app, "BACK");
        assert_eq!(
            *app.world().resource::<State<PausePanel>>().get(),
            PausePanel::Menu
        );
        assert_eq!(paused(&app), Some(IsPaused::Paused), "the run stays");

        tap(&mut app, "RESTART");
        run(&mut app, 1);
        assert_eq!(screen(&app), Screen::Gameplay);
        assert_eq!(paused(&app), Some(IsPaused::Running));
        assert_eq!(app.world().resource::<Run>().level, 1, "a new run");
        assert!(time_runs(&app));

        pause(&mut app);
        tap(&mut app, "MENU");
        assert_eq!(screen(&app), Screen::Title);
        assert_eq!(paused(&app), None);
        assert!(time_runs(&app));
    }

    #[test]
    fn back_pauses_a_running_game_and_leaves_elsewhere() {
        let back = || back::on_back_in_play(std::ptr::null_mut(), std::ptr::null_mut()) == 1;
        let paused = |app: &App| *app.world().resource::<State<IsPaused>>().get();
        let mut app = app();
        back::plugin(&mut app);
        run(&mut app, 2);

        assert!(back(), "a running game takes Back");
        run(&mut app, 2);
        assert_eq!(paused(&app), IsPaused::Paused);
        assert!(!back(), "paused, Back moves the app to the background");

        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Title);
        run(&mut app, 2);
        assert!(
            !back(),
            "on the title, Back moves the app to the background"
        );
    }
}
