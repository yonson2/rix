use std::fmt::Display;

use bevy::prelude::*;

use super::{
    enemies::register_enemy_systems,
    events::EventsPlugin,
    items::register_item_systems,
    state::{AppState, IsPaused},
};

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EventsPlugin)
            .insert_resource(LevelInfo::default())
            .add_systems(
                OnEnter(AppState::Setup),
                (setup_world, finish_setup).chain(),
            )
            .add_systems(
                Update,
                handle_current_level.run_if(in_state(IsPaused::Running)),
            );
    }
}

fn setup_world(world: &mut World) {
    register_enemy_systems(world);
    register_item_systems(world);
}

fn finish_setup(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Menu);
}

#[derive(Resource, Clone, Debug, Deref, DerefMut)]
pub struct LevelInfo {
    pub level: usize,
    pub target: f32,
    pub current: f32,
    pub points: Points,
    pub seed: u64,
    #[deref]
    pub rng: fastrand::Rng,
}

#[derive(Clone, Copy, Debug, Default, Deref, DerefMut)]
pub struct Points(pub u128);

impl Display for Points {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.0;

        if value < 10_000_000 {
            let mut s = value.to_string();
            let mut i = s.len();

            while i > 3 {
                i -= 3;
                s.insert(i, ',');
            }
            write!(f, "{}", s)
        } else {
            // For very large numbers, calculate exponent and mantissa manually
            let s = value.to_string();
            let exponent = s.len() - 1;

            let first_digit = &s[0..1];
            let decimal_part = if s.len() > 1 {
                &s[1..4.min(s.len())]
            } else {
                ""
            };

            if decimal_part.is_empty() {
                write!(f, "{}e{}", first_digit, exponent)
            } else {
                let decimal_padded = format!("{:0<3}", decimal_part);
                write!(f, "{}.{}e{}", first_digit, decimal_padded, exponent)
            }
        }
    }
}

impl LevelInfo {
    pub fn reset(&mut self) {
        *self = LevelInfo::default();
    }
}
impl Default for LevelInfo {
    fn default() -> Self {
        let seed = fastrand::u64(u64::MIN..u64::MAX - 100); //leave some buffer for levels
        Self {
            level: 1,
            target: 0.75,
            current: 0.0,
            points: Points::default(),
            seed,
            rng: fastrand::Rng::with_seed(seed),
        }
    }
}

fn handle_current_level(
    level_info: Option<ResMut<LevelInfo>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(level_info) = level_info {
        if level_info.is_changed() && level_info.current >= level_info.target {
            next_state.set(AppState::LevelTransition);
        }
    }
}
