use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    colors::{GREEN, NEON_GREEN},
    rix::{
        RixComponent,
        enemies::{Enemy, Transition},
        get_enemy_layer,
        setup::LevelInfo,
        state::IsPaused,
        wall::WALL_THICKNESS,
    },
    utils::{ColorTransition, update_mesh_color},
};

const SQUARE_SIZE: f32 = 80.;
const SQUARE_COLOR: Color = GREEN;
const SQUARE_NEON_COLOR: Color = NEON_GREEN;
const TRANSITION_DURATION: f32 = 1.8;
const SPARK_TRANSITION_DURATION: f32 = 0.2;

pub struct SquarePlugin;

impl Plugin for SquarePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                flicker_square,
                update_mesh_color::<Square>,
                handle_collision,
            )
                .run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
pub struct Square;

pub fn spawn_square(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut level_info: ResMut<LevelInfo>,
) {
    let mut transparent_color = SQUARE_COLOR;
    transparent_color.set_alpha(0.0);

    let position = generate_valid_position(*window, &mut level_info);

    commands.spawn((
        Square,
        Enemy,
        RixComponent,
        Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE))),
        MeshMaterial2d(materials.add(transparent_color)),
        Transform::from_xyz(position.x, position.y, 1.0),
        RigidBody::Dynamic,
        GravityScale(2.5),
        Mass(5.0),
        Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
        LinearVelocity(Vec2::ZERO),
        Restitution::new(0.5).with_combine_rule(CoefficientCombine::Average),
        Transition(Timer::from_seconds(TRANSITION_DURATION, TimerMode::Once)),
        get_enemy_layer(),
    ));
}

fn flicker_square(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transition, &MeshMaterial2d<ColorMaterial>), With<Square>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transition, mesh_material) in &mut query {
        transition.0.tick(time.delta());

        let t = transition.0.fraction();

        let easing = EasingCurve::new(0.0, 1.0, EaseFunction::BounceInOut);
        let alpha = easing.sample(t).unwrap();

        let mut current_color = SQUARE_COLOR;
        current_color.set_alpha(alpha);

        if let Some(material) = materials.get_mut(&mesh_material.0) {
            material.color = current_color;
        }

        if transition.0.finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

fn handle_collision(
    mut collision_events: EventReader<Collision>,
    mut square_query: Query<(Entity, &mut GravityScale), With<Square>>,
    mut commands: Commands,
) {
    for Collision(contacts) in collision_events.read() {
        let entities = [contacts.entity1, contacts.entity2];
        for entity in entities {
            if let Ok((square_ent, mut gravity)) = square_query.get_mut(entity) {
                if let Some(mut ec) = commands.get_entity(square_ent) {
                    ec.try_insert(ColorTransition {
                        timer: Timer::from_seconds(SPARK_TRANSITION_DURATION, TimerMode::Once),
                        from: SQUARE_COLOR,
                        to: SQUARE_NEON_COLOR,
                    });
                    gravity.0 *= -1.;
                }
            }
        }
    }
}

fn generate_valid_position(window: &Window, level_info: &mut LevelInfo) -> Vec2 {
    let (width, height) = (window.width(), window.height());
    let half_size = SQUARE_SIZE / 2.0;
    let margin = WALL_THICKNESS;

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let min_x = -half_width + margin + half_size;
    let max_x = half_width - margin - half_size;
    let min_y = -half_height + margin + half_size;
    let max_y = half_height - margin - half_size;
    let x = min_x + level_info.rng.f32() * (max_x - min_x);
    let y = min_y + level_info.rng.f32() * (max_y - min_y);

    Vec2::new(x, y)
}
