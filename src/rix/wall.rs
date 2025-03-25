use bevy::{
    color::palettes::tailwind::{GRAY_500, GRAY_600},
    prelude::*,
    window::PrimaryWindow,
};

pub const BASE_GRAY: Color = Color::srgb(0.29411766, 0.33333334, 0.3882353);
pub const NEON_GRAY: Color = Color::srgb(1.0241176, 1.0533333, 1.1082353);

pub const WALL_THICKNESS: f32 = 15.;
pub const WALL_MARGIN: f32 = 20.;
pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WallEvent>()
            .add_systems(Startup, spawn_walls)
            .add_systems(Update, (resize_walls, handle_wall_events));
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallCorners {
    pub top_right: Vec2,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub bottom_left: Vec2,
}

#[derive(Event)]
pub enum WallEvent {
    Reset,
    Bloom,
}

pub fn spawn_walls(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    let wall_material = materials.add(Color::from(GRAY_600));

    // Top
    commands.spawn((
        Wall {
            side: WallSide::Top,
        },
        Mesh2d(meshes.add(Rectangle::new(playable_width, WALL_THICKNESS))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, half_playable_height - WALL_THICKNESS / 2.0, 0.0),
    ));

    // Right
    commands.spawn((
        Wall {
            side: WallSide::Right,
        },
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, playable_height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(half_playable_width - WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));

    // Bottom
    commands.spawn((
        Wall {
            side: WallSide::Bottom,
        },
        Mesh2d(meshes.add(Rectangle::new(playable_width, WALL_THICKNESS))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(0.0, -half_playable_height + WALL_THICKNESS / 2.0, 0.0),
    ));

    // Left
    commands.spawn((
        Wall {
            side: WallSide::Left,
        },
        Mesh2d(meshes.add(Rectangle::new(WALL_THICKNESS, playable_height))),
        MeshMaterial2d(wall_material.clone()),
        Transform::from_xyz(-half_playable_width + WALL_THICKNESS / 2.0, 0.0, 0.0),
    ));
}

fn resize_walls(
    mut borders: Query<(&Wall, &mut Transform, &mut Mesh2d)>,
    windows: Query<&Window, Changed<Window>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    for (wall, mut transform, mut mesh2d) in &mut borders {
        match wall.side {
            WallSide::Top => {
                transform.translation.x = 0.0;
                transform.translation.y = half_playable_height - WALL_THICKNESS / 2.0;
                mesh2d.0 = meshes.add(Rectangle::new(playable_width, WALL_THICKNESS));
            }
            WallSide::Right => {
                transform.translation.x = half_playable_width - WALL_THICKNESS / 2.0;
                transform.translation.y = 0.0;
                mesh2d.0 = meshes.add(Rectangle::new(WALL_THICKNESS, playable_height));
            }
            WallSide::Bottom => {
                transform.translation.x = 0.0;
                transform.translation.y = -half_playable_height + WALL_THICKNESS / 2.0;
                mesh2d.0 = meshes.add(Rectangle::new(playable_width, WALL_THICKNESS));
            }
            WallSide::Left => {
                transform.translation.x = -half_playable_width + WALL_THICKNESS / 2.0;
                transform.translation.y = 0.0;
                mesh2d.0 = meshes.add(Rectangle::new(WALL_THICKNESS, playable_height));
            }
        }
    }
}

fn handle_wall_events(
    mut wall_events: EventReader<WallEvent>,
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
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.color = Color::from(GRAY_500);
        }
    }
}

fn reset_walls(
    walls: Query<(Entity, &Wall, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (_, _, material_handle) in &walls {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.color = BASE_GRAY;
        }
    }
}

pub fn get_world_inner_corners(window: Single<&Window, With<PrimaryWindow>>) -> WallCorners {
    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    // Calculate inner corners (accounting for wall thickness)
    let inner_right = half_playable_width - WALL_THICKNESS;
    let inner_left = -half_playable_width + WALL_THICKNESS;
    let inner_top = half_playable_height - WALL_THICKNESS;
    let inner_bottom = -half_playable_height + WALL_THICKNESS;

    WallCorners {
        top_right: Vec2::new(inner_right, inner_top),
        top_left: Vec2::new(inner_left, inner_top),
        bottom_right: Vec2::new(inner_right, inner_bottom),
        bottom_left: Vec2::new(inner_left, inner_bottom),
    }
}

pub fn get_window_inner_corners(window: Window) -> WallCorners {
    let (width, height) = (window.width(), window.height());

    let playable_width = width - 2.0 * WALL_MARGIN;
    let playable_height = height - 2.0 * WALL_MARGIN;

    let half_playable_width = playable_width / 2.0;
    let half_playable_height = playable_height / 2.0;

    // Calculate inner corners (accounting for wall thickness)
    let inner_right = half_playable_width - WALL_THICKNESS;
    let inner_left = -half_playable_width + WALL_THICKNESS;
    let inner_top = half_playable_height - WALL_THICKNESS;
    let inner_bottom = -half_playable_height + WALL_THICKNESS;

    // Convert from world coordinates to window coordinates
    let top_right = Vec2::new(inner_right + width / 2.0, height / 2.0 - inner_top);
    let top_left = Vec2::new(inner_left + width / 2.0, height / 2.0 - inner_top);
    let bottom_right = Vec2::new(inner_right + width / 2.0, height / 2.0 - inner_bottom);
    let bottom_left = Vec2::new(inner_left + width / 2.0, height / 2.0 - inner_bottom);

    WallCorners {
        top_right,
        top_left,
        bottom_right,
        bottom_left,
    }
}
