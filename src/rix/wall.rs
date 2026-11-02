use avian2d::prelude::*;
use bevy::prelude::*;

use crate::colors::{BASE_GRAY, NEON_GRAY};

use super::state::{IsPaused, Screen};

/// The playfield in world units, walls included: one fixed 9:16 field on every device
/// (D17). The camera fits it into any window ([019]).
pub const FIELD: Vec2 = Vec2::new(450., 800.);
pub const WALL_THICKNESS: f32 = 35.;
pub const WALL_THICKNESS_TOP: f32 = 65.;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WallEvent>()
            .insert_resource(PlayableArea::default())
            .add_systems(OnEnter(Screen::Gameplay), spawn_walls)
            .add_systems(
                Update,
                handle_wall_events.run_if(in_state(IsPaused::Running)),
            );
    }
}

#[derive(Component)]
pub struct Wall {
    pub side: WallSide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallCorners {
    pub top_right: Vec2,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}
#[derive(Resource, Clone, Copy, Debug, PartialEq, Deref, DerefMut, Default)]
pub struct PlayableArea(pub f32);

#[derive(Message)]
pub enum WallEvent {
    Reset,
    Bloom,
}

impl WallSide {
    pub fn next_wall(&self, clockwise: bool) -> Self {
        if clockwise {
            match self {
                WallSide::Top => WallSide::Right,
                WallSide::Right => WallSide::Bottom,
                WallSide::Bottom => WallSide::Left,
                WallSide::Left => WallSide::Top,
            }
        } else {
            match self {
                WallSide::Top => WallSide::Left,
                WallSide::Right => WallSide::Top,
                WallSide::Bottom => WallSide::Right,
                WallSide::Left => WallSide::Bottom,
            }
        }
    }

    pub fn next_corner(&self, clockwise: bool, corners: &WallCorners) -> Vec2 {
        if clockwise {
            match self {
                WallSide::Top => corners.top_right,
                WallSide::Right => corners.bottom_right,
                WallSide::Bottom => corners.bottom_left,
                WallSide::Left => corners.top_left,
            }
        } else {
            match self {
                WallSide::Top => corners.top_left,
                WallSide::Right => corners.top_right,
                WallSide::Bottom => corners.bottom_right,
                WallSide::Left => corners.bottom_left,
            }
        }
    }
}

pub fn spawn_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut area: ResMut<PlayableArea>,
) {
    let (width, height) = (FIELD.x, FIELD.y);

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    let wall_material = materials.add(BASE_GRAY);

    // Calculate playable area accounting for different wall thicknesses
    area.0 = Rectangle::new(width, height).area()
        - Rectangle::new(width, WALL_THICKNESS_TOP).area()  // Top wall
        - Rectangle::new(width, WALL_THICKNESS).area() * 3.; // Other walls

    // Top
    commands.spawn((
        Wall {
            side: WallSide::Top,
        },
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(width, WALL_THICKNESS_TOP))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, half_height - WALL_THICKNESS_TOP / 2.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(width, WALL_THICKNESS_TOP)),
    ));

    // Right
    commands.spawn((
        Wall {
            side: WallSide::Right,
        },
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(half_width - WALL_THICKNESS / 2.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(WALL_THICKNESS, height)),
    ));

    // Bottom
    commands.spawn((
        Wall {
            side: WallSide::Bottom,
        },
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(meshes.add(Rectangle::new(width, WALL_THICKNESS))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, -half_height + WALL_THICKNESS / 2.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(width, WALL_THICKNESS)),
    ));

    // Left
    commands.spawn((
        Wall {
            side: WallSide::Left,
        },
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

/// Convert world coordinates to window coordinates
fn world_to_window_coords(world_pos: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        world_pos.x + window_size.x / 2.0,
        window_size.y / 2.0 - world_pos.y,
    )
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

/// Get wall corners in window coordinates
///
/// This function first computes corners in world coordinates,
/// then converts them to window coordinates
pub fn get_window_inner_corners(window: Window) -> WallCorners {
    let (width, height) = (window.width(), window.height());
    let window_size = Vec2::new(width, height);

    // Get world coordinates first
    let world_corners = {
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
    };

    // Convert to window coordinates
    WallCorners {
        top_right: world_to_window_coords(world_corners.top_right, window_size),
        top_left: world_to_window_coords(world_corners.top_left, window_size),
        bottom_right: world_to_window_coords(world_corners.bottom_right, window_size),
        bottom_left: world_to_window_coords(world_corners.bottom_left, window_size),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDES: [WallSide; 4] = [
        WallSide::Top,
        WallSide::Right,
        WallSide::Bottom,
        WallSide::Left,
    ];

    fn corners() -> WallCorners {
        get_world_inner_corners()
    }

    #[test]
    fn next_wall_cycles_both_ways() {
        for (i, side) in SIDES.iter().enumerate() {
            assert_eq!(side.next_wall(true), SIDES[(i + 1) % 4]);
            assert_eq!(side.next_wall(false), SIDES[(i + 3) % 4]);
        }
    }

    #[test]
    fn next_corner_is_the_end_of_the_wall_in_that_direction() {
        let c = corners();
        let clockwise = [c.top_right, c.bottom_right, c.bottom_left, c.top_left];
        let counter_clockwise = [c.top_left, c.top_right, c.bottom_right, c.bottom_left];
        for (i, side) in SIDES.iter().enumerate() {
            assert_eq!(side.next_corner(true, &c), clockwise[i]);
            assert_eq!(side.next_corner(false, &c), counter_clockwise[i]);
        }
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
