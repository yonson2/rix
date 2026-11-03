use bevy::{prelude::*, sprite::Anchor};

use crate::{
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{NEON_CYAN, NEON_PURPLE, NEON_WHITE, PURPLE},
        widget::{self, Activate},
    },
};

use super::{
    setup::{LevelProgress, Points, Run},
    state::{IsPaused, Phase},
    wall::{Field, WALL_THICKNESS},
};

/// Below the field top: the badge and score row, and the bar along the top wall's inner
/// edge. Both stay inside the 65 unit top wall.
const ROW_Y: f32 = 28.;
const BAR_Y: f32 = 60.;
const BAR_HEIGHT: f32 = 6.;
const BADGE_SIZE: Vec2 = Vec2::new(84., 32.);
const SCORE_FONT_SIZE: FontSize = FontSize::Px(30.);
const BADGE_FONT_SIZE: FontSize = FontSize::Px(22.);
/// The pause button ([073]), right of the badge; the row's middle stays free.
const PAUSE_SIZE: Vec2 = Vec2::new(52., 40.);
const PAUSE_GAP: f32 = 12.;
/// The shown score and bar close this share of the gap per second.
const CATCH_UP: f32 = 10.;
/// Above the claimed shapes, enemies and items.
const HUD_Z: f32 = 10.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::Playing), setup_ui);
    app.add_systems(
        Update,
        (update_level.run_if(hud_changed), count_up).run_if(in_state(Phase::Playing)),
    );
}

#[derive(Component)]
struct UiLevel;
/// The score on screen, which counts up to `Run::points`.
#[derive(Component)]
struct UiPoints(Points);
/// The bar's shown fraction of the level target.
#[derive(Component)]
struct UiBar(f32);

/// The HUD sits on the top wall ([037]): the level badge on the left, the score on the
/// right, a bar toward the level target along the wall's inner edge. The middle of the
/// row stays free for a phone's camera cutout ([080]).
/// The level text redraws only when the run changes; the RNG changes on every bounce, so
/// it does not count ([028]).
fn hud_changed(run: Res<Run>, progress: Res<LevelProgress>) -> bool {
    run.is_changed() || progress.is_changed()
}

fn setup_ui(
    mut commands: Commands,
    run: Res<Run>,
    progress: Res<LevelProgress>,
    fonts: Res<FontAssets>,
    field: Res<Field>,
) {
    let top = field.y / 2.;
    let inner = field.x / 2. - WALL_THICKNESS;
    let text = |text: String, font_size, color| {
        (
            Text2d::new(text),
            TextFont {
                font: fonts.main.clone().into(),
                font_size,
                ..default()
            },
            TextColor(color),
        )
    };

    commands.spawn((
        DespawnOnExit(Phase::Playing),
        Shape::new(shape::rounded_rectangle(BADGE_SIZE, 8.))
            .fill(PURPLE)
            .stroke(NEON_PURPLE, 2.),
        Transform::from_xyz(-inner + BADGE_SIZE.x / 2., top - ROW_Y, HUD_Z),
        children![(
            UiLevel,
            text(level_text(run.level), BADGE_FONT_SIZE, Color::WHITE),
            Transform::from_xyz(0., 0., 0.1),
        )],
    ));
    let pause_x = -inner + BADGE_SIZE.x + PAUSE_GAP + PAUSE_SIZE.x / 2.;
    commands
        .spawn((
            DespawnOnExit(Phase::Playing),
            widget::button_at("II", fonts.main.clone(), Vec2::ZERO, PAUSE_SIZE),
        ))
        .insert(Transform::from_xyz(pause_x, top - ROW_Y, HUD_Z))
        .observe(pause);
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        UiPoints(run.points),
        text(run.points.to_string(), SCORE_FONT_SIZE, NEON_WHITE),
        Anchor::CENTER_RIGHT,
        Transform::from_xyz(inner, top - ROW_Y, HUD_Z),
    ));
    let bar = Vec2::new(2. * inner, BAR_HEIGHT);
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        Sprite::from_color(Color::BLACK.with_alpha(0.7), bar),
        Transform::from_xyz(0., top - BAR_Y, HUD_Z),
    ));
    let shown = bar_fraction(&progress);
    commands.spawn((
        DespawnOnExit(Phase::Playing),
        UiBar(shown),
        Sprite::from_color(NEON_CYAN, bar * Vec2::new(shown, 1.)),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(-inner, top - BAR_Y, HUD_Z + 0.1),
    ));
}

