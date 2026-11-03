//! The point multiplier ([046]): a gem. For a while every point counts twice; a new gem
//! starts the time over.

use bevy::prelude::*;

use crate::{
    game::{AppSystems, PausableSystems, state::Phase},
    shape::{self, Shape},
    theme::palette::{NEON_YELLOW, YELLOW},
};

pub const BOOST_SECS: f32 = 10.;
pub const BOOST_FACTOR: u128 = 2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_boost
            .run_if(resource_exists::<ScoreBoost>)
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
    app.add_systems(OnExit(Phase::Playing), |mut commands: Commands| {
        commands.remove_resource::<ScoreBoost>();
    });
}

/// While it exists, points count `BOOST_FACTOR` times.
#[derive(Resource, Deref, DerefMut)]
pub struct ScoreBoost(Timer);

impl Default for ScoreBoost {
    fn default() -> Self {
        Self(Timer::from_seconds(BOOST_SECS, TimerMode::Once))
    }
}

fn tick_boost(mut commands: Commands, time: Res<Time>, mut boost: ResMut<ScoreBoost>) {
    if boost.tick(time.delta()).is_finished() {
        commands.remove_resource::<ScoreBoost>();
    }
}

pub fn icon() -> Shape {
    let gem = [
        Vec2::new(-13., 4.),
        Vec2::new(-7., 11.),
        Vec2::new(7., 11.),
        Vec2::new(13., 4.),
        Vec2::new(0., -12.),
    ];
    Shape::new(shape::polygon(&gem))
        .fill(YELLOW)
        .stroke(NEON_YELLOW, 2.)
}
