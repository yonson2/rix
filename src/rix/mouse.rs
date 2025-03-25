use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

use crate::rix::{
    polygon::PolygonData,
    wall::{WALL_MARGIN, WALL_THICKNESS, Wall},
};

use super::{
    polygon::events as polygon_events,
    wall::{WallEvent, WallSide},
};

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePath>();
        app.add_systems(Update, handle_click);
    }
}

#[derive(Resource, Default, Debug)]
pub struct MousePath {
    path: Vec<Vec2>,
    is_drawing: bool,
    last_position: Option<Vec2>,
    start: Option<WallSide>,
    end: Option<WallSide>,
}

impl MousePath {
    fn reset(&mut self) {
        self.path.clear();
        self.is_drawing = false;
        self.last_position = None;
    }
}

/// Convert window coordinates to world coordinates
fn convert_to_world_coords(window_pos: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        window_pos.x - window_size.x / 2.,
        window_size.y / 2. - window_pos.y,
    )
}

fn is_point_in_wall(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
    window_size: Vec2,
) -> Option<WallSide> {
    // Get wall position in world coordinates
    let wall_pos = wall_transform.translation();

    // Calculate wall dimensions based on side
    let (wall_width, wall_height) = match wall_side {
        WallSide::Top | WallSide::Bottom => {
            // Top or bottom wall (horizontal)
            let width = window_size.x - 2. * WALL_MARGIN; // Playable width
            (width, WALL_THICKNESS)
        }
        WallSide::Right | WallSide::Left => {
            // Left or right wall (vertical)
            let height = window_size.y - 2. * WALL_MARGIN; // Playable height
            (WALL_THICKNESS, height)
        }
    };

    let detection_margin = 3.;
    // Calculate wall bounds in world coordinates
    let half_width = wall_width / 2. + detection_margin;
    let half_height = wall_height / 2. + detection_margin;

    let wall_min_x = wall_pos.x - half_width;
    let wall_max_x = wall_pos.x + half_width;
    let wall_min_y = wall_pos.y - half_height;
    let wall_max_y = wall_pos.y + half_height;

    // Convert cursor position from window coordinates to world coordinates
    let world_pos = convert_to_world_coords(cursor_pos, window_size);

    // Check if point is within wall bounds
    if world_pos.x >= wall_min_x
        && world_pos.x <= wall_max_x
        && world_pos.y >= wall_min_y
        && world_pos.y <= wall_max_y
    {
        Some(wall_side)
    } else {
        None
    }
}

/// checks if adding a new point would create an intersection
fn would_create_intersection(path: &[Vec2], new_point: Vec2) -> bool {
    if path.len() < 3 {
        return false;
    }

    let last_point = path[path.len() - 1];

    let p1 = last_point;
    let p2 = new_point;

    // Check if this new segment intersects with any existing segment
    // Skip the last segment which shares the last point
    for i in 0..path.len() - 2 {
        let p3 = path[i];
        let p4 = path[i + 1];

        if line_segments_intersect(p1, p2, p3, p4) {
            return true;
        }
    }

    false
}

