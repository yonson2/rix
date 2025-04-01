use crate::rix::events::UpdatePoints;
use crate::rix::polygon::{extract_points_from_shape, is_point_in_polygon};

use super::events::{self, IncreasePoints};
use super::setup::{LevelInfo, Points};
use super::state::{AppState, IsPaused, TransitionState};
use bevy::prelude::*;
use bevy_prototype_lyon::entity::Shape;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::LevelTransition), start_transition)
            .add_systems(
                Update,
                handle_transition_loading.run_if(in_state(TransitionState::Loading)),
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
    }
}

fn start_transition(mut next_state: ResMut<NextState<TransitionState>>) {
    //TODO: animate the shapes, going out, setting alpha, etc...
    next_state.set(TransitionState::Loading);
}

fn handle_transition_loading(
    mut next_state: ResMut<NextState<AppState>>,
    mut level_info: ResMut<LevelInfo>,
) {
    level_info.level += 1;
    level_info.current = 0.;
    level_info.rng = fastrand::Rng::with_seed(level_info.seed + level_info.level as u64);
    info!("Congrats, going to new level! {}", level_info.level);

    next_state.set(AppState::InGame);
}

fn calculate_area_progress(
    window: Single<&Window>,
    shapes: Query<&Shape>,
    added: Query<Entity, Added<Shape>>,
    mut level_info: ResMut<LevelInfo>,
    mut update_points: EventWriter<events::UpdatePoints>,
) {
    if added.is_empty() {
        return;
    }
    let rng = &mut level_info.rng;

    let playable_width = window.width() - super::wall::WALL_THICKNESS * 2.0;
    let playable_height = window.height() - super::wall::WALL_THICKNESS * 2.0;

    let polygons: Vec<Vec<Vec2>> = shapes.iter().map(extract_points_from_shape).collect();

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
        update_points.send(UpdatePoints::new(old_progress, level_info.current));
    }
}

fn handle_update_points(mut level_info: ResMut<LevelInfo>, mut events: EventReader<UpdatePoints>) {
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
    mut events: EventReader<IncreasePoints>,
) {
    for event in events.read() {
        let amount = event.0 * 2u128.pow(level_info.level as u32 - 1);

        let Points(old_points) = level_info.points;
        level_info.points = Points(amount + old_points);
        info!("Points from enemy; now at: {}", level_info.points);
    }
}
