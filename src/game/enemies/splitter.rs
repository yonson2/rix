//! The Splitter ([044], `docs/design/splitter.md`): a Queen octagon wanders in straight
//! lines. When she dies (eaten or bombed), two faster Soldiers hatch at free spots.

use std::f32::consts::{PI, TAU};

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    audio::Sfx,
    game::{
        AppSystems, PausableSystems,
        enemies::Enemy,
        get_enemy_layer,
        items::time_stop::time_flows,
        setup::{GameRng, by_place},
        state::Phase,
        wall::Field,
    },
    shape::{self, Shape},
    theme::palette::{LIGHT_PURPLE, NEON_PURPLE},
    utils::generate_enemy_position,
};

const QUEEN_RADIUS: f32 = 18.;
const QUEEN_SPEED: f32 = 120.;
/// The Soldier ellipse's half width and half height; it points where it goes.
const SOLDIER_HALF_SIZE: Vec2 = Vec2::new(14., 8.);
const SOLDIER_SPEED: f32 = 220.;
const SOLDIERS: usize = 2;
/// A turn comes at a random time in this range of seconds.
const TURN_SECS: (f32, f32) = (1., 2.5);
/// The largest turn, either way.
const MAX_TURN: f32 = PI / 2.;
/// The Soldiers wait this long after their Queen dies: by then the claim that ate her has
/// its collider, so no Soldier hatches inside it.
const HATCH_SECS: f32 = 0.25;
const STROKE: f32 = 3.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (wander.run_if(time_flows), hatch)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Splitter {
    Queen,
    Soldier,
}

/// Straight runs at a constant speed, with a random turn at random times.
#[derive(Component)]
struct Wander {
    speed: f32,
    turn: Timer,
}

/// Two Soldiers on their way; a level that ends first takes them along.
#[derive(Component)]
pub(super) struct Hatch(Timer);

/// What a dead Queen leaves ([044]): her Soldiers, soon.
pub fn hatch_soldiers() -> impl Bundle {
    (
        Hatch(Timer::from_seconds(HATCH_SECS, TimerMode::Once)),
        DespawnOnExit(Phase::Playing),
    )
}

pub fn spawn_splitter(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    field: Res<Field>,
    mut rng: ResMut<GameRng>,
) {
    spawn(
        &mut commands,
        Splitter::Queen,
        &spatial_query,
        &field,
        &mut rng,
    );
}

fn spawn(
    commands: &mut Commands,
    kind: Splitter,
    spatial_query: &SpatialQuery,
    field: &Field,
    rng: &mut GameRng,
) {
    let (collider, speed) = match kind {
        Splitter::Queen => (Collider::regular_polygon(QUEEN_RADIUS, 8), QUEEN_SPEED),
        Splitter::Soldier => (
            Collider::ellipse(SOLDIER_HALF_SIZE.x, SOLDIER_HALF_SIZE.y),
            SOLDIER_SPEED,
        ),
    };
    let Some(position) = generate_enemy_position(spatial_query, rng, &collider, field) else {
        return;
    };
    let angle = rng.f32() * TAU;
    let turn = turn_timer(rng);
    commands.spawn((
        kind,
        Enemy,
        DespawnOnExit(Phase::Playing),
        look(kind),
        collider,
        Transform::from_xyz(position.x, position.y, 1.).with_rotation(Quat::from_rotation_z(angle)),
        RigidBody::Dynamic,
        GravityScale(0.),
        (Mass(2.), LockedAxes::ROTATION_LOCKED),
        LinearVelocity(Vec2::from_angle(angle) * speed),
        Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
        Friction::ZERO,
        Wander { speed, turn },
        get_enemy_layer(),
    ));
}

