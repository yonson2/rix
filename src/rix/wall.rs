use avian2d::prelude::*;
use bevy::prelude::*;

use crate::colors::{BASE_GRAY, NEON_GRAY};

use super::{
    despawn_screen,
    state::{AppState, IsPaused},
};

pub const WALL_THICKNESS: f32 = 35.;
pub const WALL_THICKNESS_TOP: f32 = 65.;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<WallEvent>()
            .insert_resource(PlayableArea::default())
            .add_systems(OnEnter(AppState::InGame), spawn_walls)
            .add_systems(OnEnter(AppState::GameOver), despawn_screen::<Wall>)
            .add_systems(OnEnter(AppState::Menu), despawn_screen::<Wall>)
            .add_systems(
                Update,
                (resize_walls, handle_wall_events).run_if(in_state(IsPaused::Running)),
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
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut area: ResMut<PlayableArea>,
    walls: Query<&Wall>,
) {
    if !walls.is_empty() {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let (width, height) = (window.width(), window.height());

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
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(-half_width + WALL_THICKNESS / 2.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::from(Rectangle::new(WALL_THICKNESS, height)),
    ));
}

fn resize_walls(
    mut borders: Query<(&Wall, &mut Transform, &mut Mesh2d)>,
    windows: Query<&Window, Changed<Window>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut area: ResMut<PlayableArea>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let (width, height) = (window.width(), window.height());

    // Calculate playable area accounting for different wall thicknesses
    area.0 = Rectangle::new(width, height).area()
        - Rectangle::new(width, WALL_THICKNESS_TOP).area()  // Top wall
        - Rectangle::new(width, WALL_THICKNESS).area() * 3.; // Other walls

    let half_width = width / 2.0;
    let half_height = height / 2.0;

    for (wall, mut transform, mut mesh2d) in &mut borders {
        match wall.side {
            WallSide::Top => {
                transform.translation.x = 0.0;
                transform.translation.y = half_height - WALL_THICKNESS_TOP / 2.0;
                mesh2d.0 = meshes.add(Rectangle::new(width, WALL_THICKNESS_TOP));
            }
            WallSide::Right => {
                transform.translation.x = half_width - WALL_THICKNESS / 2.0;
                transform.translation.y = 0.0;
                mesh2d.0 = meshes.add(Rectangle::new(WALL_THICKNESS, height));
            }
            WallSide::Bottom => {
                transform.translation.x = 0.0;
                transform.translation.y = -half_height + WALL_THICKNESS / 2.0;
                mesh2d.0 = meshes.add(Rectangle::new(width, WALL_THICKNESS));
            }
            WallSide::Left => {
                transform.translation.x = -half_width + WALL_THICKNESS / 2.0;
                transform.translation.y = 0.0;
                mesh2d.0 = meshes.add(Rectangle::new(WALL_THICKNESS, height));
            }
        }
    }
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
pub fn get_world_inner_corners(window: &Window) -> WallCorners {
    let (width, height) = (window.width(), window.height());

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
    use bevy::window::WindowResolution;

    use super::*;

    const SIDES: [WallSide; 4] = [
        WallSide::Top,
        WallSide::Right,
        WallSide::Bottom,
        WallSide::Left,
    ];

    fn corners() -> WallCorners {
        let window = Window {
            resolution: WindowResolution::new(625, 625),
            ..default()
        };
        get_world_inner_corners(&window)
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
                top_right: Vec2::new(277.5, 247.5),
                top_left: Vec2::new(-277.5, 247.5),
                bottom_right: Vec2::new(277.5, -277.5),
                bottom_left: Vec2::new(-277.5, -277.5),
            }
        );
    }
}
