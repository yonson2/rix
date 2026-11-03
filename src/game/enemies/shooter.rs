//! The Shooter ([045]): a fixed turret that turns toward the finger and, while a line is
//! drawn, fires slow bullets at it. A bullet that meets the line kills; walls and claims
//! stop bullets. Bullets are not enemies: claims do not eat them and they pass enemies.

use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    audio::Sfx,
    game::{
        AppSystems, GameLayer, PausableSystems, enemies::Enemy, events, get_enemy_layer,
        items::time_stop::time_flows, polygon::TempLineSegment, setup::GameRng, state::Phase,
        user_input::UserPath, wall::Field,
    },
    shape::{self, Shape},
    theme::palette::{NEON_ORANGE, ORANGE},
    utils::generate_enemy_position,
};

const RADIUS: f32 = 16.;
const BARREL: Vec2 = Vec2::new(12., 4.);
/// The first shot comes this long after a line starts, then one every `FIRE_SECS`.
const FIRST_SHOT_SECS: f32 = 0.5;
const FIRE_SECS: f32 = 1.;
const BULLET_RADIUS: f32 = 4.;
const BULLET_SPEED: f32 = 140.;
const BULLET_Z: f32 = 1.2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (aim_and_fire.run_if(time_flows), bullet_hits)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct Shooter;

/// The time to the next shot and the last finger position.
#[derive(Component)]
struct Gun {
    timer: Timer,
    aim: Vec2,
}

#[derive(Component)]
pub struct Bullet;

fn bullet_layer() -> CollisionLayers {
    CollisionLayers::new(GameLayer::Bullet, [GameLayer::Default, GameLayer::Player])
}

pub fn spawn_shooter(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    field: Res<Field>,
    mut rng: ResMut<GameRng>,
) {
    let collider = Collider::regular_polygon(RADIUS, 6);
    let Some(position) = generate_enemy_position(&spatial_query, &mut rng, &collider, &field)
    else {
        return;
    };
    commands.spawn((
        Shooter,
        Enemy,
        DespawnOnExit(Phase::Playing),
        look(),
        collider,
        Transform::from_xyz(position.x, position.y, 1.),
        // Dynamic, so the line (colliders with no body) meets it; nothing moves it.
        RigidBody::Dynamic,
        GravityScale(0.),
        LockedAxes::ALL_LOCKED,
        Gun {
            timer: first_shot(),
            aim: position + Vec2::X,
        },
        get_enemy_layer(),
    ));
}

/// The body and the barrel, in play and in the wiki ([071]).
pub fn look() -> impl Bundle {
    let body: Vec<Vec2> = RegularPolygon::new(RADIUS, 6)
        .vertices(0.)
        .into_iter()
        .collect();
    (
        Shape::new(shape::polygon(&body))
            .fill(ORANGE)
            .stroke(NEON_ORANGE, 3.),
        children![(
            Shape::new(shape::rounded_rectangle(BARREL * 2., 2.)).fill(NEON_ORANGE),
            Transform::from_xyz(RADIUS, 0., 0.1),
        )],
    )
}

fn first_shot() -> Timer {
    Timer::from_seconds(FIRST_SHOT_SECS, TimerMode::Once)
}

/// Whether the gun fires this tick; it waits for a line and starts over after one.
fn tick_gun(timer: &mut Timer, drawing: bool, delta: Duration) -> bool {
    if !drawing {
        *timer = first_shot();
        return false;
    }
    if !timer.tick(delta).is_finished() {
        return false;
    }
    *timer = Timer::from_seconds(FIRE_SECS, TimerMode::Once);
    true
}

