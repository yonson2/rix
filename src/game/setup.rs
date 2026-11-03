use std::fmt::Display;

use bevy::prelude::*;

use super::{AppSystems, PausableSystems, effects, events, state::Phase};

pub(super) fn plugin(app: &mut App) {
    let run = Run::with_seed(random_seed());
    app.add_plugins((events::plugin, effects::plugin))
        .insert_resource(GameRng::new(run.seed))
        .insert_resource(run)
        .init_resource::<LevelProgress>()
        .add_systems(
            FixedUpdate,
            handle_current_level
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

/// One run, from level 1 to the death ([028]).
#[derive(Resource, Clone, Debug)]
pub struct Run {
    pub seed: u64,
    pub level: usize,
    pub points: Points,
    pub kill_count: usize,
}

/// The claimed fraction of the level and the fraction that clears it.
#[derive(Resource, Clone, Copy, Debug)]
pub struct LevelProgress {
    pub target: f32,
    pub current: f32,
}

/// All gameplay randomness (AGENTS.md hard rule). The run seed seeds level 1; each later
/// level reseeds it with `seed + level`.
#[derive(Resource, Deref, DerefMut)]
pub struct GameRng(pub fastrand::Rng);

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

impl Run {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            level: 1,
            points: Points::default(),
            kill_count: 0,
        }
    }
}

impl Default for LevelProgress {
    fn default() -> Self {
        Self {
            target: 0.75,
            current: 0.,
        }
    }
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        Self(fastrand::Rng::with_seed(seed))
    }
}

/// The seed of a new run: the only use of the global `fastrand`. 40 bits, so its code has
/// eight characters ([067]) and the `seed + level` reseeds do not overflow.
pub fn random_seed() -> u64 {
    fastrand::u64(..1 << 40)
}

/// Starts a new run with this seed: its stats, an empty level and the seed's RNG.
pub fn new_run(commands: &mut Commands, seed: u64) {
    commands.insert_resource(Run::with_seed(seed));
    commands.insert_resource(LevelProgress::default());
    commands.insert_resource(GameRng::new(seed));
}

fn handle_current_level(progress: Res<LevelProgress>, mut next_state: ResMut<NextState<Phase>>) {
    if progress.is_changed() && progress.current >= progress.target {
        next_state.set(Phase::LevelTransition);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_below_ten_million_use_thousands_separators() {
        let shown: Vec<String> = [0, 999, 1_000, 1_234_567, 9_999_999]
            .map(|p| Points(p).to_string())
            .into();
        assert_eq!(shown, ["0", "999", "1,000", "1,234,567", "9,999,999"]);
    }

    #[test]
    fn points_from_ten_million_use_truncated_scientific_notation() {
        assert_eq!(Points(10_000_000).to_string(), "1.000e7");
        assert_eq!(Points(19_999_999).to_string(), "1.999e7");
        assert_eq!(Points(u128::MAX).to_string(), "3.402e38");
    }

    #[test]
    fn with_seed_starts_level_1() {
        let run = Run::with_seed(7);
        assert_eq!(
            (run.level, run.seed, run.kill_count, run.points.0),
            (1, 7, 0, 0)
        );
        let progress = LevelProgress::default();
        assert_eq!((progress.target, progress.current), (0.75, 0.));
    }
}
