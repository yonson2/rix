//! The Shooter ([045]): a fixed turret that fires a slow bullet every second, with or
//! without a line ([128]). With no line each shot picks a random way; while a line is
//! drawn the barrel turns toward the finger. A bullet that meets the line kills; walls and
//! claims stop bullets. Bullets are not enemies: claims do not eat them and they pass
//! enemies.

use std::{f32::consts::TAU, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    audio::Sfx,
    game::{
        AppSystems, GameLayer, PausableSystems,
        enemies::Enemy,
        events, get_enemy_layer,
        items::time_stop::time_flows,
        polygon::TempLineSegment,
        setup::{GameRng, by_place},
        state::Phase,
        user_input::UserPath,
        wall::Field,
    },
    shape::{self, Shape},
    theme::palette::{NEON_ORANGE, ORANGE},
    utils::generate_enemy_position,
};

const RADIUS: f32 = 16.;
const BARREL: Vec2 = Vec2::new(12., 4.);
/// One shot every `FIRE_SECS`, with or without a line ([128]).
const FIRE_SECS: f32 = 1.;
/// How fast the barrel turns, in radians a second: half a turn between two shots.
const TURN_SPEED: f32 = TAU / 2.;
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

/// The time to the next shot, where the barrel points, the random way it turns to with no
/// line, and the last finger position.
#[derive(Component)]
struct Gun {
    timer: Timer,
    heading: Vec2,
    idle: Vec2,
    finger: Vec2,
}

impl Gun {
    /// The timer starts at `phase` (0 to 1 of `FIRE_SECS`), so turrets do not fire in step.
    fn new(angle: f32, phase: f32, finger: Vec2) -> Self {
        let mut timer = Timer::from_seconds(FIRE_SECS, TimerMode::Repeating);
        timer.set_elapsed(Duration::from_secs_f32(phase * FIRE_SECS));
        Self {
            timer,
            heading: Vec2::from_angle(angle),
            idle: Vec2::from_angle(angle),
            finger,
        }
    }
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
    let angle = rng.f32() * TAU;
    let phase = rng.f32();
    commands.spawn((
        Shooter,
        Enemy,
        DespawnOnExit(Phase::Playing),
        look(),
        collider,
        Transform::from_xyz(position.x, position.y, 1.).with_rotation(Quat::from_rotation_z(angle)),
        // Dynamic, so the line (colliders with no body) meets it; nothing moves it.
        RigidBody::Dynamic,
        GravityScale(0.),
        LockedAxes::ALL_LOCKED,
        Gun::new(angle, phase, position),
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

/// The barrel turns toward the finger while a line is drawn, else toward its random way;
/// it fires where it points. A shot with no line picks the next random way.
fn aim_and_fire(
    mut commands: Commands,
    time: Res<Time>,
    path: Res<UserPath>,
    mut rng: ResMut<GameRng>,
    mut fingers: MessageReader<events::InputPosition>,
    mut guns: Query<(&mut Gun, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let finger = fingers.read().last().map(|p| p.0);
    let mut guns: Vec<_> = guns.iter_mut().collect();
    guns.sort_by(|a, b| by_place(a.1.translation.truncate(), b.1.translation.truncate()));
    for (mut gun, mut transform) in guns {
        let position = transform.translation.truncate();
        if let Some(finger) = finger {
            gun.finger = finger;
        }
        let target = match path.is_drawing {
            true => (gun.finger - position).normalize_or(gun.heading),
            false => gun.idle,
        };
        gun.heading = gun
            .heading
            .rotate_towards(target, TURN_SPEED * time.delta_secs());
        transform.rotation = Quat::from_rotation_z(gun.heading.to_angle());
        if !gun.timer.tick(time.delta()).just_finished() {
            continue;
        }
        if !path.is_drawing {
            gun.idle = Vec2::from_angle(rng.f32() * TAU);
        }
        let muzzle = position + gun.heading * (RADIUS + BARREL.x * 2.);
        commands.spawn((
            bullet(muzzle, gun.heading),
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

    use bevy::ecs::system::RunSystemOnce;

    /// The fixed tick, 64 a second.
    const TICK: Duration = Duration::from_micros(15_625);

    /// A gun at the origin that points along x, its timer at 0.
    fn gun_world() -> World {
        let mut world = World::new();
        world.init_resource::<Time>();
        world.init_resource::<UserPath>();
        world.init_resource::<Messages<events::InputPosition>>();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<ColorMaterial>>();
        world.insert_resource(GameRng::new(1));
        world.spawn((Gun::new(0., 0., Vec2::ZERO), Transform::default()));
        world
    }

    /// Runs `ticks` fixed ticks; returns the ticks with a shot and the shots' ways.
    fn run_gun(world: &mut World, ticks: usize) -> (Vec<usize>, Vec<Vec2>) {
        let mut fired = Vec::new();
        let mut ways = Vec::new();
        for tick in 0..ticks {
            world.resource_mut::<Time>().advance_by(TICK);
            world.run_system_once(aim_and_fire).unwrap();
            let mut bullets = world.query_filtered::<(Entity, &LinearVelocity), With<Bullet>>();
            let shots: Vec<_> = bullets.iter(world).map(|(e, v)| (e, v.0)).collect();
            for (bullet, velocity) in shots {
                fired.push(tick);
                ways.push(velocity / BULLET_SPEED);
                world.despawn(bullet);
            }
        }
        (fired, ways)
    }

    fn heading(world: &mut World) -> Vec2 {
        world.query::<&Gun>().single(world).unwrap().heading
    }

    #[test]
    fn the_gun_fires_every_second_with_or_without_a_line() {
        let mut world = gun_world();
        assert_eq!(run_gun(&mut world, 128).0, [63, 127]);
        world.resource_mut::<UserPath>().is_drawing = true;
        assert_eq!(run_gun(&mut world, 64).0, [63], "a line does not reset it");
    }

    #[test]
    fn with_no_line_each_shot_goes_another_way() {
        let mut world = gun_world();
        let (_, ways) = run_gun(&mut world, 64 * 4);
        assert_eq!(ways.len(), 4);
        assert_eq!(ways[0], Vec2::X, "the first way: the spawn angle");
        for (i, a) in ways.iter().enumerate() {
            assert!((a.length() - 1.).abs() < 1e-4);
            for b in &ways[i + 1..] {
                assert!(a.distance(*b) > 1e-3, "{a} and {b}");
            }
        }
    }

    #[test]
    fn while_a_line_is_drawn_the_barrel_turns_to_the_finger() {
        let mut world = gun_world();
        world.resource_mut::<UserPath>().is_drawing = true;
        world.write_message(events::InputPosition(Vec2::new(0., 100.)));
        run_gun(&mut world, 1);
        let step = TURN_SPEED * TICK.as_secs_f32();
        assert!((heading(&mut world).to_angle() - step).abs() < 1e-4);
        // A quarter turn takes half a second; the shot at one second goes up.
        let (fired, ways) = run_gun(&mut world, 63);
        assert_eq!(fired, [62]);
        assert!(ways[0].distance(Vec2::Y) < 1e-3, "{}", ways[0]);
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
