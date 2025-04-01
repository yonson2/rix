use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    colors::{GREEN, LIGHT_GREEN, NEON_GREEN},
    rix::{
        RixComponent,
        enemies::{Enemy, Transition},
        state::{AppState, IsPaused},
    },
    utils::{ColorTransition, update_mesh_color},
};

const SQUARE_SIZE: f32 = 80.;
const SQUARE_COLOR: Color = GREEN;
const SQUARE_NEON_COLOR: Color = NEON_GREEN;
const TRANSITION_DURATION: f32 = 1.8;
const COLOR_TRANSITION_DURATION: f32 = 2.6;

pub struct SquarePlugin;

impl Plugin for SquarePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_square);
        app.add_systems(
            Update,
            (flicker_square, update_mesh_color::<Square>).run_if(in_state(IsPaused::Running)),
        );
    }
}

#[derive(Component)]
pub struct Square;

fn spawn_square(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut transparent_color = SQUARE_COLOR;
    transparent_color.set_alpha(0.0);

    let mut color_transition = ColorTransition {
        timer: Timer::from_seconds(COLOR_TRANSITION_DURATION, TimerMode::Repeating),
        from: SQUARE_COLOR,
        to: SQUARE_NEON_COLOR,
    };
    color_transition.timer.pause();

    commands.spawn((
        Square,
        Enemy,
        RixComponent,
        Mesh2d(meshes.add(Rectangle::new(SQUARE_SIZE, SQUARE_SIZE))),
        MeshMaterial2d(materials.add(transparent_color)),
        Transform::from_xyz(0.0, 0.0, 1.0),
        RigidBody::Dynamic,
        Collider::rectangle(SQUARE_SIZE, SQUARE_SIZE),
        LinearVelocity(Vec2::ZERO),
        AngularDamping(0.),
        Restitution::new(1.).with_combine_rule(CoefficientCombine::Max),
        LinearDamping(0.),
        Transition(Timer::from_seconds(TRANSITION_DURATION, TimerMode::Once)),
        color_transition,
    ));
}

fn flicker_square(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Transition,
            &mut ColorTransition,
            &MeshMaterial2d<ColorMaterial>,
        ),
        With<Square>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transition, mut color_transition, mesh_material) in &mut query {
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
            color_transition.timer.unpause();
        }
    }
}
