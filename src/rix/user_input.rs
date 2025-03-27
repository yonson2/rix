use std::cmp::max;

use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

use crate::rix::{
    polygon::PolygonData,
    wall::{WALL_THICKNESS, Wall},
};

use super::{
    events as polygon_events,
    state::{AppState, IsPaused},
    wall::{WallEvent, WallSide},
};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserPath>();
        app.add_systems(Update, handle_click.run_if(in_state(IsPaused::Running)));
        app.add_systems(OnExit(AppState::InGame), cleanup_resources);
    }
}

#[derive(Resource, Default, Debug)]
pub struct UserPath {
    path: Vec<Vec2>,
    is_drawing: bool,
    last_position: Option<Vec2>,
    start: Option<WallSide>,
    end: Option<WallSide>,
}

impl UserPath {
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
            // horizontal
            let width = window_size.x;
            (width, WALL_THICKNESS)
        }
        WallSide::Right | WallSide::Left => {
            // vertical
            let height = window_size.y;
            (WALL_THICKNESS, height)
        }
    };

    //TODO: check this value and improve/remove
    let detection_margin = 3.;
    // Calculate wall bounds in world coordinates
    let half_width = wall_width / 2. + detection_margin;
    let half_height = wall_height / 2. + detection_margin;

    let wall_min_x = wall_pos.x - half_width;
    let wall_max_x = wall_pos.x + half_width;
    let wall_min_y = wall_pos.y - half_height;
    let wall_max_y = wall_pos.y + half_height;

    // Check if point is within wall bounds
    if cursor_pos.x >= wall_min_x
        && cursor_pos.x <= wall_max_x
        && cursor_pos.y >= wall_min_y
        && cursor_pos.y <= wall_max_y
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
) -> Vec2 {
    let wall_pos = wall_transform.translation();

    match wall_side {
        WallSide::Top => {
            // inner edge is at the bottom of the wall
            Vec2::new(cursor_pos.x, wall_pos.y - WALL_THICKNESS / 2.0)
        }
        WallSide::Right => {
            //inner edge is at the left of the wall
            Vec2::new(wall_pos.x - WALL_THICKNESS / 2.0, cursor_pos.y)
        }
        WallSide::Bottom => {
            // inner edge is at the top of the wall
            Vec2::new(cursor_pos.x, wall_pos.y + WALL_THICKNESS / 2.0)
        }
        WallSide::Left => {
            // inner edge is at the right of the wall
            Vec2::new(wall_pos.x + WALL_THICKNESS / 2.0, cursor_pos.y)
        }
    }
}

// Function to find which wall was clicked and get its inner edge point in world coordinates
fn get_wall_inner_edge(
    cursor_pos: Vec2,
    walls: Query<(&GlobalTransform, &Wall)>,
    window_size: Vec2,
) -> Option<(WallSide, Vec2)> {
    for (transform, wall) in walls.iter() {
        if let Some(side) = is_point_in_wall(cursor_pos, transform, wall.side, window_size) {
            return Some((
                side,
                calculate_inner_edge_point(cursor_pos, transform, wall.side),
            ));
        }
    }
    None
}

#[derive(SystemParam)]
pub struct HandleClickParams<'w, 's> {
    window: Single<'w, &'static Window, With<PrimaryWindow>>,
    clicks: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
    walls: Query<'w, 's, (&'static GlobalTransform, &'static Wall)>,
    user_path: ResMut<'w, UserPath>,
    cl_events: EventWriter<'w, polygon_events::ClearLines>,
    b_events: EventWriter<'w, polygon_events::BuildPolygon>,
    dl_events: EventWriter<'w, polygon_events::DrawLine>,
    wall_events: EventWriter<'w, WallEvent>,
}

