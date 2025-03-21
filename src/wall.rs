use avian2d::prelude::*;
use bevy::prelude::*;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_border)
            .add_systems(Update, update_border_on_resize);
    }
}

#[derive(Component)]
pub struct Border;

pub fn spawn_border(mut commands: Commands, windows: Query<&Window>) {
    let window = windows.single();
    let wall_thickness = 50.0;

    let (width, height) = (window.width(), window.height());

    // Top wall
    commands.spawn((
        Border,
        RigidBody::Static,
        Collider::rectangle(width, wall_thickness),
        Restitution::new(1.0),
        LinearDamping(0.),
        AngularDamping(0.),
        Transform::from_xyz(0.0, height / 2.0 + wall_thickness / 2.0, 0.0),
    ));

    // Bottom wall
    commands.spawn((
        Border,
        RigidBody::Static,
        Collider::rectangle(width, wall_thickness),
        Restitution::new(1.0),
        LinearDamping(0.),
        AngularDamping(0.),
        Transform::from_xyz(0.0, -height / 2.0 - wall_thickness / 2.0, 0.0),
    ));

    // Left wall
    commands.spawn((
        Border,
        RigidBody::Static,
        Collider::rectangle(wall_thickness, height),
        Restitution::new(1.0),
        LinearDamping(0.),
        AngularDamping(0.),
        Transform::from_xyz(-width / 2.0 - wall_thickness / 2.0, 0.0, 0.0),
    ));

    // Right wall
    commands.spawn((
        Border,
        RigidBody::Static,
        Collider::rectangle(wall_thickness, height),
        Restitution::new(1.0),
        LinearDamping(0.),
        AngularDamping(0.),
        Transform::from_xyz(width / 2.0 + wall_thickness / 2.0, 0.0, 0.0),
    ));
}

fn update_border_on_resize(
    mut borders: Query<(&Border, &mut Transform, &mut Collider)>,
    windows: Query<&Window, Changed<Window>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let wall_thickness = 50.0;
    let width = window.width();
    let height = window.height();

    for (_, mut transform, mut collider) in borders.iter_mut() {
        // Determine which wall this is based on position
        if transform.translation.y > 0.0 {
            // Top wall
            *collider = Collider::rectangle(width, wall_thickness);
            transform.translation.y = height / 2.0 + wall_thickness / 2.0;
        } else if transform.translation.y < 0.0 {
            // Bottom wall
            *collider = Collider::rectangle(width, wall_thickness);
            transform.translation.y = -height / 2.0 - wall_thickness / 2.0;
        } else if transform.translation.x < 0.0 {
            // Left wall
            *collider = Collider::rectangle(wall_thickness, height);
            transform.translation.x = -width / 2.0 - wall_thickness / 2.0;
        } else {
            // Right wall
            *collider = Collider::rectangle(wall_thickness, height);
            transform.translation.x = width / 2.0 + wall_thickness / 2.0;
        }
    }
}
