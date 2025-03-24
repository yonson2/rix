use bevy::{prelude::*, window::PrimaryWindow};

use crate::rix::{
    polygon::PathEvent,
    wall::{WALL_MARGIN, WALL_THICKNESS, Wall},
};

use super::wall::{WallEvent, WallSide};

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePath>();
        app.add_systems(Update, handle_click);
        app.add_event::<PathEvent>();
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
    // Window coordinates: origin at top-left, y increases downward
    // World coordinates: origin at center, y increases upward
    let world_x = cursor_pos.x - window_size.x / 2.;
    let world_y = window_size.y / 2. - cursor_pos.y;

    // Check if point is within wall bounds
    if world_x >= wall_min_x
        && world_x <= wall_max_x
        && world_y >= wall_min_y
        && world_y <= wall_max_y
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

/// calculates the inner edge point of a wall
/// i.e. the closest point of the wall to the drawing area
/// that is closest to where the user is clicking.
fn calculate_inner_edge_point(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
    window_size: Vec2,
) -> Vec2 {
    // Convert cursor position to world coordinates
    let world_x = cursor_pos.x - window_size.x / 2.;
    let world_y = window_size.y / 2. - cursor_pos.y;

    // Get wall position in world coordinates
    let wall_pos = wall_transform.translation();

    // Calculate the inner edge point based on wall side
    let inner_point = match wall_side {
        WallSide::Top => {
            // inner edge is at the bottom of the wall
            Vec2::new(world_x, wall_pos.y - WALL_THICKNESS / 2.0)
        }
        WallSide::Right => {
            //inner edge is at the left of the wall
            Vec2::new(wall_pos.x - WALL_THICKNESS / 2.0, world_y)
        }
        WallSide::Bottom => {
            // inner edge is at the top of the wall
            Vec2::new(world_x, wall_pos.y + WALL_THICKNESS / 2.0)
        }
        WallSide::Left => {
            // inner edge is at the right of the wall
            Vec2::new(wall_pos.x + WALL_THICKNESS / 2.0, world_y)
        }
    };

    // Convert back to window coordinates
    Vec2::new(
        inner_point.x + window_size.x / 2.,
        window_size.y / 2. - inner_point.y,
    )
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

pub fn handle_click(
    window: Single<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    walls: Query<(&GlobalTransform, &Wall)>,
    mut mouse_path: ResMut<MousePath>,
    mut path_events: EventWriter<PathEvent>,
    mut wall_events: EventWriter<WallEvent>,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());

    // We now try to get our wall inner edge, if we don't get any we are not in the wall.
    let maybe_inner_edge_point = get_wall_inner_edge(cursor_position, walls, window_size);

    if buttons.just_pressed(MouseButton::Left) && maybe_inner_edge_point.is_some() {
        info!("Starting drawing");
        mouse_path.reset();
        path_events.send(PathEvent::ClearLines);
        wall_events.send(WallEvent::Bloom);
        mouse_path.is_drawing = true;

        let (side, inner_edge_point) = maybe_inner_edge_point.unwrap();
        mouse_path.path.push(inner_edge_point);
        mouse_path.last_position = Some(inner_edge_point);
        mouse_path.start = Some(side);
    } else if buttons.pressed(MouseButton::Left) && mouse_path.is_drawing {
        let current_point = cursor_position;
        let last_position = mouse_path.last_position.expect("drawing without prev");

        if is_approximately_equal(last_position, current_point) {
            return;
        }

        if would_create_intersection(&mouse_path.path, current_point) {
            info!("Path would intersect itself! Drawing stopped.");
            mouse_path.reset();
            path_events.send(PathEvent::ClearLines);
            return;
        }

        // We are still in the wall, just update the starting point.
        if maybe_inner_edge_point.is_some() && mouse_path.path.len() == 1 {
            let (side, inner_edge_point) = maybe_inner_edge_point.unwrap();
            mouse_path.path[0] = inner_edge_point;
            mouse_path.last_position = Some(inner_edge_point);
            mouse_path.start = Some(side);
        }
        // We've reached the end, re-connected to a wall after the first time.
        else if maybe_inner_edge_point.is_some() && mouse_path.path.len() > 1 {
            let (side, inner_edge_point) = maybe_inner_edge_point.unwrap();
            // Add the adjusted end point
            mouse_path.path.push(inner_edge_point);
            mouse_path.last_position = Some(inner_edge_point);
            mouse_path.end = Some(side);

            //TODO: Now we need to get the two possible polygons,
            //one that will be the:
            //  if start && end are on the same side:
            //  we connect the end with the start.
            //  if start && end are on different sides:
            //  we push points that go from the end to next corner,
            //  until we reach the starting point. So, for example:
            //  if starting side = top, and end = bottom, we would:
            //  add a new vec2 to mouse_path that is the bottom right
            //  corner, add the top right corner and finally add the
            //  starting point.
            //
            //  The "other" polygon is the one that would get if we started
            //  from the bottom and went left instead of right.
            //  let (poly1, poly2) = get_possible_polygons()

            // Path is valid - it starts and ends on walls
            info!("Valid path created from wall to wall");

            // First clear temporary lines, then draw the final path
            path_events.send(PathEvent::ClearLines);
            wall_events.send(WallEvent::Reset);
            path_events.send(PathEvent::DrawPolygon(mouse_path.path.clone()));
            mouse_path.reset();
            return;
        }

        // not in wall but is drawing.
        if maybe_inner_edge_point.is_none() {
            path_events.send(PathEvent::DrawLine(vec![
                mouse_path.path[mouse_path.path.len() - 1],
                current_point,
            ]));
            mouse_path.path.push(current_point);
            mouse_path.last_position = Some(current_point);
        }
    } else if buttons.just_released(MouseButton::Left) {
        info!("Mouse released, clearing any incomplete path");
        if mouse_path.is_drawing {
            path_events.send(PathEvent::ClearLines);
            wall_events.send(WallEvent::Reset);
        }
        mouse_path.reset();
    }
}

// Helper function to compare Vec2 values with a small epsilon
fn is_approximately_equal(a: Vec2, b: Vec2) -> bool {
    const EPSILON: f32 = 0.001;
    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
}
