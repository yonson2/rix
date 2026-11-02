use crate::rix::events::UpdatePoints;
use crate::rix::polygon::{Territory, is_point_in_polygon};
use crate::utils::{FadeShape, update_shape_fade};

use super::events::{self, IncreasePoints};
use super::polygon::DrawnShape;
use super::setup::{LevelInfo, Points};
use super::state::{AppState, IsPaused, TransitionState};
use bevy::prelude::*;

// Point values for scoring
pub const ENEMY_POINTS: u128 = 50;
pub const ITEM_POINTS: u128 = 100;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LevelTransition), start_transition)
            .add_systems(
                Update,
                (update_shape_fade, handle_transition_loading)
                    .run_if(in_state(TransitionState::Loading)),
            );

        app.add_systems(
            Update,
            (
                calculate_area_progress,
                handle_update_points,
                handle_increase_points,
            )
                .run_if(in_state(IsPaused::Running)),
        );

        app.add_systems(Update, despawn_after);
    }
}

fn start_transition(
    mut commands: Commands,
    mut next_state: ResMut<NextState<TransitionState>>,
    mut level_info: ResMut<LevelInfo>,
    shapes: Query<Entity, With<DrawnShape>>,
) {
    for ent in shapes.iter() {
        if let Ok(mut ec) = commands.get_entity(ent) {
            ec.try_insert(FadeShape::with_direction(0.3, level_info.rng.f32()));
        }
    }
    next_state.set(TransitionState::Loading);
}

fn handle_transition_loading(
    mut next_state: ResMut<NextState<AppState>>,
    mut level_info: ResMut<LevelInfo>,
    shapes: Query<Entity, With<DrawnShape>>,
) {
    if shapes.iter().count() == 0 {
        level_info.current = 0.;
        level_info.level += 1;
        level_info.rng = fastrand::Rng::with_seed(level_info.seed + level_info.level as u64);
        info!("Congrats, going to new level! {}", level_info.level);
        next_state.set(AppState::InGame);
    }
}

fn calculate_area_progress(
    window: Single<&Window>,
    shapes: Query<&Territory>,
    added: Query<Entity, Added<Territory>>,
    mut level_info: ResMut<LevelInfo>,
    mut update_points: MessageWriter<events::UpdatePoints>,
) {
    if added.is_empty() {
        return;
    }
    let rng = &mut level_info.rng;

    let playable_width = window.width() - super::wall::WALL_THICKNESS * 2.0;
    let playable_height = window.height() - super::wall::WALL_THICKNESS * 2.0;

    let polygons: Vec<Vec<Vec2>> = shapes.iter().map(|t| t.0.clone()).collect();

    const NUM_SAMPLES: usize = 100000;

    let mut filled_points = 0.0;

    for _ in 0..NUM_SAMPLES {
        let world_x = rng.f32() * playable_width - playable_width / 2.0;
        let world_y = rng.f32() * playable_height - playable_height / 2.0;
        let point = Vec2::new(world_x, world_y);

        for polygon in &polygons {
            if is_point_in_polygon(&point, polygon) {
                filled_points += 1.0;
                break;
            }
        }
    }

    let progress = filled_points / NUM_SAMPLES as f32;
    let old_progress = level_info.current;
    level_info.current = progress;

    if level_info.current - old_progress > 0.01 {
        update_points.write(UpdatePoints::new(old_progress, level_info.current));
    }
}

fn handle_update_points(mut level_info: ResMut<LevelInfo>, mut events: MessageReader<UpdatePoints>) {
    for event in events.read() {
        let amount = ((event.to - event.from) * 100.) as u128;
        let amount = amount * 2u128.pow(level_info.level as u32 - 1);

        let Points(old_points) = level_info.points;
        level_info.points = Points(amount + old_points);
        info!("Points now at: {}", level_info.points);
    }
}

fn handle_increase_points(
    mut level_info: ResMut<LevelInfo>,
    mut events: MessageReader<IncreasePoints>,
) {
    for event in events.read() {
        let amount = event.0 * 2u128.pow(level_info.level as u32 - 1);

        let Points(old_points) = level_info.points;
        level_info.points = Points(amount + old_points);
        info!("Points from enemy; now at: {}", level_info.points);
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
        if da.is_finished() {
            if let Ok(mut ec) = commands.get_entity(e) {
                ec.despawn();
            }
        }
    }
}
