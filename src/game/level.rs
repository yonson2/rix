use crate::game::events::UpdatePoints;
use crate::game::territory::FreeRegion;
use crate::utils::{FadeShape, update_shape_fade};

use super::events::{self, IncreasePoints};
use super::polygon::DrawnShape;
use super::setup::{GameRng, LevelProgress, Points, Run};
use super::state::{Phase, TransitionState};
use super::{AppSystems, PausableSystems};
use bevy::prelude::*;

// Point values for scoring
pub const ENEMY_POINTS: u128 = 50;
pub const ITEM_POINTS: u128 = 100;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::LevelTransition), start_transition)
        .add_systems(
            Update,
            (update_shape_fade, handle_transition_loading)
                .in_set(AppSystems::Update)
                .run_if(in_state(TransitionState::Loading)),
        );

    app.add_systems(
        Update,
        (
            calculate_area_progress.run_if(resource_changed::<FreeRegion>),
            handle_update_points,
            handle_increase_points,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    app.add_systems(Update, despawn_after.in_set(AppSystems::TickTimers));
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
    mut level: ResMut<LevelProgress>,
    mut update_points: MessageWriter<events::UpdatePoints>,
) {
    let old_progress = level.current;
    level.current = free.claimed_fraction();

    if level.current - old_progress > 0.01 {
        update_points.write(UpdatePoints::new(old_progress, level.current));
    }
}

fn handle_update_points(mut run: ResMut<Run>, mut events: MessageReader<UpdatePoints>) {
    for event in events.read() {
        let amount = ((event.to - event.from) * 100.) as u128;
        let amount = amount * 2u128.pow(run.level as u32 - 1);

        let Points(old_points) = run.points;
        run.points = Points(amount + old_points);
        info!("Points now at: {}", run.points);
    }
}

fn handle_increase_points(mut run: ResMut<Run>, mut events: MessageReader<IncreasePoints>) {
    for event in events.read() {
        let amount = event.0 * 2u128.pow(run.level as u32 - 1);

        let Points(old_points) = run.points;
        run.points = Points(amount + old_points);
        info!("Points from enemy; now at: {}", run.points);
    }
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
