//! Stop time ([046]): an hourglass. For a while enemies and bullets stand still. A frozen
//! enemy still kills a line that touches it, and a claim still eats it.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    game::{
        AppSystems, PausableSystems,
        enemies::{Enemy, shooter::Bullet},
        state::Phase,
    },
    shape::{self, Shape},
    theme::palette::{CYAN, NEON_CYAN},
};

const STOP_SECS: f32 = 4.;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_stop
            .run_if(resource_exists::<TimeStop>)
            .in_set(AppSystems::TickTimers)
            .in_set(PausableSystems),
    );
    app.add_systems(
        FixedUpdate,
        freeze
            .run_if(resource_exists::<TimeStop>)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
    app.add_systems(OnExit(Phase::Playing), |mut commands: Commands| {
        commands.remove_resource::<TimeStop>();
    });
}

/// While it exists, time stands still; a new hourglass starts the time over.
#[derive(Resource, Deref, DerefMut)]
pub struct TimeStop(Timer);

impl Default for TimeStop {
    fn default() -> Self {
        Self(Timer::from_seconds(STOP_SECS, TimerMode::Once))
    }
}

/// The run condition of the systems that move enemies or make them act.
pub fn time_flows(stop: Option<Res<TimeStop>>) -> bool {
    stop.is_none()
}

/// What a body had before it froze.
#[derive(Component)]
struct Frozen {
    velocity: Vec2,
    locked: Option<LockedAxes>,
}

/// Locked axes keep a frozen body still under gravity and pushes. It stays dynamic, so the
/// line still meets it. Enemies that spawn during the stop freeze too.
fn freeze(
    mut commands: Commands,
    mut bodies: Query<
        (
            Entity,
            &mut LinearVelocity,
            Option<&LockedAxes>,
            Has<Frozen>,
        ),
        Or<(With<Enemy>, With<Bullet>)>,
    >,
) {
    for (entity, mut velocity, locked, frozen) in &mut bodies {
        if !frozen {
            commands.entity(entity).insert((
                Frozen {
                    velocity: velocity.0,
                    locked: locked.copied(),
                },
                LockedAxes::ALL_LOCKED,
            ));
        }
        velocity.0 = Vec2::ZERO;
    }
}

fn tick_stop(
    mut commands: Commands,
    time: Res<Time>,
    mut stop: ResMut<TimeStop>,
    mut frozen: Query<(Entity, &Frozen, &mut LinearVelocity)>,
) {
    if !stop.tick(time.delta()).is_finished() {
        return;
    }
    commands.remove_resource::<TimeStop>();
    for (entity, frozen, mut velocity) in &mut frozen {
        velocity.0 = frozen.velocity;
        let mut body = commands.entity(entity);
        body.remove::<Frozen>();
        match frozen.locked {
            Some(locked) => body.insert(locked),
            None => body.remove::<LockedAxes>(),
        };
    }
}

pub fn icon() -> Shape {
    let (w, h, waist) = (10., 13., 2.5);
    let glass = [
        Vec2::new(-w, h),
        Vec2::new(w, h),
        Vec2::new(waist, 0.),
        Vec2::new(w, -h),
        Vec2::new(-w, -h),
        Vec2::new(-waist, 0.),
    ];
    Shape::new(shape::polygon(&glass))
        .fill(CYAN)
        .stroke(NEON_CYAN, 2.)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::game::{get_enemy_layer, get_player_layer};

    fn physics_app() -> App {
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
            Duration::from_secs_f64(1. / 64.),
        ))
        .add_systems(
            FixedUpdate,
            (
                tick_stop.run_if(resource_exists::<TimeStop>),
                freeze.run_if(resource_exists::<TimeStop>),
            )
                .chain(),
        );
        app.register_required_components::<Collider, CollisionEventsEnabled>();
        app.finish();
        app
    }

    fn position(app: &App, entity: Entity) -> Vec2 {
        app.world()
            .get::<Transform>(entity)
            .unwrap()
            .translation
            .truncate()
    }

    #[test]
    fn a_stop_freezes_enemies_under_gravity_then_gives_back_their_motion() {
        let mut app = physics_app();
        let ball = app
            .world_mut()
            .spawn((
                Enemy,
                RigidBody::Dynamic,
                Collider::circle(10.),
                LinearVelocity(Vec2::new(100., 0.)),
                Transform::default(),
                get_enemy_layer(),
            ))
            .id();
        let turret = app
            .world_mut()
            .spawn((
                Enemy,
                RigidBody::Dynamic,
                LockedAxes::ALL_LOCKED,
                Collider::circle(10.),
                Transform::from_xyz(0., 100., 0.),
                get_enemy_layer(),
            ))
            .id();
        app.insert_resource(TimeStop::default());
        app.update();
        app.update();
        let start = position(&app, ball);
        for _ in 0..(STOP_SECS * 64. - 8.) as usize {
            app.update();
        }
        assert!(
            position(&app, ball).distance(start) < 0.01,
            "gravity does not pull it"
        );
        assert_eq!(
            app.world().get::<LinearVelocity>(ball).unwrap().0,
            Vec2::ZERO
        );

        for _ in 0..32 {
            app.update();
        }
        assert!(app.world().get_resource::<TimeStop>().is_none());
        assert!(app.world().get::<LockedAxes>(ball).is_none());
        assert!(
            app.world().get::<LockedAxes>(turret).is_some(),
            "its own lock stays"
        );
        assert!(position(&app, ball).x > start.x + 10., "it goes on");
    }

    #[test]
    fn a_frozen_enemy_still_meets_the_line() {
        let mut app = physics_app();
        app.insert_resource(TimeStop::default());
        app.world_mut().spawn((
            Enemy,
            RigidBody::Dynamic,
            Collider::circle(10.),
            Transform::default(),
            get_enemy_layer(),
        ));
        for _ in 0..4 {
            app.update();
        }
        app.world_mut().spawn((
            Collider::rectangle(100., 4.),
            Transform::default(),
            get_player_layer(),
        ));
        let mut hits = 0;
        for _ in 0..4 {
            app.update();
            hits += app
                .world()
                .resource::<Messages<CollisionStart>>()
                .iter_current_update_messages()
                .count();
        }
        assert!(hits > 0);
    }
}
