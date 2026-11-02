use bevy::{
    input::{
        ButtonState,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::PrimaryWindow,
};

use crate::rix::{
    AppSystems, PausableSystems,
    events::{self, ShapeMeta},
    polygon::{PolygonData, Territory, is_point_in_polygon},
    state::{IsPaused, Phase},
    wall::{FIELD, WALL_THICKNESS, WALL_THICKNESS_TOP, Wall, WallEvent, WallSide},
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
                .in_set(AppSystems::RecordInput)
                .in_set(PausableSystems),
        );
        app.add_systems(OnExit(Phase::Playing), cleanup_resources);
        app.add_systems(OnExit(IsPaused::Paused), cancel_line);
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
    mut mouse_button_events: MessageReader<MouseButtonInput>,
    mut touch_events: MessageReader<TouchInput>,
    mut cursor_moved_events: MessageReader<CursorMoved>,
    mut input_state: ResMut<InputState>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = *camera;
    let to_world = |pos| to_field(camera, camera_transform, pos);

    let mut latest_cursor_pos_screen: Option<Vec2> = window.cursor_position();

    for event in cursor_moved_events.read() {
        latest_cursor_pos_screen = Some(event.position); // Overwrite with newer event positions
    }
    let latest_cursor_pos_world = latest_cursor_pos_screen.and_then(to_world);

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
        let Some(touch_pos_world) = to_world(touch_pos_screen) else {
            continue;
        };

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
    walls: Query<(&GlobalTransform, &Wall)>,
    shapes: Query<(&Territory, &ShapeMeta)>,
    mut cl_events: MessageWriter<events::ClearLines>,
    mut b_events: MessageWriter<events::BuildPolygon>,
    mut dl_events: MessageWriter<events::DrawLine>,
    mut wall_events: MessageWriter<WallEvent>,
    mut input_pos_events: MessageWriter<events::InputPosition>,
) {
    let just_pressed = input_state.just_pressed_this_frame;
    let pressed = input_state.is_pressed;
    let just_released = input_state.just_released_this_frame;

    let Some(user_position) = input_state.last_position_world else {
        if just_released && user_path.is_drawing {
            cl_events.write(events::ClearLines);
            wall_events.write(WallEvent::Reset);
            user_path.reset();
        }
        return;
    };

    if user_path.is_drawing {
        input_pos_events.write(events::InputPosition(user_position));
    }

    let walls_vec: Vec<(&GlobalTransform, &Wall)> = walls.iter().collect(); // Renamed local var
    let shapes_points: Vec<(Vec<Vec2>, ShapeMeta)> = shapes
        .iter()
        .map(|(shape, meta)| (shape.0.clone(), *meta))
        .collect();

    let maybe_edge = get_edge_point(user_position, &walls_vec, &shapes_points);

    if just_pressed && let Some((side, edge)) = maybe_edge {
        //FIX: we are calling this twice without need.
        let is_polygon_edge = shapes.iter().any(|(shape, _)| {
            let points = shape.0.clone();
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
                    let points = shape.0.clone();
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
        cl_events.write(events::ClearLines);
        wall_events.write(WallEvent::Bloom);
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
            wall_events.write(WallEvent::Reset);
            cl_events.write(events::ClearLines);

            // Reset pressed state because the action failed/was cancelled
            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            return;
        }

        if let Some((side, edge)) = maybe_edge
            && user_path.path.len() == 1
        {
            user_path.path[0] = edge; // Modify first point directly
            user_path.start = Some(side); // Update start side if snapped
        }
        // Complete path
        else if let Some((side, edge)) = maybe_edge
            && user_path.path.len() > 1
        {
            user_path.path.push(edge); // Add final edge point
            user_path.end = Some(side);

            let path = user_path.path.clone();
            // Check start point
            let start_point = path.first().unwrap();
            let start_polygon = shapes.iter().find_map(|(shape, meta)| {
                let points = shape.0.clone();
                if is_point_in_polygon(start_point, &points) {
                    Some((points, *meta))
                } else {
                    None
                }
            });

            // Check end point
            let end_point = path.last().unwrap();
            let end_polygon = shapes.iter().find_map(|(shape, meta)| {
                let points = shape.0.clone();
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

            cl_events.write(events::ClearLines);
            b_events.write(events::BuildPolygon(polygon_data));
            wall_events.write(WallEvent::Reset);
            user_path.reset();

            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            info!("Valid path created from wall to wall");
            return;
        }

        // Draw line segment in open space
        if maybe_edge.is_none() {
            let prev_point = user_path.path.last().unwrap();
            dl_events.write(events::DrawLine(vec![*prev_point, user_position]));
            user_path.path.push(user_position);
        }
    } else if just_released {
        if user_path.is_drawing {
            cl_events.write(events::ClearLines);
            wall_events.write(WallEvent::Reset);
        }
        user_path.reset(); // Reset path on release
    }
}

fn cleanup_resources(mut user_path: ResMut<UserPath>, mut input_state: ResMut<InputState>) {
    user_path.reset();
    *input_state = InputState::default(); // Reset input state
}

/// A run resumes with no line in progress: the touch that drew it ended while paused, and
/// the pause drops that message ([034]).
fn cancel_line(
    mut user_path: ResMut<UserPath>,
    mut input_state: ResMut<InputState>,
    mut cl_events: MessageWriter<events::ClearLines>,
    mut wall_events: MessageWriter<WallEvent>,
) {
    if user_path.is_drawing {
        cl_events.write(events::ClearLines);
        wall_events.write(WallEvent::Reset);
    }
    user_path.reset();
    *input_state = InputState::default();
}

/// A viewport position in the world, clamped to the field: a finger in the letterbox stays
/// on the nearest wall ([019]).
fn to_field(camera: &Camera, camera_transform: &GlobalTransform, pos: Vec2) -> Option<Vec2> {
    camera
        .viewport_to_world_2d(camera_transform, pos)
        .ok()
        .map(|p| p.clamp(-FIELD / 2., FIELD / 2.))
}

fn is_point_in_wall(
    cursor_pos: Vec2,
    wall_transform: &GlobalTransform,
    wall_side: WallSide,
) -> Option<WallSide> {
    let wall_pos = wall_transform.translation();

    let (wall_width, wall_height) = match wall_side {
        WallSide::Top => (FIELD.x, WALL_THICKNESS_TOP),
        WallSide::Bottom => (FIELD.x, WALL_THICKNESS),
        WallSide::Right | WallSide::Left => (WALL_THICKNESS, FIELD.y),
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
) -> Option<(WallSide, Vec2)> {
    let wall_edge = walls.iter().find_map(|(transform, wall)| {
        is_point_in_wall(cursor_pos, transform, wall.side).map(|side| {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The right wall, placed as `spawn_walls` places it on the field.
    fn right_wall() -> GlobalTransform {
        GlobalTransform::from_xyz(207.5, 0., 0.)
    }

    #[test]
    fn point_in_wall_includes_half_a_wall_of_margin() {
        let wall = right_wall();
        let side = WallSide::Right;
        assert_eq!(
            is_point_in_wall(Vec2::new(210., 0.), &wall, side),
            Some(side)
        );
        // Inner edge at x = 190; the margin reaches x = 172.5.
        assert_eq!(
            is_point_in_wall(Vec2::new(175., 0.), &wall, side),
            Some(side)
        );
        assert_eq!(is_point_in_wall(Vec2::new(165., 0.), &wall, side), None);
    }

    #[test]
    fn inner_edge_point_snaps_to_the_inner_corners_lines() {
        let at = Vec2::new(10., 20.);
        let top = GlobalTransform::from_xyz(0., 367.5, 0.);
        let bottom = GlobalTransform::from_xyz(0., -382.5, 0.);
        let left = GlobalTransform::from_xyz(-207.5, 0., 0.);
        assert_eq!(
            calculate_inner_edge_point(at, &top, WallSide::Top),
            Vec2::new(10., 335.)
        );
        assert_eq!(
            calculate_inner_edge_point(at, &right_wall(), WallSide::Right),
            Vec2::new(190., 20.)
        );
        assert_eq!(
            calculate_inner_edge_point(at, &bottom, WallSide::Bottom),
            Vec2::new(10., -365.)
        );
        assert_eq!(
            calculate_inner_edge_point(at, &left, WallSide::Left),
            Vec2::new(-190., 20.)
        );
    }

    #[test]
    fn edge_point_prefers_walls_then_claimed_shapes() {
        let transform = right_wall();
        let wall = Wall {
            side: WallSide::Right,
        };
        let walls = [(&transform, &wall)];
        let shape = vec![
            Vec2::new(-100., -100.),
            Vec2::new(0., -100.),
            Vec2::new(0., 0.),
            Vec2::new(-100., 0.),
        ];
        let meta = ShapeMeta {
            start_wall: WallSide::Bottom,
            start_point: shape[0],
        };
        let shapes = [(shape, meta)];

        assert_eq!(
            get_edge_point(Vec2::new(210., 5.), &walls, &shapes),
            Some((WallSide::Right, Vec2::new(190., 5.)))
        );
        assert_eq!(
            get_edge_point(Vec2::new(-50., -50.), &walls, &shapes),
            Some((WallSide::Bottom, Vec2::new(-50., -50.)))
        );
        assert_eq!(get_edge_point(Vec2::new(50., 50.), &walls, &shapes), None);
    }

    #[test]
    fn resume_cancels_the_line_and_releases_the_pointer() {
        use bevy::ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.init_resource::<Messages<events::ClearLines>>();
        world.init_resource::<Messages<WallEvent>>();
        world.insert_resource(UserPath {
            path: vec![Vec2::ZERO],
            is_drawing: true,
            start: Some(WallSide::Left),
            end: None,
        });
        world.insert_resource(InputState {
            is_pressed: true,
            active_touch_id: Some(0),
            ..default()
        });
        world.run_system_once(cancel_line).unwrap();

        let path = world.resource::<UserPath>();
        assert!(!path.is_drawing && path.path.is_empty());
        let input = world.resource::<InputState>();
        assert!(!input.is_pressed && input.active_touch_id.is_none());
        assert_eq!(world.resource::<Messages<events::ClearLines>>().len(), 1);
        assert_eq!(world.resource::<Messages<WallEvent>>().len(), 1);
    }

    #[test]
    fn segments_intersect_inclusive_of_endpoints() {
        let (a, b) = (Vec2::new(0., 0.), Vec2::new(2., 2.));
        assert!(line_segments_intersect(
            a,
            b,
            Vec2::new(0., 2.),
            Vec2::new(2., 0.)
        ));
        assert!(line_segments_intersect(a, b, b, Vec2::new(3., 0.)));
        assert!(!line_segments_intersect(
            a,
            b,
            Vec2::new(3., 0.),
            Vec2::new(4., 0.)
        ));
        assert!(!line_segments_intersect(
            a,
            b,
            Vec2::new(0., 1.),
            Vec2::new(2., 3.)
        ));
    }

    #[test]
    fn segments_intersect_quirk_collinear_overlap_is_not_an_intersection() {
        let (a, b) = (Vec2::new(0., 0.), Vec2::new(2., 0.));
        assert!(!line_segments_intersect(
            a,
            b,
            Vec2::new(1., 0.),
            Vec2::new(3., 0.)
        ));
    }

    #[test]
    fn path_intersection_ignores_the_last_segment() {
        let path = [Vec2::new(0., 0.), Vec2::new(10., 0.), Vec2::new(10., 10.)];
        assert!(!would_create_intersection(&path[..2], Vec2::new(5., -5.)));
        assert!(would_create_intersection(&path, Vec2::new(5., -5.)));
        assert!(!would_create_intersection(&path, Vec2::new(20., 20.)));
        // Going back down through the corner touches the first segment at (10, 0).
        assert!(would_create_intersection(&path, Vec2::new(10., -5.)));
    }

    #[test]
    fn approximately_equal_below_a_thousandth() {
        assert!(is_approximately_equal(
            Vec2::ZERO,
            Vec2::new(0.0009, -0.0009)
        ));
        assert!(!is_approximately_equal(Vec2::ZERO, Vec2::new(0.002, 0.)));
    }
}
