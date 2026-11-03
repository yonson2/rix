use crate::game::events::UpdatePoints;
use crate::game::territory::FreeRegion;
use crate::theme::{FontAssets, palette::NEON_CYAN};
use crate::utils::{FadeShape, update_shape_fade};

use super::events::{self, IncreasePoints};
use super::polygon::DrawnShape;
use super::setup::{GameRng, LevelProgress, Points, Run};
use super::state::{Phase, TransitionState};
use super::wall::Field;
use super::{AppSystems, PausableSystems};
use bevy::prelude::*;

// Point values for scoring
pub const ENEMY_POINTS: u128 = 50;
pub const ITEM_POINTS: u128 = 100;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::LevelTransition), start_transition)
        .add_systems(
            Update,
            (update_shape_fade, animate_banner, handle_transition_loading)
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

/// The "LEVEL N" banner of a level clear ([038]): it pops in, holds, and fades out; the
/// next level waits for it.
#[derive(Component)]
struct Banner(Timer);

const BANNER_SECS: f32 = 1.2;
const BANNER_POP_SECS: f32 = 0.25;
const BANNER_FADE_SECS: f32 = 0.3;

fn start_transition(
    mut commands: Commands,
    mut next_state: ResMut<NextState<TransitionState>>,
    mut rng: ResMut<GameRng>,
    shapes: Query<Entity, With<DrawnShape>>,
    run: Res<Run>,
    fonts: Res<FontAssets>,
) {
    for ent in shapes.iter() {
        if let Ok(mut ec) = commands.get_entity(ent) {
            ec.try_insert(FadeShape::with_direction(0.3, rng.f32()));
        }
    }
    commands.spawn((
        Banner(Timer::from_seconds(BANNER_SECS, TimerMode::Once)),
        DespawnOnExit(Phase::LevelTransition),
        Text2d::new(format!("LEVEL {}", run.level + 1)),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(64.),
            ..default()
        },
        TextColor(NEON_CYAN),
        Transform::from_xyz(0., 0., 15.).with_scale(Vec3::ZERO),
    ));
    next_state.set(TransitionState::Loading);
}

fn animate_banner(
    time: Res<Time>,
    mut banner: Query<(&mut Banner, &mut Transform, &mut TextColor)>,
) {
    for (mut banner, mut transform, mut color) in &mut banner {
        banner.0.tick(time.delta());
        let t = banner.0.elapsed_secs();
        let pop = EaseFunction::BackOut.sample_clamped(t / BANNER_POP_SECS);
        transform.scale = Vec3::splat(pop);
        let fade = ((BANNER_SECS - t) / BANNER_FADE_SECS).clamp(0., 1.);
        color.0.set_alpha(fade);
    }
}

fn handle_transition_loading(
    mut next_state: ResMut<NextState<Phase>>,
    mut run: ResMut<Run>,
    mut progress: ResMut<LevelProgress>,
    mut rng: ResMut<GameRng>,
    shapes: Query<Entity, With<DrawnShape>>,
    banner: Query<&Banner>,
) {
    if shapes.iter().count() == 0 && banner.iter().all(|b| b.0.is_finished()) {
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

fn handle_update_points(mut run: ResMut<Run>, mut events: MessageReader<UpdatePoints>) {
    for event in events.read() {
        let amount = ((event.to - event.from) * 100.) as u128;
        let amount = level_points(amount, run.level);

        let Points(old_points) = run.points;
        run.points = Points(old_points.saturating_add(amount));
        info!("Points now at: {}", run.points);
    }
}

fn handle_increase_points(mut run: ResMut<Run>, mut events: MessageReader<IncreasePoints>) {
    for event in events.read() {
        let amount = level_points(event.0, run.level);

        let Points(old_points) = run.points;
        run.points = Points(old_points.saturating_add(amount));
        info!("Points from enemy; now at: {}", run.points);
    }
}

/// `amount` doubled per level after the first; it saturates at `u128::MAX` ([031]).
fn level_points(amount: u128, level: usize) -> u128 {
    let multiplier = 2u128.checked_pow(level as u32 - 1).unwrap_or(u128::MAX);
    amount.saturating_mul(multiplier)
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
        assert_eq!(level_points(50, 1), 50);
        assert_eq!(level_points(50, 3), 200);
        assert_eq!(level_points(100, 123), u128::MAX);
        assert_eq!(level_points(50, 200), u128::MAX);
        assert_eq!(level_points(0, 200), 0);
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
        world.write_message(IncreasePoints(ENEMY_POINTS));
        world.write_message(UpdatePoints::new(0., 0.5));
        world.run_system_once(handle_increase_points).unwrap();
        assert_eq!(world.resource::<Run>().points.0, u128::MAX);
        world.run_system_once(handle_update_points).unwrap();
        assert_eq!(world.resource::<Run>().points.0, u128::MAX);
    }
}
