use bevy::{
    input::{
        ButtonState,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_prototype_lyon::prelude::*;

use crate::rix::{
    events::{self, ShapeMeta},
    polygon::{PolygonData, extract_points_from_shape, is_point_in_polygon},
    state::{AppState, IsPaused},
    wall::{WALL_THICKNESS, WALL_THICKNESS_TOP, Wall, WallEvent, WallSide},
};

pub struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserPath>();
        app.init_resource::<InputState>();
        app.add_systems(
            Update,
            (
                reset_input_state_flags,
                handle_input_events.before(handle_input),
                handle_input,
            )
                .chain()
                .run_if(in_state(IsPaused::Running)),
        );
        app.add_systems(OnExit(AppState::InGame), cleanup_resources);
    }
}

#[derive(Resource, Default, Debug)]
pub struct UserPath {
    path: Vec<Vec2>,
    pub is_drawing: bool,

    start: Option<WallSide>,
    end: Option<WallSide>,
}

impl UserPath {
    fn reset(&mut self) {
        self.path.clear();
        self.is_drawing = false;
        self.start = None;
        self.end = None;
    }
}

#[derive(Resource, Default, Debug)]
pub struct InputState {
    is_pressed: bool,
    active_touch_id: Option<u64>,
    last_position_screen: Option<Vec2>,
    last_position_world: Option<Vec2>,
    just_pressed_this_frame: bool,
    just_released_this_frame: bool,
}

// System to reset frame-specific flags
fn reset_input_state_flags(mut input_state: ResMut<InputState>) {
    input_state.just_pressed_this_frame = false;
    input_state.just_released_this_frame = false;
}

