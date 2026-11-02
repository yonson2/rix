use crate::game::events::UpdatePoints;
use crate::game::polygon::Territory;
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
            calculate_area_progress,
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
    shapes: Query<&Territory>,
    added: Query<Entity, Added<Territory>>,
    mut level: ResMut<LevelProgress>,
    mut update_points: MessageWriter<events::UpdatePoints>,
) {
    if added.is_empty() {
        return;
    }
    let playable_width = super::wall::FIELD.x - super::wall::WALL_THICKNESS * 2.0;
    let playable_height = super::wall::FIELD.y - super::wall::WALL_THICKNESS * 2.0;

    let polygons: Vec<&[Vec2]> = shapes.iter().map(|t| t.0.as_slice()).collect();
    let progress = claimed_fraction(&polygons, Vec2::new(playable_width, playable_height));
    let old_progress = level.current;
    level.current = progress;

    if level.current - old_progress > 0.01 {
        update_points.write(UpdatePoints::new(old_progress, level.current));
    }
}

/// The part of a `size` box around the origin that the polygons cover: their union, with
/// the even-odd rule of `is_point_in_polygon`, summed over one scanline per world unit.
/// It replaced a 100,000-sample Monte Carlo estimate ([024]).
fn claimed_fraction(polygons: &[&[Vec2]], size: Vec2) -> f32 {
    let half = size / 2.;
    let rows = size.y.ceil() as u32;
    let row_height = size.y / rows as f32;
    let mut crossings = vec![];
    let mut spans: Vec<(f32, f32)> = vec![];
    let mut covered = 0.;
    for row in 0..rows {
        let y = -half.y + (row as f32 + 0.5) * row_height;
        spans.clear();
        for polygon in polygons {
            crossings.clear();
            for (a, b) in polygon.iter().zip(polygon.iter().cycle().skip(1)) {
                if (a.y > y) != (b.y > y) {
                    crossings.push(a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y));
                }
            }
            crossings.sort_by(f32::total_cmp);
            spans.extend(crossings.as_chunks::<2>().0.iter().map(|&[a, b]| (a, b)));
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut end = -half.x;
        for &(from, to) in &spans {
            let to = to.min(half.x);
            if to > end {
                covered += to - from.max(end);
                end = to;
            }
        }
    }
    covered * row_height / (size.x * size.y)
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

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: Vec2 = Vec2::new(100., 100.);

    fn rect(min: Vec2, max: Vec2) -> Vec<Vec2> {
        vec![
            min,
            Vec2::new(max.x, min.y),
            max,
            Vec2::new(min.x, max.y),
            min,
        ]
    }

    fn fraction(polygons: &[Vec<Vec2>]) -> f32 {
        let polygons: Vec<&[Vec2]> = polygons.iter().map(Vec::as_slice).collect();
        claimed_fraction(&polygons, BOX)
    }

    #[test]
    fn claimed_fraction_of_rectangles() {
        let quarter = rect(Vec2::splat(-50.), Vec2::ZERO);
        assert!((fraction(&[]) - 0.).abs() < 1e-6);
        assert!((fraction(std::slice::from_ref(&quarter)) - 0.25).abs() < 1e-4);
        let reversed: Vec<Vec2> = quarter.iter().rev().copied().collect();
        assert!(
            (fraction(&[reversed]) - 0.25).abs() < 1e-4,
            "winding does not matter"
        );
        let whole = rect(Vec2::splat(-50.), Vec2::splat(50.));
        assert!((fraction(&[whole]) - 1.).abs() < 1e-4);
    }

    #[test]
    fn claimed_fraction_counts_overlaps_once_and_clips_to_the_box() {
        let bottom_half = rect(Vec2::new(-50., -50.), Vec2::new(50., 0.));
        let nested = rect(Vec2::new(-50., -50.), Vec2::new(50., -25.));
        assert!((fraction(&[bottom_half.clone(), nested]) - 0.5).abs() < 1e-4);
        let middle = rect(Vec2::new(-50., -25.), Vec2::new(50., 25.));
        assert!((fraction(&[bottom_half, middle]) - 0.75).abs() < 1e-4);
        let past_the_box = rect(Vec2::new(0., -80.), Vec2::new(90., 80.));
        assert!((fraction(&[past_the_box]) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn claimed_fraction_of_a_concave_polygon() {
        let l_shape = [
            (-50., -50.),
            (50., -50.),
            (50., 0.),
            (0., 0.),
            (0., 50.),
            (-50., 50.),
        ]
        .map(|(x, y)| Vec2::new(x, y))
        .to_vec();
        assert!((fraction(&[l_shape]) - 0.75).abs() < 1e-4);
        let triangle = vec![
            Vec2::new(-50., -50.),
            Vec2::new(50., -50.),
            Vec2::new(-50., 50.),
        ];
        assert!((fraction(&[triangle]) - 0.5).abs() < 1e-3);
    }
}