/// A queen's or a soldier's look, in play and in the wiki ([071]).
pub fn look(kind: Splitter) -> Shape {
    let outline = match kind {
        Splitter::Queen => RegularPolygon::new(QUEEN_RADIUS, 8)
            .vertices(0.)
            .into_iter()
            .collect(),
        Splitter::Soldier => ellipse(SOLDIER_HALF_SIZE),
    };
    Shape::new(shape::polygon(&outline))
        .fill(LIGHT_PURPLE)
        .stroke(NEON_PURPLE, STROKE)
}

fn ellipse(half_size: Vec2) -> Vec<Vec2> {
    (0..20)
        .map(|i| Vec2::from_angle(i as f32 / 20. * TAU) * half_size)
        .collect()
}

fn turn_timer(rng: &mut GameRng) -> Timer {
    let (min, max) = TURN_SECS;
    Timer::from_seconds(min + rng.f32() * (max - min), TimerMode::Once)
}

/// Keeps the speed after a bounce, turns at the end of each run, faces the way it goes.
fn wander(
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    mut bodies: Query<(&mut Wander, &mut LinearVelocity, &mut Transform)>,
) {
    // Two turns in one tick draw in place order, not query order ([104]).
    let mut bodies: Vec<_> = bodies.iter_mut().collect();
    bodies.sort_by(|a, b| by_place(a.2.translation.truncate(), b.2.translation.truncate()));
    for (mut wander, mut velocity, mut transform) in bodies {
        let mut angle = velocity.to_angle();
        if wander.turn.tick(time.delta()).is_finished() {
            angle += (rng.f32() * 2. - 1.) * MAX_TURN;
            wander.turn = turn_timer(&mut rng);
        }
        velocity.0 = Vec2::from_angle(angle) * wander.speed;
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn hatch(
    mut commands: Commands,
    time: Res<Time>,
    mut hatches: Query<(Entity, &mut Hatch)>,
    spatial_query: SpatialQuery,
    field: Res<Field>,
    mut rng: ResMut<GameRng>,
) {
    for (entity, mut hatch) in &mut hatches {
        if !hatch.0.tick(time.delta()).is_finished() {
            continue;
        }
        commands.entity(entity).despawn();
        commands.trigger(Sfx::Hatch);
        for _ in 0..SOLDIERS {
            spawn(
                &mut commands,
                Splitter::Soldier,
                &spatial_query,
                &field,
                &mut rng,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn a_hatch_turns_into_two_soldiers_at_free_spots() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            TransformPlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(Field::default())
        .insert_resource(GameRng::new(1));
        app.finish();
        let hatch = app
            .world_mut()
            .spawn(Hatch(Timer::from_seconds(0., TimerMode::Once)))
            .id();
        #[derive(Resource, Default)]
        struct Heard(Vec<Sfx>);
        app.init_resource::<Heard>()
            .add_observer(|sfx: On<Sfx>, mut heard: ResMut<Heard>| heard.0.push(*sfx));
        app.world_mut().run_system_once(super::hatch).unwrap();
        app.world_mut().flush();

        assert!(app.world().get_entity(hatch).is_err());
        assert_eq!(app.world().resource::<Heard>().0, [Sfx::Hatch]);
        let soldiers: Vec<Splitter> = app
            .world_mut()
            .query_filtered::<&Splitter, With<Enemy>>()
            .iter(app.world())
            .copied()
            .collect();
        assert_eq!(soldiers, [Splitter::Soldier; SOLDIERS]);
    }

    #[test]
    fn the_queen_outline_matches_her_collider() {
        let outline: Vec<Vec2> = RegularPolygon::new(QUEEN_RADIUS, 8)
            .vertices(0.)
            .into_iter()
            .collect();
        let collider = Collider::regular_polygon(QUEEN_RADIUS, 8);
        for point in outline {
            let inside = collider.contains_point(Vec2::ZERO, 0., point * 0.99);
            let outside = collider.contains_point(Vec2::ZERO, 0., point * 1.01);
            assert!(inside && !outside, "{point}");
        }
    }
}