// System to process hardware input events and update InputState
// This system translates raw events into the abstract state the main logic needs
fn handle_input_events(
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut touch_events: EventReader<TouchInput>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut input_state: ResMut<InputState>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let window_size = Vec2::new(window.width(), window.height());

    let mut latest_cursor_pos_screen: Option<Vec2> = window.cursor_position();

    for event in cursor_moved_events.read() {
        latest_cursor_pos_screen = Some(event.position); // Overwrite with newer event positions
    }
    let latest_cursor_pos_world =
        latest_cursor_pos_screen.map(|p| convert_to_world_coords(p, window_size));

    for event in mouse_button_events.read() {
        if event.button == MouseButton::Left {
            match event.state {
                ButtonState::Pressed => {
                    if !input_state.is_pressed {
                        input_state.is_pressed = true;
                        input_state.just_pressed_this_frame = true;
                        input_state.active_touch_id = None; // Mouse takes over

                        input_state.last_position_screen = latest_cursor_pos_screen;
                        input_state.last_position_world = latest_cursor_pos_world;
                    } // else: Ignore if already pressed
                }
                ButtonState::Released => {
                    if input_state.is_pressed && input_state.active_touch_id.is_none() {
                        input_state.is_pressed = false;
                        input_state.just_released_this_frame = true;

                        input_state.last_position_screen = latest_cursor_pos_screen;
                        input_state.last_position_world = latest_cursor_pos_world;
                    }
                }
            }
        }
    }

    for event in touch_events.read() {
        let touch_pos_screen = event.position;
        let touch_pos_world = convert_to_world_coords(touch_pos_screen, window_size);

        match event.phase {
            TouchPhase::Started => {
                if !input_state.is_pressed {
                    input_state.is_pressed = true;
                    input_state.just_pressed_this_frame = true;
                    input_state.active_touch_id = Some(event.id);
                    input_state.last_position_screen = Some(touch_pos_screen);
                    input_state.last_position_world = Some(touch_pos_world);
                } // else: Ignore if already pressed
            }
            TouchPhase::Moved => {
                if input_state.active_touch_id == Some(event.id) {
                    input_state.last_position_screen = Some(touch_pos_screen);
                    input_state.last_position_world = Some(touch_pos_world);
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if input_state.active_touch_id == Some(event.id) {
                    input_state.is_pressed = false;
                    input_state.just_released_this_frame = true;
                    input_state.active_touch_id = None;
                    input_state.last_position_screen = Some(touch_pos_screen);
                    input_state.last_position_world = Some(touch_pos_world);
                }
            }
        }
    }

    let mouse_is_active_and_pressed =
        input_state.is_pressed && input_state.active_touch_id.is_none();

    if mouse_is_active_and_pressed {
        if input_state.last_position_world != latest_cursor_pos_world {
            input_state.last_position_screen = latest_cursor_pos_screen;
            input_state.last_position_world = latest_cursor_pos_world;
        }
    } else if !input_state.is_pressed && input_state.last_position_world != latest_cursor_pos_world
    {
        input_state.last_position_screen = latest_cursor_pos_screen;
        input_state.last_position_world = latest_cursor_pos_world;
    }
}

// Main Input Handling Logic (Original Logic, Reads from InputState)
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    mut input_state: ResMut<InputState>,
    mut user_path: ResMut<UserPath>,
    window: Single<&Window, With<PrimaryWindow>>,
    walls: Query<(&GlobalTransform, &Wall)>,
    shapes: Query<(&Shape, &ShapeMeta)>,
    mut cl_events: EventWriter<events::ClearLines>,
    mut b_events: EventWriter<events::BuildPolygon>,
    mut dl_events: EventWriter<events::DrawLine>,
    mut wall_events: EventWriter<WallEvent>,
    mut input_pos_events: EventWriter<events::InputPosition>,
) {
    let just_pressed = input_state.just_pressed_this_frame;
    let pressed = input_state.is_pressed;
    let just_released = input_state.just_released_this_frame;

    let Some(user_position) = input_state.last_position_world else {
        if just_released && user_path.is_drawing {
            cl_events.send(events::ClearLines);
            wall_events.send(WallEvent::Reset);
            user_path.reset();
        }
        return;
    };

    let window_size = Vec2::new(window.width(), window.height());

    if user_path.is_drawing {
        input_pos_events.send(events::InputPosition(user_position));
    }

    let walls_vec: Vec<(&GlobalTransform, &Wall)> = walls.iter().collect(); // Renamed local var
    let shapes_points: Vec<(Vec<Vec2>, ShapeMeta)> = shapes
        .iter()
        .map(|(shape, meta)| (extract_points_from_shape(shape), *meta))
        .collect();

    let maybe_edge = get_edge_point(user_position, &walls_vec, &shapes_points, window_size);

    if just_pressed && maybe_edge.is_some() {
        let (side, edge) = maybe_edge.unwrap();

        //FIX: we are calling this twice without need.
        let is_polygon_edge = shapes.iter().any(|(shape, _)| {
            let points = extract_points_from_shape(shape);
            is_point_in_polygon(&edge, &points) // Use original check
        });

        // Only check for filled areas if we're on a wall edge, not a polygon edge
        let area_already_filled = if !is_polygon_edge {
            let inward_dir = match side {
                WallSide::Top => Vec2::new(0.0, -1.0),
                WallSide::Right => Vec2::new(-1.0, 0.0),
                WallSide::Bottom => Vec2::new(0.0, 1.0),
                WallSide::Left => Vec2::new(1.0, 0.0),
            };

            let check_distances = [1.0, 5.0, 10.0];

            check_distances.iter().any(|&distance| {
                let check_point = edge + (inward_dir * distance);

                shapes.iter().any(|(shape, _)| {
                    let points = extract_points_from_shape(shape);
                    is_point_in_polygon(&check_point, &points)
                })
            })
        } else {
            false // This allows drawing from polygon edges
        };

        if area_already_filled {
            info!("Cannot start drawing - area is already filled");
            return;
        }

        info!("Starting drawing");
        user_path.reset();
        cl_events.send(events::ClearLines);
        wall_events.send(WallEvent::Bloom);
        user_path.is_drawing = true;

        user_path.path.push(edge);
        user_path.start = Some(side);
    } else if pressed && user_path.is_drawing {
        let last_position = user_path
            .path
            .last()
            .expect("Path shouldn't be empty if drawing");

        if is_approximately_equal(*last_position, user_position) {
            return;
        }

        if would_create_intersection(&user_path.path, user_position) {
            user_path.reset();
            wall_events.send(WallEvent::Reset);
            cl_events.send(events::ClearLines);

            // Reset pressed state because the action failed/was cancelled
            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            return;
        }

        if maybe_edge.is_some() && user_path.path.len() == 1 {
            let (side, edge) = maybe_edge.unwrap();
            user_path.path[0] = edge; // Modify first point directly
            user_path.start = Some(side); // Update start side if snapped
        }
        // Complete path
        else if maybe_edge.is_some() && user_path.path.len() > 1 {
            let (side, edge) = maybe_edge.unwrap();
            user_path.path.push(edge); // Add final edge point
            user_path.end = Some(side);

            let path = user_path.path.clone();
            // Check start point
            let start_point = path.first().unwrap();
            let start_polygon = shapes.iter().find_map(|(shape, meta)| {
                let points = extract_points_from_shape(shape);
                if is_point_in_polygon(start_point, &points) {
                    Some((points, *meta))
                } else {
                    None
                }
            });

            // Check end point
            let end_point = path.last().unwrap();
            let end_polygon = shapes.iter().find_map(|(shape, meta)| {
                let points = extract_points_from_shape(shape);
                if is_point_in_polygon(end_point, &points) {
                    Some((points, *meta))
                } else {
                    None
                }
            });

            let polygon_data = PolygonData {
                start_wall: user_path.start.unwrap(),
                end_wall: user_path.end.unwrap(),
                start_polygon,
                end_polygon,
                path,
            };

            cl_events.send(events::ClearLines);
            b_events.send(events::BuildPolygon(polygon_data));
            wall_events.send(WallEvent::Reset);
            user_path.reset();

            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            info!("Valid path created from wall to wall");
            return;
        }

        // Draw line segment in open space
        if maybe_edge.is_none() {
            let prev_point = user_path.path.last().unwrap();
            dl_events.send(events::DrawLine(vec![*prev_point, user_position]));
            user_path.path.push(user_position);
        }
    } else if just_released {
        if user_path.is_drawing {
            cl_events.send(events::ClearLines);
            wall_events.send(WallEvent::Reset);
        }
        user_path.reset(); // Reset path on release
    }
}