/// Only a running game pauses: not while dying.
fn pause(_: On<Activate>, state: Res<State<IsPaused>>, mut next: ResMut<NextState<IsPaused>>) {
    if *state.get() == IsPaused::Running {
        next.set(IsPaused::Paused);
    }
}

fn level_text(level: usize) -> String {
    format!("LV {level}")
}

/// The claimed part of the level target, from 0 to 1.
fn bar_fraction(progress: &LevelProgress) -> f32 {
    (progress.current / progress.target).clamp(0., 1.)
}

fn update_level(mut level: Single<&mut Text2d, With<UiLevel>>, run: Res<Run>) {
    level.0 = level_text(run.level);
}

/// Draws only: the score and the bar move toward the run's values and stop there.
fn count_up(
    time: Res<Time>,
    run: Res<Run>,
    progress: Res<LevelProgress>,
    field: Res<Field>,
    points: Single<(&mut UiPoints, &mut Text2d)>,
    bar: Single<(&mut UiBar, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    let (mut shown, mut text) = points.into_inner();
    let next = count_toward(shown.0.0, run.points.0, dt);
    if next != shown.0.0 {
        shown.0 = Points(next);
        **text = shown.0.to_string();
    }
    let (mut shown, mut sprite) = bar.into_inner();
    let next = ease_toward(shown.0, bar_fraction(&progress), dt);
    if next != shown.0 {
        shown.0 = next;
        let width = (field.x - 2. * WALL_THICKNESS) * next;
        sprite.custom_size = Some(Vec2::new(width, BAR_HEIGHT));
    }
}

/// One frame of the count-up: `shown` closes `CATCH_UP` of the gap per second, at least
/// one point, never past `target`. A lower target shows at once.
fn count_toward(shown: u128, target: u128, dt: f32) -> u128 {
    if target <= shown {
        return target;
    }
    let step = ((target - shown) as f64 * f64::from((CATCH_UP * dt).min(1.))).ceil() as u128;
    (shown + step.max(1)).min(target)
}

/// One frame of the bar: as `count_toward`, it snaps when the gap is too small to see.
fn ease_toward(shown: f32, target: f32, dt: f32) -> f32 {
    if (target - shown).abs() < 1e-3 {
        return target;
    }
    shown + (target - shown) * (CATCH_UP * dt).min(1.)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::setup::GameRng;

    #[derive(Resource, Default)]
    struct Updates(u32);

    fn count(mut updates: ResMut<Updates>) {
        updates.0 += 1;
    }

    fn bounce(mut rng: ResMut<GameRng>) {
        rng.f32();
    }

    #[test]
    fn hud_updates_only_when_run_or_progress_change() {
        let mut app = App::new();
        app.insert_resource(Run::with_seed(1))
            .init_resource::<LevelProgress>()
            .insert_resource(GameRng::new(1))
            .init_resource::<Updates>()
            .add_systems(Update, (bounce, count.run_if(hud_changed)).chain());
        app.update();
        let first = app.world().resource::<Updates>().0;
        for _ in 0..600 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<Updates>().0,
            first,
            "10 s of RNG draws do not redraw the HUD"
        );
        app.world_mut().resource_mut::<Run>().points.0 += 50;
        app.update();
        app.world_mut().resource_mut::<LevelProgress>().current = 0.5;
        app.update();
        assert_eq!(app.world().resource::<Updates>().0, first + 2);
    }

    #[test]
    fn the_score_counts_up_to_the_target_in_under_a_second() {
        let (mut shown, mut frames) = (0, 0);
        while shown < 1000 {
            let next = count_toward(shown, 1000, 1. / 60.);
            assert!(next > shown && next <= 1000);
            (shown, frames) = (next, frames + 1);
        }
        assert!(frames <= 60, "{frames} frames");
        assert_eq!(
            count_toward(500, 20, 1. / 60.),
            20,
            "a lower score shows at once"
        );
        assert_eq!(count_toward(7, 7, 0.), 7);
    }

    #[test]
    fn the_bar_eases_to_the_target_and_stops() {
        let (mut shown, mut frames) = (0., 0);
        while shown != 0.5 {
            shown = ease_toward(shown, 0.5, 1. / 60.);
            frames += 1;
        }
        assert!(frames <= 60, "{frames} frames");
        let progress = |current| LevelProgress {
            target: 0.75,
            current,
        };
        assert_eq!(bar_fraction(&progress(0.375)), 0.5);
        assert_eq!(bar_fraction(&progress(0.9)), 1., "a claim past the target");
    }
}
