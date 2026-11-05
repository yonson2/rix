//! Countdown challenges ([122], D39): the most points before the clock runs out. The run
//! clock counts down in `FixedUpdate` while the game runs: a pause, a level transition and
//! a death stop it, the Time Stop does not. At 0 the run ends as a death does, with a buzzer
//! and a "TIME!" flash, and the game over screen says "TIME!". A death ends the run as in
//! the normal game ([109] Q2).

use std::time::Duration;

use bevy::prelude::*;

use super::{
    AppSystems, PausableSystems,
    state::{IsPaused, Phase},
    wall::Field,
};
use crate::{
    audio::Sfx,
    screens::Screen,
    theme::{
        FontAssets,
        palette::{NEON_BURGUNDY, NEON_WHITE},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_clock
            .run_if(resource_exists::<Countdown>)
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    )
    .add_systems(
        OnEnter(Phase::Playing),
        spawn_clock.run_if(resource_exists::<Countdown>),
    )
    .add_systems(
        Update,
        show_clock
            .run_if(resource_exists::<Countdown>)
            .run_if(in_state(Phase::Playing)),
    )
    .add_systems(OnEnter(IsPaused::Dying), spawn_time_up.run_if(time_is_up))
    .add_systems(Update, flash_time_up.run_if(in_state(IsPaused::Dying)));
}

/// The run clock of a countdown challenge; `new_run` sets it from the rules.
#[derive(Resource, Debug)]
pub struct Countdown(Timer);

impl Countdown {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }

    /// Runs the clock; true the moment it reaches 0.
    pub fn tick(&mut self, delta: Duration) -> bool {
        self.0.tick(delta).just_finished()
    }

    /// The run ended on the clock.
    pub fn is_up(&self) -> bool {
        self.0.is_finished()
    }
}

/// The run ended on the clock.
pub fn time_is_up(clock: Option<Res<Countdown>>) -> bool {
    clock.is_some_and(|clock| clock.is_up())
}

/// The last seconds: the clock grows, beats and turns red, with a tick each second.
const FINAL_SECS: u32 = 10;
const FINAL_SCALE: f32 = 1.5;
const BEAT_SCALE: f32 = 0.35;
/// Under the top wall, clear of the tips ([070]).
const CLOCK_Y: f32 = 40.;
const CLOCK_SIZE: f32 = 40.;
const HUD_Z: f32 = 10.;
const TIME_UP_SIZE: f32 = 96.;
/// The "TIME!" text pops from this scale and flashes as a hit line does ([038]).
const POP_SCALE: f32 = 1.6;
const POP_SECS: f32 = 0.3;
const FLASH_SECS: f32 = 0.1;

/// At 0 the run ends: the death's freeze, then the game over screen.
fn tick_clock(
    time: Res<Time>,
    mut clock: ResMut<Countdown>,
    mut next: ResMut<NextState<IsPaused>>,
) {
    if clock.tick(time.delta()) {
        next.set(IsPaused::Dying);
    }
}

/// The clock's text: minutes and seconds, the seconds rounded up.
fn clock_text(left: f32) -> String {
    let secs = left.ceil() as u32;
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// The clock's scale and color at `left` seconds: plain until the last `FINAL_SECS`, then
/// bigger and red, with a beat that swells as each second starts and fades.
fn clock_look(left: f32) -> (f32, Color) {
    if left > FINAL_SECS as f32 {
        return (1., NEON_WHITE);
    }
    let beat = left.fract() * left.fract() * left.fract();
    (FINAL_SCALE + BEAT_SCALE * beat, NEON_BURGUNDY)
}

#[derive(Component)]
struct UiClock;

fn spawn_clock(
    mut commands: Commands,
    clock: Res<Countdown>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
) {
    let y = field.inner_corners().top_left.y - CLOCK_Y;
    commands.spawn((
        UiClock,
        DespawnOnExit(Phase::Playing),
        Text2d::new(clock_text(clock.0.remaining_secs())),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(CLOCK_SIZE),
            ..default()
        },
        TextColor(NEON_WHITE),
        Transform::from_xyz(0., y, HUD_Z),
    ));
}