fn line_segments_intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> bool {
    // Calculate direction vectors
    let r = p2 - p1;
    let s = p4 - p3;

    // Calculate the cross product
    let cross_rs = r.x * s.y - r.y * s.x;

    // If cross product is zero, lines are parallel
    if cross_rs.abs() < f32::EPSILON {
        return false;
    }

    let q_minus_p = p3 - p1;
    let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / cross_rs;
    let u = (q_minus_p.x * r.y - q_minus_p.y * r.x) / cross_rs;

    // Check if intersection point is within both line segments
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

/// Convert world coordinates to window coordinates
fn world_to_window_coords(world_pos: Vec2, window_size: Vec2) -> Vec2 {
    Vec2::new(
        world_pos.x + window_size.x / 2.,
        window_size.y / 2. - world_pos.y,
    )
}

/// calculates the inner edge point of a wall
/// i.e. the closest point of the wall to the drawing area
/// that is closest to where the user is clicking.
fn calculate_inner_edge_point(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
    window_size: Vec2,
) -> Vec2 {
    let world_pos = convert_to_world_coords(cursor_pos, window_size);
    let wall_pos = wall_transform.translation();

    let inner_point = match wall_side {
        WallSide::Top => {
            // inner edge is at the bottom of the wall
            Vec2::new(world_pos.x, wall_pos.y - WALL_THICKNESS / 2.0)
        }
        WallSide::Right => {
            //inner edge is at the left of the wall
            Vec2::new(wall_pos.x - WALL_THICKNESS / 2.0, world_pos.y)
        }
        WallSide::Bottom => {
            // inner edge is at the top of the wall
            Vec2::new(world_pos.x, wall_pos.y + WALL_THICKNESS / 2.0)
        }
        WallSide::Left => {
            // inner edge is at the right of the wall
            Vec2::new(wall_pos.x + WALL_THICKNESS / 2.0, world_pos.y)
        }
    };

    world_to_window_coords(inner_point, window_size)
}

// Function to find which wall was clicked and get its inner edge point
fn get_wall_inner_edge(
    cursor_pos: Vec2,
    walls: Query<(&GlobalTransform, &Wall)>,
    window_size: Vec2,
) -> Option<(WallSide, Vec2)> {
    for (transform, wall) in walls.iter() {
        if let Some(side) = is_point_in_wall(cursor_pos, transform, wall.side, window_size) {
            return Some((
                side,
                calculate_inner_edge_point(cursor_pos, transform, wall.side, window_size),
            ));
        }
    }
    None
}

#[derive(SystemParam)]
pub struct HandleClickParams<'w, 's> {
    window: Single<'w, &'static Window, With<PrimaryWindow>>,
    buttons: Res<'w, ButtonInput<MouseButton>>,
    walls: Query<'w, 's, (&'static GlobalTransform, &'static Wall)>,
    mouse_path: ResMut<'w, MousePath>,
    cl_events: EventWriter<'w, polygon_events::ClearLines>,
    b_events: EventWriter<'w, polygon_events::BuildPolygon>,
    dl_events: EventWriter<'w, polygon_events::DrawLine>,
    wall_events: EventWriter<'w, WallEvent>,
}

pub fn handle_click(mut p: HandleClickParams) {
    let Some(cursor_position) = p.window.cursor_position() else {
        return;
    };

    let window_size = Vec2::new(p.window.width(), p.window.height());

    // We now try to get our wall inner edge, if we don't get any we are not in the wall.
    let maybe_inner_edge_point = get_wall_inner_edge(cursor_position, p.walls, window_size);

    if p.buttons.just_pressed(MouseButton::Left) && maybe_inner_edge_point.is_some() {
        info!("Starting drawing");
        p.mouse_path.reset();
        p.cl_events.send(polygon_events::ClearLines);
        p.wall_events.send(WallEvent::Bloom);
        p.mouse_path.is_drawing = true;

        let (side, inner_edge_point) = maybe_inner_edge_point.unwrap();
        p.mouse_path.path.push(inner_edge_point);
        p.mouse_path.last_position = Some(inner_edge_point);
        p.mouse_path.start = Some(side);
    } else if p.buttons.pressed(MouseButton::Left) && p.mouse_path.is_drawing {
        let current_point = cursor_position;
        let last_position = p.mouse_path.last_position.expect("drawing without prev");

        if is_approximately_equal(last_position, current_point) {
            return;
        }

        if would_create_intersection(&p.mouse_path.path, current_point) {
            info!("Path would intersect itself! Drawing stopped.");
            p.mouse_path.reset();
            p.wall_events.send(WallEvent::Reset);
            p.cl_events.send(polygon_events::ClearLines);
            return;
        }

        // We are still in the wall, just update the starting point.
        if maybe_inner_edge_point.is_some() && p.mouse_path.path.len() == 1 {
            let (side, inner_edge_point) = maybe_inner_edge_point.unwrap();
            p.mouse_path.path[0] = inner_edge_point;
            p.mouse_path.last_position = Some(inner_edge_point);
            p.mouse_path.start = Some(side);
        }
        // We've reached the end.
        else if maybe_inner_edge_point.is_some() && p.mouse_path.path.len() > 1 {
            let (side, inner_edge_point) = maybe_inner_edge_point.unwrap();
            // Add the adjusted end point
            p.mouse_path.path.push(inner_edge_point);
            p.mouse_path.last_position = Some(inner_edge_point);
            p.mouse_path.end = Some(side);

            let (path, start_wall, end_wall) = (
                p.mouse_path.path.clone(),
                p.mouse_path.start.unwrap(),
                p.mouse_path.end.unwrap(),
            );

            let polygon_data = PolygonData {
                start_wall,
                end_wall,
                path,
            };

            // Path is valid - it starts and ends on walls
            info!("Valid path created from wall to wall");
            p.cl_events.send(polygon_events::ClearLines);
            p.b_events.send(polygon_events::BuildPolygon(polygon_data));
            p.wall_events.send(WallEvent::Reset);
            p.mouse_path.reset();
            return;
        }

        // not in wall but is drawing.
        if maybe_inner_edge_point.is_none() {
            p.dl_events.send(polygon_events::DrawLine(vec![
                p.mouse_path.path[p.mouse_path.path.len() - 1],
                current_point,
            ]));
            p.mouse_path.path.push(current_point);
            p.mouse_path.last_position = Some(current_point);
        }
    } else if p.buttons.just_released(MouseButton::Left) {
        info!("Mouse released, clearing any incomplete path");
        if p.mouse_path.is_drawing {
            p.cl_events.send(polygon_events::ClearLines);
            p.wall_events.send(WallEvent::Reset);
        }
        p.mouse_path.reset();
    }
}

// Helper function to compare Vec2 values with a small epsilon
fn is_approximately_equal(a: Vec2, b: Vec2) -> bool {
    const EPSILON: f32 = 0.001;
    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
}