fn cleanup_resources(mut user_path: ResMut<UserPath>, mut input_state: ResMut<InputState>) {
    user_path.reset();
    *input_state = InputState::default(); // Reset input state
}

pub fn convert_to_world_coords(window_pos: Vec2, window_size: Vec2) -> Vec2 {
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
    let wall_pos = wall_transform.translation();

    let (wall_width, wall_height) = match wall_side {
        WallSide::Top => {
            let width = window_size.x;
            (width, WALL_THICKNESS_TOP)
        }
        WallSide::Bottom => {
            let width = window_size.x;
            (width, WALL_THICKNESS)
        }
        WallSide::Right | WallSide::Left => {
            let height = window_size.y;
            (WALL_THICKNESS, height)
        }
    };

    let detection_margin = WALL_THICKNESS * 0.5;
    let half_width = wall_width / 2. + detection_margin;
    let half_height = wall_height / 2. + detection_margin;

    let wall_min_x = wall_pos.x - half_width;
    let wall_max_x = wall_pos.x + half_width;
    let wall_min_y = wall_pos.y - half_height;
    let wall_max_y = wall_pos.y + half_height;

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

fn calculate_inner_edge_point(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
) -> Vec2 {
    let wall_pos = wall_transform.translation();

    match wall_side {
        WallSide::Top => Vec2::new(cursor_pos.x, wall_pos.y - WALL_THICKNESS_TOP / 2.0),
        WallSide::Right => Vec2::new(wall_pos.x - WALL_THICKNESS / 2.0, cursor_pos.y),
        WallSide::Bottom => Vec2::new(cursor_pos.x, wall_pos.y + WALL_THICKNESS / 2.0),
        WallSide::Left => Vec2::new(wall_pos.x + WALL_THICKNESS / 2.0, cursor_pos.y),
    }
}

fn get_edge_point(
    cursor_pos: Vec2,
    walls: &[(&GlobalTransform, &Wall)],
    shapes: &[(Vec<Vec2>, ShapeMeta)],
    window_size: Vec2,
) -> Option<(WallSide, Vec2)> {
    let wall_edge = walls.iter().find_map(|(transform, wall)| {
        is_point_in_wall(cursor_pos, transform, wall.side, window_size).map(|side| {
            (
                side,
                calculate_inner_edge_point(cursor_pos, transform, wall.side),
            )
        })
    });
    if wall_edge.is_some() {
        return wall_edge;
    }
    shapes.iter().find_map(|(points, meta)| {
        if is_point_in_polygon(&cursor_pos, points) {
            Some((meta.start_wall, cursor_pos))
        } else {
            None
        }
    })
}

fn would_create_intersection(path: &[Vec2], new_point: Vec2) -> bool {
    if path.len() < 3 {
        return false;
    }

    let last_point = path[path.len() - 1];

    let p1 = last_point;
    let p2 = new_point;

    for i in 0..path.len() - 2 {
        // Check against all segments except the immediately previous one
        let p3 = path[i];
        let p4 = path[i + 1];

        if line_segments_intersect(p1, p2, p3, p4) {
            return true;
        }
    }

    false
}

fn line_segments_intersect(p1: Vec2, p2: Vec2, p3: Vec2, p4: Vec2) -> bool {
    let r = p2 - p1;
    let s = p4 - p3;

    let cross_rs = r.x * s.y - r.y * s.x;

    // Check if lines are parallel
    if cross_rs.abs() < f32::EPSILON {
        return false;
    }

    let q_minus_p = p3 - p1;
    let t = (q_minus_p.x * s.y - q_minus_p.y * s.x) / cross_rs;
    let u = (q_minus_p.x * r.y - q_minus_p.y * r.x) / cross_rs;

    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn is_approximately_equal(a: Vec2, b: Vec2) -> bool {
    const EPSILON: f32 = 0.001;
    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON
}
