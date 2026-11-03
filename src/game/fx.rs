//! Juice ([040]): a claim flashes white, a death or a bomb shakes the screen, a ball bounce
//! and a kill throw sparks. It only draws: its systems run in `Update` and the sparks draw
//! from `FxRng`, never `GameRng`, so a seed replays the same game ([072]).

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::{state::IsPaused, wall::Field};
use crate::{crt::PostProcessSettings, screens::Screen, shape::Shape};

const FLASH_SECS: f32 = 0.3;
/// Not HDR: a claim of most of the field would flood the bloom.
const FLASH_COLOR: Color = Color::srgb(1., 1., 1.);
/// Trauma 1 runs out in this time.
const SHAKE_SECS: f32 = 0.5;
/// The screen offset at trauma 1, in world units.
const SHAKE_MAX: f32 = 8.;
const DEATH_SHAKE: f32 = 0.8;
pub const BOMB_SHAKE: f32 = 0.6;
const SPARK_SECS: f32 = 0.4;
const SPARK_SIZE: Vec2 = Vec2::new(7., 2.);
/// A spark loses this share of its speed per second, as `exp(-drag * t)`.
const SPARK_DRAG: f32 = 4.;
const KILL_GLOW: f32 = 3.;
/// Above the enemies and the corpses, under the explosions.
const SPARK_Z: f32 = 1.6;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Trauma>()
        .init_resource::<FxRng>()
        .add_observer(on_shake)
        .add_observer(on_burst)
        .add_systems(OnEnter(IsPaused::Dying), |mut commands: Commands| {
            commands.trigger(Shake(DEATH_SHAKE));
        })
        .add_systems(Update, (flash_claims, move_sparks, shake_screen));
}

/// A new claimed piece starts with a white fill and settles to `fill`.
#[derive(Component)]
pub struct Flash {
    timer: Timer,
    fill: Color,
}

impl Flash {
    pub fn to(fill: Color) -> Self {
        Self {
            timer: Timer::from_seconds(FLASH_SECS, TimerMode::Once),
            fill,
        }
    }
}

/// The level clear's fade sets the alpha; the flash keeps it.
fn flash_claims(
    mut commands: Commands,
    time: Res<Time>,
    mut shapes: Query<(Entity, &mut Flash, &mut Shape)>,
) {
    for (entity, mut flash, mut shape) in &mut shapes {
        flash.timer.tick(time.delta());
        let t = EaseFunction::QuadraticOut.sample_clamped(flash.timer.fraction());
        let alpha = shape.fill.map_or(1., |fill| fill.alpha());
        shape.fill = Some(FLASH_COLOR.mix(&flash.fill, t).with_alpha(alpha));
        if flash.timer.is_finished() {
            commands.entity(entity).remove::<Flash>();
        }
    }
}

/// Adds trauma to the screen shake; the offset grows with the square of the trauma.
#[derive(Event)]
pub struct Shake(pub f32);

#[derive(Resource, Default)]
struct Trauma(f32);

fn on_shake(shake: On<Shake>, mut trauma: ResMut<Trauma>) {
    trauma.0 = (trauma.0 + shake.0).min(1.);
}

/// The CRT pass moves the picture, so input and picking never see the shake. Real time:
/// a death stops the game clock ([038]) and the shake goes on.
fn shake_screen(
    time: Res<Time<Real>>,
    field: Res<Field>,
    mut trauma: ResMut<Trauma>,
    mut settings: Query<&mut PostProcessSettings>,
) {
    trauma.0 = (trauma.0 - time.delta_secs() / SHAKE_SECS).max(0.);
    let offset = shake_offset(trauma.0, time.elapsed_secs()) / field.0;
    for mut settings in &mut settings {
        settings.shake = offset;
    }
}

/// The offset at `trauma` (0 to 1) and time `t`: a fast wobble, `SHAKE_MAX` at most per
/// axis.
fn shake_offset(trauma: f32, t: f32) -> Vec2 {
    Vec2::new((t * 97.).sin(), (t * 131.).cos()) * SHAKE_MAX * trauma * trauma
}

/// Sparks fly out of `pos` in `color`, within `spread` radians around `heading`.
#[derive(Event)]
pub struct Burst {
    pub pos: Vec2,
    pub color: Color,
    heading: f32,
    spread: f32,
    count: u32,
    speed: f32,
}

impl Burst {
    /// A few slow sparks, away from the wall: `velocity` is the ball's after the bounce.
    pub fn bounce(pos: Vec2, color: Color, velocity: Vec2) -> Self {
        Self {
            pos,
            color,
            heading: velocity.to_angle(),
            spread: PI,
            count: 4,
            speed: 150.,
        }
    }

    /// Many fast sparks all around, brighter than the enemy so they bloom.
    pub fn kill(pos: Vec2, color: Color) -> Self {
        let c = color.to_linear();
        let k = KILL_GLOW;
        Self {
            pos,
            color: LinearRgba::new(c.red * k, c.green * k, c.blue * k, c.alpha).into(),
            heading: 0.,
            spread: TAU,
            count: 16,
            speed: 300.,
        }
    }
}

/// The sparks' own randomness, apart from the game's.
#[derive(Resource)]
struct FxRng(fastrand::Rng);

impl Default for FxRng {
    fn default() -> Self {
        Self(fastrand::Rng::with_seed(0))
    }
}

#[derive(Component)]
struct Spark {
    velocity: Vec2,
    timer: Timer,
}

