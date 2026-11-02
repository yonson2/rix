use avian2d::prelude::*;
use bevy::prelude::*;

use crate::theme::palette::{BASE_GRAY, NEON_GRAY};

use super::{AppSystems, PausableSystems};
use crate::screens::Screen;

/// The playfield in world units, walls included: one fixed 9:16 field on every device
/// (D17). The camera fits it into any window ([019]).
pub const FIELD: Vec2 = Vec2::new(450., 800.);
pub const WALL_THICKNESS: f32 = 35.;
pub const WALL_THICKNESS_TOP: f32 = 65.;

pub(super) fn plugin(app: &mut App) {
    app.add_message::<WallEvent>()
        .add_systems(OnEnter(Screen::Gameplay), spawn_walls)
        .add_systems(
            Update,
            handle_wall_events
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

#[derive(Component)]
pub struct Wall;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallCorners {
    pub top_right: Vec2,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}
#[derive(Message)]
pub enum WallEvent {
    Reset,
    Bloom,
}

pub fn spawn_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (width, height) = (FIELD.x, FIELD.y);

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let wall_material = materials.add(BASE_GRAY);

    // Top
    commands.spawn((
        Wall,
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(width, WALL_THICKNESS_TOP))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, half_height - WALL_THICKNESS_TOP / 2.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(width, WALL_THICKNESS_TOP)),
    ));

    // Right
    commands.spawn((
        Wall,
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(half_width - WALL_THICKNESS / 2.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(WALL_THICKNESS, height)),
    ));

    // Bottom
    commands.spawn((
        Wall,
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(width, WALL_THICKNESS))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, -half_height + WALL_THICKNESS / 2.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(width, WALL_THICKNESS)),
    ));

    // Left
    commands.spawn((
        Wall,
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(-half_width + WALL_THICKNESS / 2.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(WALL_THICKNESS, height)),
    ));
}

fn handle_wall_events(
    mut wall_events: MessageReader<WallEvent>,
    walls: Query<(Entity, &Wall, &MeshMaterial2d<ColorMaterial>)>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Some(event) = wall_events.read().next() {
        match *event {
            WallEvent::Bloom => bloom_walls(walls, materials),
            WallEvent::Reset => reset_walls(walls, materials),
        }
    }
}

/// brightens up the walls just enough so they start glowing.
fn bloom_walls(
    walls: Query<(Entity, &Wall, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (_, _, material_handle) in &walls {
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.color = NEON_GRAY;
        }
    }
}

fn reset_walls(
    walls: Query<(Entity, &Wall, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (_, _, material_handle) in &walls {
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.color = BASE_GRAY;
        }
    }
}

/// Get wall corners in world coordinates
pub fn get_world_inner_corners() -> WallCorners {
    let (width, height) = (FIELD.x, FIELD.y);

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let inner_right = half_width - WALL_THICKNESS;
    let inner_left = -half_width + WALL_THICKNESS;
    let inner_top = half_height - WALL_THICKNESS_TOP;
    let inner_bottom = -half_height + WALL_THICKNESS;

    WallCorners {
        top_right: Vec2::new(inner_right, inner_top),
        top_left: Vec2::new(inner_left, inner_top),
        bottom_right: Vec2::new(inner_right, inner_bottom),
        bottom_left: Vec2::new(inner_left, inner_bottom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corners() -> WallCorners {
        get_world_inner_corners()
    }

    #[test]
    fn world_inner_corners_leave_out_the_walls() {
        assert_eq!(
            corners(),
            WallCorners {
                top_right: Vec2::new(190., 335.),
                top_left: Vec2::new(-190., 335.),
                bottom_right: Vec2::new(190., -365.),
                bottom_left: Vec2::new(-190., -365.),
            }
        );
    }
}