pub fn handle_click(mut p: HandleClickParams) {
    let input = UserInput::new(
        p.clicks.into_inner(),
        p.touches.into_inner(),
        p.window.cursor_position(),
    );
    let Some(user_position) = input.position() else {
        return;
    };

    let window_size = Vec2::new(p.window.width(), p.window.height());
    let user_position = convert_to_world_coords(user_position, window_size);

    // We now try to get our wall inner edge, if we don't get any we are not in the wall.
    let maybe_inner_edge_point = get_wall_inner_edge(user_position, p.walls, window_size);

    if input.just_pressed() && maybe_inner_edge_point.is_some() {
        info!("Starting drawing");
        p.user_path.reset();
        p.cl_events.send(polygon_events::ClearLines);
        p.wall_events.send(WallEvent::Bloom);
        p.user_path.is_drawing = true;

        let (side, world_inner_edge) = maybe_inner_edge_point.unwrap();
        p.user_path.path.push(world_inner_edge);
        p.user_path.last_position = Some(world_inner_edge);
        p.user_path.start = Some(side);
    } else if input.pressed() && p.user_path.is_drawing {
        let last_position = p.user_path.last_position.expect("drawing without prev");

        if is_approximately_equal(last_position, user_position) {
            return;
        }

        if would_create_intersection(&p.user_path.path, user_position) {
            info!("Path would intersect itself! Drawing stopped.");
            p.user_path.reset();
            p.wall_events.send(WallEvent::Reset);
            p.cl_events.send(polygon_events::ClearLines);
            return;
        }

        // We are still in the wall, just update the starting point.
        if maybe_inner_edge_point.is_some() && p.user_path.path.len() == 1 {
            let (side, world_inner_edge) = maybe_inner_edge_point.unwrap();
            p.user_path.path[0] = world_inner_edge;
            p.user_path.last_position = Some(world_inner_edge);
            p.user_path.start = Some(side);
        }
        // We've reached the end.
        else if maybe_inner_edge_point.is_some() && p.user_path.path.len() > 1 {
            let (side, world_inner_edge) = maybe_inner_edge_point.unwrap();
            // Add the adjusted end point
            p.user_path.path.push(world_inner_edge);
            p.user_path.last_position = Some(world_inner_edge);
            p.user_path.end = Some(side);

            let path = p.user_path.path.clone();

            let polygon_data = PolygonData {
                start_wall: p.user_path.start.unwrap(),
                end_wall: p.user_path.end.unwrap(),
                path,
            };

            // Path is valid - it starts and ends on walls
            info!("Valid path created from wall to wall");
            p.cl_events.send(polygon_events::ClearLines);
            p.b_events.send(polygon_events::BuildPolygon(polygon_data));
            p.wall_events.send(WallEvent::Reset);
            p.user_path.reset();
            return;
        }

        // not in wall but is drawing.
        if maybe_inner_edge_point.is_none() {
            p.dl_events.send(polygon_events::DrawLine(vec![
                p.user_path.path[p.user_path.path.len() - 1],
                user_position,
            ]));
            p.user_path.path.push(user_position);
            p.user_path.last_position = Some(user_position);
        }
    } else if input.just_released() {
        info!("Input released, clearing any incomplete path");
        if p.user_path.is_drawing {
            p.cl_events.send(polygon_events::ClearLines);
            p.wall_events.send(WallEvent::Reset);
        }
        p.user_path.reset();
    }
}

// Helper function to compare Vec2 values with a small epsilon
fn is_approximately_equal(a: Vec2, b: Vec2) -> bool {
    const EPSILON: f32 = 0.001;
    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
}

fn cleanup_resources(mut user_path: ResMut<UserPath>) {
    user_path.reset();
}

/// `UserInput` is a common handler for abstracting the same behaviour across
/// mouse input and touch input, if on a system where both options are available
/// right now I'm choosing to support touch first and then fallback to mouse.
pub struct UserInput<'w> {
    clicks: &'w ButtonInput<MouseButton>,
    touches: &'w Touches,
    cursor_pos: Option<Vec2>,
}

impl<'w> UserInput<'w> {
    fn new(
        clicks: &'w ButtonInput<MouseButton>,
        touches: &'w Touches,
        cursor_pos: Option<Vec2>,
    ) -> Self {
        Self {
            clicks,
            touches,
            cursor_pos,
        }
    }

    fn len(&self) -> usize {
        max(
            self.clicks.get_pressed().count(),
            self.touches.iter().count(),
        )
    }

    fn just_pressed(&self) -> bool {
        let touch = self.touches.any_just_pressed();
        if touch {
            touch
        } else {
            self.clicks.just_pressed(MouseButton::Left)
        }
    }

    fn pressed(&self) -> bool {
        let touch = self.touches.iter().next().is_some();
        if touch {
            touch
        } else {
            self.clicks.pressed(MouseButton::Left)
        }
    }

    fn just_released(&self) -> bool {
        let touch = self.touches.any_just_released();
        if touch {
            touch
        } else {
            self.clicks.just_released(MouseButton::Left)
        }
    }

    fn position(&self) -> Option<Vec2> {
        match self.touches.iter().next() {
            Some(t) => Some(t.position()),
            None => self.cursor_pos,
        }
    }
}