/// Draws only, and ticks in the last seconds.
fn show_clock(
    mut commands: Commands,
    clock: Res<Countdown>,
    mut texts: Query<(&mut Text2d, &mut TextColor, &mut Transform), With<UiClock>>,
    mut shown: Local<u32>,
) {
    let left = clock.0.remaining_secs();
    let secs = left.ceil() as u32;
    if secs < *shown && (1..=FINAL_SECS).contains(&secs) {
        commands.trigger(Sfx::Tick);
    }
    *shown = secs;
    let (scale, color) = clock_look(left);
    for (mut text, mut text_color, mut transform) in &mut texts {
        text.0 = clock_text(left);
        text_color.0 = color;
        transform.scale = Vec3::splat(scale);
    }
}

/// The "TIME!" text, and how long it has shown (real time: the game stands still).
#[derive(Component)]
struct TimeUp(f32);

fn spawn_time_up(mut commands: Commands, fonts: Res<FontAssets>) {
    commands.spawn((
        TimeUp(0.),
        DespawnOnExit(Screen::Gameplay),
        Text2d::new("TIME!"),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(TIME_UP_SIZE),
            ..default()
        },
        TextColor(NEON_WHITE),
        Transform::from_xyz(0., 0., HUD_Z).with_scale(Vec3::splat(POP_SCALE)),
    ));
}

/// Draws only.
fn flash_time_up(
    time: Res<Time<Real>>,
    mut texts: Query<(&mut TimeUp, &mut TextColor, &mut Transform)>,
) {
    for (mut time_up, mut color, mut transform) in &mut texts {
        time_up.0 += time.delta_secs();
        let pop = (1. - time_up.0 / POP_SECS).max(0.);
        transform.scale = Vec3::splat(1. + (POP_SCALE - 1.) * pop);
        let white = ((time_up.0 / FLASH_SECS) as u32).is_multiple_of(2);
        color.0 = if white { NEON_WHITE } else { NEON_BURGUNDY };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_shows_minutes_and_seconds_rounded_up() {
        assert_eq!(clock_text(60.), "1:00");
        assert_eq!(clock_text(59.01), "1:00");
        assert_eq!(clock_text(59.), "0:59");
        assert_eq!(clock_text(300.), "5:00");
        assert_eq!(clock_text(9.2), "0:10");
        assert_eq!(clock_text(0.), "0:00");
    }

    #[test]
    fn the_last_seconds_grow_beat_and_turn_red() {
        assert_eq!(clock_look(10.5), (1., NEON_WHITE));
        let (start, color) = clock_look(9.99);
        let (end, _) = clock_look(9.01);
        assert_eq!(color, NEON_BURGUNDY);
        assert!(
            start > end && end > FINAL_SCALE - 1e-3,
            "a beat: {start} then {end}"
        );
        assert!(clock_look(4.99).0 > clock_look(4.5).0);
    }

    #[test]
    fn the_clock_ends_the_run_at_0_only() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Time>();
        world.init_resource::<NextState<IsPaused>>();
        world.insert_resource(Countdown::new(60.));
        let tick = |world: &mut World, secs: u64| {
            world
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs(secs));
            world.run_system_once(tick_clock).unwrap();
            matches!(
                *world.resource::<NextState<IsPaused>>(),
                NextState::Pending(IsPaused::Dying)
            )
        };
        assert!(!tick(&mut world, 59));
        assert!(!world.run_system_once(time_is_up).unwrap());
        assert!(tick(&mut world, 1), "at 0 the run ends");
        assert!(world.run_system_once(time_is_up).unwrap());
    }

    #[test]
    fn a_death_is_not_time_up() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        assert!(!world.run_system_once(time_is_up).unwrap(), "no clock");
        world.insert_resource(Countdown::new(60.));
        assert!(!world.run_system_once(time_is_up).unwrap(), "time left");
    }
}