fn on_burst(burst: On<Burst>, mut commands: Commands, mut rng: ResMut<FxRng>) {
    for _ in 0..burst.count {
        let angle = burst.heading + (rng.0.f32() - 0.5) * burst.spread;
        let speed = burst.speed * (0.5 + rng.0.f32());
        commands.spawn((
            Spark {
                velocity: Vec2::from_angle(angle) * speed,
                timer: Timer::from_seconds(SPARK_SECS, TimerMode::Once),
            },
            DespawnOnExit(Screen::Gameplay),
            Sprite::from_color(burst.color, SPARK_SIZE),
            Transform::from_translation(burst.pos.extend(SPARK_Z))
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Game time: the sparks stop while the game is paused or the player dies.
fn move_sparks(
    mut commands: Commands,
    time: Res<Time>,
    mut sparks: Query<(Entity, &mut Spark, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut spark, mut transform) in &mut sparks {
        spark.timer.tick(time.delta());
        transform.translation += (spark.velocity * dt).extend(0.);
        spark.velocity *= (-SPARK_DRAG * dt).exp();
        transform.scale = Vec3::splat(1. - spark.timer.fraction());
        if spark.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::{
        shape,
        theme::palette::CYAN,
        utils::{FadeShape, update_shape_fade},
    };

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )))
            .init_resource::<Field>()
            .init_resource::<Trauma>()
            .init_resource::<FxRng>()
            .add_observer(on_shake)
            .add_observer(on_burst)
            .add_systems(Update, (flash_claims, move_sparks, shake_screen));
        app.update();
        app
    }

    #[test]
    fn a_claim_flashes_white_then_settles_to_its_fill() {
        let mut app = app();
        let square = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y].map(|p| p * 10.);
        let piece = app
            .world_mut()
            .spawn((
                Shape::new(shape::polygon(&square)).fill(CYAN),
                Flash::to(CYAN),
            ))
            .id();
        app.update();
        let fill = |app: &App| app.world().get::<Shape>(piece).unwrap().fill.unwrap();
        assert_ne!(fill(&app), CYAN, "still flashing");
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(fill(&app), CYAN);
        assert!(app.world().get::<Flash>(piece).is_none());
    }

    #[test]
    fn a_flashing_claim_fades_out_with_the_level_clear() {
        let mut app = app();
        app.add_systems(Update, update_shape_fade.after(flash_claims));
        let piece = app
            .world_mut()
            .spawn((
                Shape::new(shape::polygon(&[Vec2::ZERO, Vec2::X, Vec2::Y])).fill(FLASH_COLOR),
                Flash::to(CYAN),
                FadeShape::with_direction(0.3, 0.),
            ))
            .id();
        let fill = |app: &App| app.world().get::<Shape>(piece).unwrap().fill.unwrap();
        let mut last = 1.;
        for _ in 0..10 {
            app.update();
            assert!(fill(&app).alpha() < last, "the fade goes on");
            last = fill(&app).alpha();
        }
        assert_ne!(fill(&app).with_alpha(1.), FLASH_COLOR, "the flash goes on");
    }

    #[test]
    fn shakes_add_up_to_one_and_run_out() {
        let mut app = app();
        app.world_mut().spawn(PostProcessSettings::default());
        app.world_mut().trigger(Shake(0.8));
        app.world_mut().trigger(Shake(0.8));
        assert_eq!(app.world().resource::<Trauma>().0, 1.);
        app.update();
        let shake = |app: &mut App| {
            app.world_mut()
                .query::<&PostProcessSettings>()
                .single(app.world())
                .unwrap()
                .shake
        };
        assert_ne!(shake(&mut app), Vec2::ZERO);
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(app.world().resource::<Trauma>().0, 0.);
        assert_eq!(shake(&mut app), Vec2::ZERO, "the picture is back in place");
    }

    #[test]
    fn the_shake_stays_within_its_maximum() {
        for i in 0..1000 {
            let offset = shake_offset(1., i as f32 * 0.013);
            assert!(offset.abs().max_element() <= SHAKE_MAX);
        }
        assert_eq!(shake_offset(0., 1.234), Vec2::ZERO);
    }

    #[test]
    fn bounce_sparks_fly_away_from_the_wall() {
        let mut app = app();
        app.world_mut()
            .trigger(Burst::bounce(Vec2::ZERO, CYAN, Vec2::new(-200., 0.)));
        app.world_mut().flush();
        let mut sparks = app.world_mut().query::<&Spark>();
        assert_eq!(sparks.iter(app.world()).count(), 4);
        assert!(sparks.iter(app.world()).all(|s| s.velocity.x < 0.));
    }

    #[test]
    fn a_burst_throws_sparks_that_die_out() {
        let mut app = app();
        app.world_mut().trigger(Burst::kill(Vec2::ZERO, CYAN));
        app.world_mut().flush();
        let sparks = |app: &mut App| app.world_mut().query::<&Spark>().iter(app.world()).count();
        assert_eq!(sparks(&mut app), 16);
        for _ in 0..10 {
            app.update();
        }
        let far = app
            .world_mut()
            .query_filtered::<&Transform, With<Spark>>()
            .iter(app.world())
            .all(|t| t.translation.truncate().length() > 10.);
        assert!(far, "the sparks fly out");
        for _ in 0..20 {
            app.update();
        }
        assert_eq!(sparks(&mut app), 0);
    }
}
