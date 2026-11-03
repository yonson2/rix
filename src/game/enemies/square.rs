use avian2d::prelude::*;
use bevy::{prelude::*, time::Time};

use crate::{
    common::Transition,
    game::{
        AppSystems, PausableSystems, enemies::Enemy, events::DespawnEnemy, get_enemy_layer,
        setup::GameRng, state::Phase, wall::Field,
    },
    theme::palette::{GREEN, NEON_GREEN},
    utils::{ColorTransition, generate_enemy_position, update_mesh_color},
};

pub const SQUARE_SIZE: f32 = 40.;
const SQUARE_COLOR: Color = GREEN;
const SQUARE_NEON_COLOR: Color = NEON_GREEN;
const TRANSITION_DURATION: f32 = 3.;
const SPARK_TRANSITION_DURATION: f32 = 0.2;
const PULSE_MIN_SCALE: f32 = 1.;
const PULSE_MAX_SCALE: f32 = 2.;
const PULSE_SPEED: f32 = 1.5;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (flicker_square, handle_collision, pulse_square)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(
        Update,
        update_mesh_color::<Square>
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct Square;

#[derive(Component)]
struct PulseAnimation {
    timer: Timer,
    direction: PulseDirection,
}

#[derive(Clone, Copy)]
enum PulseDirection {
    Expanding,
    Contracting,
}

pub fn spawn_square(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    spatial_query: SpatialQuery,
    field: Res<Field>,
    mut rng: ResMut<GameRng>,
) {
    let mut transparent_color = SQUARE_COLOR;
    transparent_color.set_alpha(0.0);

    let Some(position) = generate_valid_position(&mut rng, spatial_query, &field) else {
        return;
    };
    let dir = if rng.bool() { 1. } else { -1. };

    commands.spawn((
        Square,
        Enemy,
        DespawnOnExit(Phase::Playing),
        Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE))),
        MeshMaterial2d(materials.add(transparent_color)),
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(2.5 * dir),
        Mass(15.0),
        Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
        LinearVelocity(Vec2::ZERO),
        Restitution::new(0.8).with_combine_rule(CoefficientCombine::Average),
        Transition(Timer::from_seconds(
            TRANSITION_DURATION,
            TimerMode::Repeating,
        )),
        get_enemy_layer(),
    ));
}

fn flicker_square(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Transition,
            &mut Transform,
            &MeshMaterial2d<ColorMaterial>,
            &Collider,
        ),
        With<Square>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
    spatial_query: SpatialQuery,
    field: Res<Field>,
    mut rng: ResMut<GameRng>,
) {
    for (entity, mut transition, mut transform, mesh_material, collider) in &mut query {
        transition.0.tick(time.delta());

        let t = transition.0.fraction();

        let easing = EasingCurve::new(0.0, 1.0, EaseFunction::BounceInOut);
        let alpha = 1. - easing.sample(t).unwrap();

        let mut current_color = SQUARE_COLOR;
        current_color.set_alpha(alpha);

        if let Some(mut material) = materials.get_mut(&mesh_material.0) {
            material.color = current_color;
        }

        if transition.0.is_finished() {
            match generate_enemy_position(&spatial_query, &mut rng, collider, &field) {
                Some(pos) => *transform = Transform::from_xyz(pos.x, pos.y, 1.0),
                // No room left to teleport to: the claims crushed it (todo.md).
                None => commands.trigger(DespawnEnemy(entity)),
            }
        }
    }
}

fn handle_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionStart>,
    mut square_query: Query<(Entity, &mut GravityScale), With<Square>>,
    pulse_query: Query<(), With<PulseAnimation>>,
) {
    for CollisionStart {
        collider1: entity1,
        collider2: entity2,
        ..
    } in collision_events.read()
    {
        let entities = [*entity1, *entity2];
        for entity in entities {
            if let Ok((square_ent, mut gravity)) = square_query.get_mut(entity)
                && let Ok(mut ec) = commands.get_entity(square_ent)
            {
                ec.try_insert(ColorTransition {
                    timer: Timer::from_seconds(SPARK_TRANSITION_DURATION, TimerMode::Once),
                    from: SQUARE_COLOR,
                    to: SQUARE_NEON_COLOR,
                });

                if pulse_query.get(square_ent).is_err() {
                    ec.try_insert(PulseAnimation {
                        timer: Timer::from_seconds(PULSE_SPEED, TimerMode::Once),
                        direction: PulseDirection::Expanding,
                    });
                }

                gravity.0 *= -1.;
            }
        }
    }
}

fn pulse_square(
    time: Res<Time>,
    mut commands: Commands,
    mut square_query: Query<(Entity, &mut Transform, &mut PulseAnimation)>,
) {
    for (entity, mut transform, mut pulse_anim) in square_query.iter_mut() {
        pulse_anim.timer.tick(time.delta());

        let progress = pulse_anim.timer.fraction();

        let scale_factor = match pulse_anim.direction {
            PulseDirection::Expanding => lerp(PULSE_MIN_SCALE, PULSE_MAX_SCALE, progress),
            PulseDirection::Contracting => lerp(PULSE_MAX_SCALE, PULSE_MIN_SCALE, progress),
        };

        transform.scale = Vec3::splat(scale_factor);

        if pulse_anim.timer.is_finished() {
            match pulse_anim.direction {
                PulseDirection::Expanding => {
                    pulse_anim.direction = PulseDirection::Contracting;
                    pulse_anim.timer.reset();
                }
                PulseDirection::Contracting => {
                    commands.entity(entity).remove::<PulseAnimation>();
                }
            }
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn generate_valid_position(
    rng: &mut fastrand::Rng,
    spatial_query: SpatialQuery,
    field: &Field,
) -> Option<Vec2> {
    let collider = Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE);

    generate_enemy_position(&spatial_query, rng, &collider, field)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    #[derive(Resource, Default)]
    struct Deaths(usize);

    /// A physics world, with a static block over the whole field when `full`.
    fn field(full: bool) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            bevy::mesh::MeshPlugin,
            TransformPlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(GameRng::new(1))
        .init_resource::<Field>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<Deaths>();
        app.finish();
        if full {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::rectangle(450., 800.),
                Transform::default(),
            ));
        }
        app.update();
        app
    }

    fn spot(app: &mut App) -> Option<Vec2> {
        app.world_mut()
            .run_system_once(
                |query: SpatialQuery, mut rng: ResMut<GameRng>, field: Res<Field>| {
                    let collider = Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE);
                    generate_enemy_position(&query, &mut rng, &collider, &field)
                },
            )
            .unwrap()
    }

    #[test]
    fn a_full_field_has_no_spot_and_a_square_that_cannot_teleport_dies() {
        assert!(spot(&mut field(false)).is_some());
        let mut app = field(true);
        assert_eq!(spot(&mut app), None);

        let world = app.world_mut();
        world.add_observer(|_: On<DespawnEnemy>, mut deaths: ResMut<Deaths>| deaths.0 += 1);
        world.spawn((
            Square,
            Transition(Timer::from_seconds(
                TRANSITION_DURATION,
                TimerMode::Repeating,
            )),
            Transform::default(),
            MeshMaterial2d::<ColorMaterial>::default(),
            Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
        ));
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(TRANSITION_DURATION));
        world.run_system_once(flicker_square).unwrap();
        world.flush();
        assert_eq!(world.resource::<Deaths>().0, 1);
    }
}
