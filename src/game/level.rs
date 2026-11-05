use crate::game::events::UpdatePoints;
use crate::game::territory::FreeRegion;
use crate::utils::{FadeShape, update_shape_fade};

use super::events::{self, IncreasePoints};
use super::fx::PopUp;
use super::items::multiplier::{BOOST_FACTOR, ScoreBoost};
use super::polygon::DrawnShape;
use super::rules::Rules;
use super::setup::{GameRng, LevelProgress, Points, Run};
use super::state::{Phase, TransitionState};
use super::wall::Field;
use super::{AppSystems, PausableSystems};
use bevy::prelude::*;

// Point values for scoring
pub const ENEMY_POINTS: u128 = 50;
pub const ITEM_POINTS: u128 = 100;
/// The claim that clears a level pays this for each whole percent above the target ([110]).
const OVER_POINTS: u128 = 10;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::LevelTransition), start_transition)
        .add_systems(
            Update,
            (update_shape_fade, handle_transition_loading)
                .in_set(AppSystems::Update)
                .run_if(in_state(TransitionState::Loading)),
        );

    app.add_systems(
        FixedUpdate,
        (
            calculate_area_progress.run_if(resource_changed::<FreeRegion>),
            handle_update_points,
            handle_increase_points,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    app.add_systems(FixedUpdate, despawn_after.in_set(AppSystems::TickTimers));
}

fn start_transition(
    mut commands: Commands,
    mut next_state: ResMut<NextState<TransitionState>>,
    mut rng: ResMut<GameRng>,
    shapes: Query<Entity, With<DrawnShape>>,
) {
    for ent in shapes.iter() {
        if let Ok(mut ec) = commands.get_entity(ent) {
            ec.try_insert(FadeShape::with_direction(0.3, rng.f32()));
        }
    }
    next_state.set(TransitionState::Loading);
}

/// The next level starts as soon as the claimed shapes have faded: no banner, so the game
/// keeps its pace (D30, [097]).
fn handle_transition_loading(
    mut next_state: ResMut<NextState<Phase>>,
    mut run: ResMut<Run>,
    mut progress: ResMut<LevelProgress>,
    mut rng: ResMut<GameRng>,
    shapes: Query<Entity, With<DrawnShape>>,
) {
    if shapes.iter().count() == 0 {
        progress.current = 0.;
        run.level += 1;
        *rng = GameRng::new(run.seed + run.level as u64);
        info!("Congrats, going to new level! {}", run.level);
        next_state.set(Phase::Playing);
    }
}

fn calculate_area_progress(
    free: Res<FreeRegion>,
    field: Res<Field>,
    mut level: ResMut<LevelProgress>,
    mut update_points: MessageWriter<events::UpdatePoints>,
) {
    let old_progress = level.current;
    level.current = free.claimed_fraction(&field);

    if level.current - old_progress > 0.01 {
        update_points.write(UpdatePoints::new(old_progress, level.current));
    }
}

fn handle_update_points(
    mut commands: Commands,
    mut run: ResMut<Run>,
    mut events: MessageReader<UpdatePoints>,
    progress: Res<LevelProgress>,
    boost: Option<Res<ScoreBoost>>,
    rules: Res<Rules>,
) {
    for event in events.read() {
        let amount = ((event.to - event.from) * 100.) as u128;
        let amount = boosted(level_points(amount, run.level, &rules), &boost);
        let over = overclaim_bonus(event.from, event.to, progress.target);
        let over = boosted(level_points(over, run.level, &rules), &boost);
        if over > 0 {
            commands.trigger(PopUp::new(format!("+{} OVER", Points(over)), Vec2::ZERO));
        }

        let Points(old_points) = run.points;
        run.points = Points(old_points.saturating_add(amount).saturating_add(over));
        info!("Points now at: {}", run.points);
    }
}

/// The claim from `from` to `to` that crosses `target` pays `OVER_POINTS` for each whole
/// percent above it ([110]).
fn overclaim_bonus(from: f32, to: f32, target: f32) -> u128 {
    if from >= target || to < target {
        return 0;
    }
    // The margin keeps 0.85 - 0.75 at 10 percent, not 9.99.
    ((to - target) * 100. + 1e-3) as u128 * OVER_POINTS
}

/// One claim that eats `eaten` enemies pays `eaten` times `ENEMY_POINTS` for each ([110]):
/// this on top of the pay of each kill.
pub fn multi_eat_bonus(eaten: usize) -> u128 {
    let eaten = eaten as u128;
    eaten * eaten.saturating_sub(1) * ENEMY_POINTS
}

/// The pop-up of a multi-eat ([110]); one enemy gets none.
pub fn multi_eat_label(eaten: usize) -> Option<String> {
    match eaten {
        0 | 1 => None,
        2 => Some("DOUBLE!".into()),
        3 => Some("TRIPLE!".into()),
        4 => Some("QUAD!".into()),
        n => Some(format!("{n}X!")),
    }
}

fn handle_increase_points(
    mut run: ResMut<Run>,
    mut events: MessageReader<IncreasePoints>,
    boost: Option<Res<ScoreBoost>>,
    rules: Res<Rules>,
) {
    for event in events.read() {
        let amount = boosted(level_points(event.0, run.level, &rules), &boost);

        let Points(old_points) = run.points;
        run.points = Points(old_points.saturating_add(amount));
        info!("Points from enemy; now at: {}", run.points);
    }
}

/// A gem's points count `BOOST_FACTOR` times ([046]).
fn boosted(amount: u128, boost: &Option<Res<ScoreBoost>>) -> u128 {
    match boost {
        Some(_) => amount.saturating_mul(BOOST_FACTOR),
        None => amount,
    }
}

/// `amount` doubled per level after the first, then the rules' pay ([121]); it saturates at
/// `u128::MAX` ([031]).
fn level_points(amount: u128, level: usize, rules: &Rules) -> u128 {
    let multiplier = 2u128.checked_pow(level as u32 - 1).unwrap_or(u128::MAX);
    rules.pay(amount.saturating_mul(multiplier))
}

#[derive(Component, Debug, Deref, DerefMut)]
pub struct DespawnAfter(pub Timer);

impl DespawnAfter {
    pub fn from_seconds(duration: f32) -> Self {
        Self(Timer::from_seconds(duration, TimerMode::Once))
    }

    // Helper method to get remaining seconds
    pub fn remaining_secs(&self) -> f32 {
        self.duration().as_secs_f32() - self.elapsed_secs()
    }
}

fn despawn_after(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DespawnAfter)>,
) {
    for (e, mut da) in query.iter_mut() {
        da.tick(time.delta());
        if da.is_finished()
            && let Ok(mut ec) = commands.get_entity(e)
        {
            ec.despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn points_double_per_level_and_saturate() {
        assert_eq!(level_points(50, 1, &Rules::default()), 50);
        assert_eq!(level_points(50, 3, &Rules::default()), 200);
        assert_eq!(level_points(100, 123, &Rules::default()), u128::MAX);
        assert_eq!(level_points(50, 200, &Rules::default()), u128::MAX);
        assert_eq!(level_points(0, 200, &Rules::default()), 0);
        let overdrive = Rules {
            speed: 2,
            ..default()
        };
        assert_eq!(
            level_points(50, 3, &overdrive),
            300,
            "a faster challenge pays more"
        );
    }

    #[test]
    fn points_at_level_200_saturate_the_score() {
        let mut world = World::new();
        world.init_resource::<Messages<IncreasePoints>>();
        world.init_resource::<Messages<UpdatePoints>>();
        let mut run = Run::with_seed(1);
        run.level = 200;
        run.points = Points(u128::MAX - 10);
        world.insert_resource(run);
        world.insert_resource(LevelProgress::default());
        world.init_resource::<Rules>();
        world.write_message(IncreasePoints(ENEMY_POINTS));
        world.write_message(UpdatePoints::new(0., 0.5));
        world.run_system_once(handle_increase_points).unwrap();
        assert_eq!(world.resource::<Run>().points.0, u128::MAX);
        world.run_system_once(handle_update_points).unwrap();
        assert_eq!(world.resource::<Run>().points.0, u128::MAX);
    }

    #[test]
    fn a_gem_doubles_kill_and_claim_points_while_it_lasts() {
        let score = |boost: bool| {
            let mut world = World::new();
            world.init_resource::<Messages<IncreasePoints>>();
            world.init_resource::<Messages<UpdatePoints>>();
            world.insert_resource(Run::with_seed(1));
            world.insert_resource(LevelProgress::default());
            world.init_resource::<Rules>();
            if boost {
                world.insert_resource(ScoreBoost::default());
            }
            world.write_message(IncreasePoints(ENEMY_POINTS));
            world.write_message(UpdatePoints::new(0., 0.1));
            world.run_system_once(handle_increase_points).unwrap();
            world.run_system_once(handle_update_points).unwrap();
            world.resource::<Run>().points.0
        };
        assert_eq!(score(false), 60);
        assert_eq!(score(true), 120);
    }

    #[test]
    fn the_clearing_claim_pays_for_each_percent_over_the_target() {
        assert_eq!(overclaim_bonus(0.5, 0.74, 0.75), 0, "no clear");
        assert_eq!(overclaim_bonus(0.5, 0.75, 0.75), 0, "no percent over");
        assert_eq!(overclaim_bonus(0.6, 0.85, 0.75), 10 * OVER_POINTS);
        assert_eq!(
            overclaim_bonus(0.7, 0.888, 0.75),
            13 * OVER_POINTS,
            "whole percents"
        );
        assert_eq!(
            overclaim_bonus(0.76, 0.9, 0.75),
            0,
            "only the claim that crosses"
        );
    }

    #[test]
    fn an_overclaim_pays_with_the_level_and_the_gem() {
        let score = |level: usize, boost: bool| {
            let mut world = World::new();
            world.init_resource::<Messages<UpdatePoints>>();
            let mut run = Run::with_seed(1);
            run.level = level;
            world.insert_resource(run);
            world.insert_resource(LevelProgress::default());
            world.init_resource::<Rules>();
            if boost {
                world.insert_resource(ScoreBoost::default());
            }
            world.write_message(UpdatePoints::new(0.6, 0.85));
            world.run_system_once(handle_update_points).unwrap();
            world.resource::<Run>().points.0
        };
        // 25 for the claimed percent, 100 for the 10 percent over.
        assert_eq!(score(1, false), 125);
        assert_eq!(score(3, false), 500);
        assert_eq!(score(1, true), 250);
    }

    #[test]
    fn a_claim_that_eats_k_enemies_pays_k_times_for_each() {
        assert_eq!(multi_eat_bonus(0), 0);
        assert_eq!(multi_eat_bonus(1), 0);
        // With the kills' own pay: 2 x 2 x 50, 3 x 3 x 50.
        assert_eq!(multi_eat_bonus(2) + 2 * ENEMY_POINTS, 200);
        assert_eq!(multi_eat_bonus(3) + 3 * ENEMY_POINTS, 450);
        let labels: Vec<_> = (0..=6).map(multi_eat_label).collect();
        assert_eq!(
            labels,
            [
                None,
                None,
                Some("DOUBLE!".into()),
                Some("TRIPLE!".into()),
                Some("QUAD!".into()),
                Some("5X!".into()),
                Some("6X!".into())
            ]
        );
    }
}