fn aim_and_fire(
    mut commands: Commands,
    time: Res<Time>,
    path: Res<UserPath>,
    mut fingers: MessageReader<events::InputPosition>,
    mut guns: Query<(&mut Gun, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let finger = fingers.read().last().map(|p| p.0);
    for (mut gun, mut transform) in &mut guns {
        let position = transform.translation.truncate();
        if let Some(finger) = finger {
            gun.aim = finger;
        }
        let direction = (gun.aim - position).normalize_or(Vec2::X);
        transform.rotation = Quat::from_rotation_z(direction.to_angle());
        if !tick_gun(&mut gun.timer, path.is_drawing, time.delta()) {
            continue;
        }
        let muzzle = position + direction * (RADIUS + BARREL.x * 2.);
        commands.spawn((
            bullet(muzzle, direction),
            Mesh2d(meshes.add(Circle::new(BULLET_RADIUS))),
            MeshMaterial2d(materials.add(NEON_ORANGE)),
        ));
        commands.trigger(Sfx::Shot);
    }
}

/// A bullet without its look: it flies through enemies and items.
fn bullet(position: Vec2, direction: Vec2) -> impl Bundle {
    (
        Bullet,
        DespawnOnExit(Phase::Playing),
        Transform::from_translation(position.extend(BULLET_Z)),
        RigidBody::Dynamic,
        GravityScale(0.),
        Sensor,
        Collider::circle(BULLET_RADIUS),
        LinearVelocity(direction * BULLET_SPEED),
        bullet_layer(),
    )
}

fn bullet_hits(
    mut commands: Commands,
    mut collisions: MessageReader<CollisionStart>,
    bullets: Query<(), With<Bullet>>,
    lines: Query<(), With<TempLineSegment>>,
    mut die: MessageWriter<events::Die>,
) {
    for hit in collisions.read() {
        for (bullet, other) in [
            (hit.collider1, hit.collider2),
            (hit.collider2, hit.collider1),
        ] {
            if !bullets.contains(bullet) {
                continue;
            }
            if lines.contains(other) {
                die.write(events::Die);
            }
            commands.entity(bullet).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::get_player_layer;

    #[test]
    fn the_gun_fires_half_a_second_into_a_line_then_every_second() {
        let tick = Duration::from_secs_f32(0.05);
        let mut timer = first_shot();
        let shots = |timer: &mut Timer, drawing, ticks| {
            (0..ticks)
                .filter(|_| tick_gun(timer, drawing, tick))
                .collect::<Vec<_>>()
        };
        assert_eq!(shots(&mut timer, false, 40), [] as [usize; 0]);
        assert_eq!(shots(&mut timer, true, 50), [9, 29, 49]);
        // A new line starts over: half a second again.
        assert_eq!(shots(&mut timer, false, 1), [] as [usize; 0]);
        assert_eq!(shots(&mut timer, true, 10), [9]);
    }

    /// Fires a bullet down at `target` for one second; returns the deaths and whether the
    /// bullet is gone.
    fn shoot_at(target: impl Bundle) -> (usize, bool) {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            TransformPlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f64(1. / 60.),
        ))
        .add_message::<events::Die>()
        .add_systems(FixedUpdate, bullet_hits);
        app.register_required_components::<Collider, CollisionEventsEnabled>();
        app.finish();
        app.world_mut().spawn(target);
        // An enemy between the gun and the target: bullets pass it.
        app.world_mut().spawn((
            RigidBody::Dynamic,
            GravityScale(0.),
            Collider::circle(10.),
            Transform::from_xyz(0., 60., 0.),
            get_enemy_layer(),
        ));
        let shot = app
            .world_mut()
            .spawn(bullet(Vec2::new(0., 100.), Vec2::NEG_Y))
            .id();
        let mut deaths = 0;
        for _ in 0..60 {
            app.update();
            deaths += app
                .world()
                .resource::<Messages<events::Die>>()
                .iter_current_update_messages()
                .count();
        }
        (deaths, app.world().get_entity(shot).is_err())
    }

    #[test]
    fn a_bullet_kills_on_the_line_and_stops_on_a_wall() {
        let line = (
            TempLineSegment,
            Collider::rectangle(100., 4.),
            Transform::default(),
            get_player_layer(),
        );
        assert_eq!(shoot_at(line), (1, true));
        let wall = (
            RigidBody::Static,
            Collider::rectangle(100., 10.),
            Transform::default(),
        );
        assert_eq!(shoot_at(wall), (0, true));
    }
}
