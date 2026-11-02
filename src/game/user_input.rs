use bevy::{
    input::{ButtonState, touch::TouchPhase},
    prelude::*,
    window::{PrimaryWindow, WindowEvent},
};

use crate::game::{
    AppSystems, PausableSystems, events,
    polygon::is_point_in_polygon,
    state::{IsPaused, Phase},
    territory::{FreeRegion, line_end, locate},
    wall::{FIELD, WALL_THICKNESS, WallEvent},
};

/// A press this close to the free boundary, or outside the free region, starts a line.
const START_MARGIN: f32 = WALL_THICKNESS * 0.5;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UserPath>();
    app.init_resource::<InputState>();
    app.add_systems(
        FixedUpdate,
        (handle_input_events, handle_input)
            .chain()
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
    app.add_systems(OnExit(Phase::Playing), cleanup_resources);
    app.add_systems(OnExit(IsPaused::Paused), cancel_line);
}

#[derive(Resource, Default, Debug)]
pub struct UserPath {
    path: Vec<Vec2>,
    pub is_drawing: bool,
}

impl UserPath {
    fn reset(&mut self) {
        self.path.clear();
        self.is_drawing = false;
    }
}

#[derive(Resource, Default, Debug)]
pub struct InputState {
    is_pressed: bool,
    active_touch_id: Option<u64>,
    last_position_world: Option<Vec2>,
    /// This tick's steps of the one pointer, in arrival order ([077]).
    steps: Vec<Pointer>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Pointer {
    Press(Vec2),
    Move(Vec2),
    Release,
}

impl InputState {
    /// Turns one window event into a step of the one pointer: the first touch, or the left
    /// mouse button. `cursor` is the mouse position on screen.
    fn record(
        &mut self,
        event: &WindowEvent,
        cursor: &mut Option<Vec2>,
        to_world: impl Fn(Vec2) -> Option<Vec2>,
    ) {
        match event {
            WindowEvent::CursorMoved(e) => {
                *cursor = Some(e.position);
                if self.active_touch_id.is_none() {
                    self.last_position_world = to_world(e.position);
                    if self.is_pressed
                        && let Some(p) = self.last_position_world
                    {
                        self.steps.push(Pointer::Move(p));
                    }
                }
            }
            WindowEvent::MouseButtonInput(e) if e.button == MouseButton::Left => {
                let p = cursor.and_then(&to_world);
                match e.state {
                    ButtonState::Pressed if !self.is_pressed => {
                        self.is_pressed = true;
                        self.active_touch_id = None;
                        self.last_position_world = p;
                        self.steps.extend(p.map(Pointer::Press));
                    }
                    ButtonState::Released if self.is_pressed && self.active_touch_id.is_none() => {
                        self.is_pressed = false;
                        self.last_position_world = p;
                        self.steps.push(Pointer::Release);
                    }
                    _ => {}
                }
            }
            WindowEvent::TouchInput(e) => {
                let Some(p) = to_world(e.position) else {
                    return;
                };
                match e.phase {
                    TouchPhase::Started if !self.is_pressed => {
                        self.is_pressed = true;
                        self.active_touch_id = Some(e.id);
                        self.last_position_world = Some(p);
                        self.steps.push(Pointer::Press(p));
                    }
                    TouchPhase::Moved if self.active_touch_id == Some(e.id) => {
                        self.last_position_world = Some(p);
                        self.steps.push(Pointer::Move(p));
                    }
                    TouchPhase::Ended | TouchPhase::Canceled
                        if self.active_touch_id == Some(e.id) =>
                    {
                        self.is_pressed = false;
                        self.active_touch_id = None;
                        self.last_position_world = Some(p);
                        self.steps.push(Pointer::Release);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

/// Reads the window events in arrival order (touches, clicks and cursor moves in one
/// stream), so a press, its moves and its release in one tick stay apart ([077]).
fn handle_input_events(
    mut window_events: MessageReader<WindowEvent>,
    mut input_state: ResMut<InputState>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (camera, camera_transform) = *camera;
    let mut cursor = window.cursor_position();
    for event in window_events.read() {
        input_state.record(event, &mut cursor, |pos| {
            to_field(camera, camera_transform, pos)
        });
    }
}

/// Runs the line logic once per pointer step of this tick, then sends this tick's line
/// messages.
#[allow(clippy::too_many_arguments)]
pub fn handle_input(
    mut input_state: ResMut<InputState>,
    mut user_path: ResMut<UserPath>,
    free: Res<FreeRegion>,
    mut cl_events: MessageWriter<events::ClearLines>,
    mut b_events: MessageWriter<events::BuildPolygon>,
    mut dl_events: MessageWriter<events::DrawLine>,
    mut wall_events: MessageWriter<WallEvent>,
    mut input_pos_events: MessageWriter<events::InputPosition>,
) {
    if user_path.is_drawing
        && let Some(position) = input_state.last_position_world
    {
        input_pos_events.write(events::InputPosition(position));
    }

    let (mut clear, mut draws) = (false, Vec::new());
    for step in std::mem::take(&mut input_state.steps) {
        let user_position = match step {
            Pointer::Press(p) => {
                if let Some(start) = start_point(&free, p) {
                    info!("Starting drawing");
                    user_path.reset();
                    clear_lines(&mut clear, &mut draws);
                    wall_events.write(WallEvent::Bloom);
                    user_path.is_drawing = true;

                    user_path.path.push(start);
                }
                continue;
            }
            Pointer::Release => {
                if user_path.is_drawing {
                    clear_lines(&mut clear, &mut draws);
                    wall_events.write(WallEvent::Reset);
                }
                user_path.reset();
                continue;
            }
            Pointer::Move(p) if user_path.is_drawing => p,
            Pointer::Move(_) => continue,
        };

        let last_position = *user_path
            .path
            .last()
            .expect("Path shouldn't be empty if drawing");

        if is_approximately_equal(last_position, user_position) {
            continue;
        }

        // Until the line leaves the boundary, its start follows the finger.
        if user_path.path.len() == 1
            && let Some(start) = start_point(&free, user_position)
        {
            user_path.path[0] = start;
            continue;
        }

        let end = line_end(&free, last_position, user_position);
        let next = end.map_or(user_position, |(kept, _)| kept);
        let crosses = would_create_intersection(&user_path.path, next)
            || end.is_some_and(|(kept, snap)| {
                would_create_intersection(&[user_path.path.as_slice(), &[kept]].concat(), snap)
            });
        if crosses {
            user_path.reset();
            wall_events.write(WallEvent::Reset);
            clear_lines(&mut clear, &mut draws);

            // Reset pressed state because the action failed/was cancelled
            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            continue;
        }

        // Complete path
        if let Some((kept, snap)) = end {
            user_path.path.extend([kept, snap]);

            clear_lines(&mut clear, &mut draws);
            b_events.write(events::BuildPolygon(user_path.path.clone()));
            wall_events.write(WallEvent::Reset);
            user_path.reset();

            input_state.is_pressed = false;
            input_state.active_touch_id = None;
            info!("Valid path created from wall to wall");
            continue;
        }

        // Draw line segment in open space
        draws.push(events::DrawLine(vec![last_position, user_position]));
        user_path.path.push(user_position);
    }

    if clear {
        cl_events.write(events::ClearLines);
    }
    dl_events.write_batch(draws);
}

/// A clear drops the segments drawn before it in this tick: `handle_clear_lines` cannot see
/// segments spawned in the same tick, so they would stay ([077]).
fn clear_lines(clear: &mut bool, draws: &mut Vec<events::DrawLine>) {
    *clear = true;
    draws.clear();
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

/// Where a line starts for a finger at `p`: the nearest point of the free boundary, when
/// `p` is on a wall or a claimed piece (outside the free region) or near the boundary.
fn start_point(free: &[Vec2], p: Vec2) -> Option<Vec2> {
    let edge = locate(free, p).2;
    (!is_point_in_polygon(&p, free) || edge.distance(p) < START_MARGIN).then_some(edge)
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
    use bevy::input::touch::TouchInput;

    /// The box with the part below y = -300 claimed.
    fn cut_box() -> FreeRegion {
        FreeRegion(vec![
            Vec2::new(-190., -300.),
            Vec2::new(190., -300.),
            Vec2::new(190., 335.),
            Vec2::new(-190., 335.),
        ])
    }

    /// A world with `free` and the messages `handle_input` writes.
    fn line_world(free: FreeRegion) -> World {
        let mut world = World::new();
        world.init_resource::<Messages<events::ClearLines>>();
        world.init_resource::<Messages<events::BuildPolygon>>();
        world.init_resource::<Messages<events::DrawLine>>();
        world.init_resource::<Messages<events::InputPosition>>();
        world.init_resource::<Messages<WallEvent>>();
        world.init_resource::<UserPath>();
        world.init_resource::<InputState>();
        world.insert_resource(free);
        world
    }

    /// Records these touch events (world positions) as one tick, then runs the line logic.
    fn one_tick(world: &mut World, touches: &[(TouchPhase, f32, f32)]) {
        use bevy::ecs::system::RunSystemOnce;

        let mut input = world.resource_mut::<InputState>();
        for &(phase, x, y) in touches {
            let event = WindowEvent::TouchInput(TouchInput {
                phase,
                position: Vec2::new(x, y),
                window: Entity::PLACEHOLDER,
                force: None,
                id: 0,
            });
            input.record(&event, &mut None, Some);
        }
        world.run_system_once(handle_input).unwrap();
    }

    #[test]
    fn a_flick_with_press_moves_and_release_in_one_tick_claims() {
        use TouchPhase::*;

        let mut world = line_world(cut_box());
        let flick = [
            (Started, -210., 0.),
            (Moved, -100., 0.),
            (Moved, 210., 0.),
            (Ended, 210., 0.),
        ];
        one_tick(&mut world, &flick);
        let lines = world.resource::<Messages<events::BuildPolygon>>();
        assert_eq!(lines.len(), 1, "the release does not drop the last move");
        assert!(!world.resource::<UserPath>().is_drawing);
        let segments = world.resource::<Messages<events::DrawLine>>();
        assert_eq!(segments.len(), 0, "no segment of the claimed line is left");
    }

    #[test]
    fn a_press_and_moves_in_one_tick_start_at_the_wall() {
        use TouchPhase::*;

        let mut world = line_world(cut_box());
        one_tick(
            &mut world,
            &[(Started, -210., 0.), (Moved, -100., 0.), (Moved, 0., 0.)],
        );
        let path = world.resource::<UserPath>();
        assert!(path.is_drawing);
        assert_eq!(
            path.path,
            [
                Vec2::new(-190., 0.),
                Vec2::new(-100., 0.),
                Vec2::new(0., 0.)
            ]
        );
    }

    #[test]
    fn a_line_starts_on_walls_claims_and_near_the_boundary() {
        let free = cut_box().0;
        let on_wall = start_point(&free, Vec2::new(210., 5.));
        assert_eq!(on_wall, Some(Vec2::new(190., 5.)));
        let on_claim = start_point(&free, Vec2::new(-50., -340.));
        assert_eq!(on_claim, Some(Vec2::new(-50., -300.)));
        let near_wall = start_point(&free, Vec2::new(175., 5.));
        assert_eq!(near_wall, Some(Vec2::new(190., 5.)));
        let in_corner = start_point(&free, Vec2::new(-220., 380.));
        assert_eq!(in_corner, Some(Vec2::new(-190., 335.)));
        assert_eq!(start_point(&free, Vec2::new(165., 5.)), None);
        assert_eq!(start_point(&free, Vec2::new(0., 0.)), None);
    }

    #[test]
    fn a_drag_from_a_claim_to_a_claim_sends_one_line_between_the_claimed_edges() {
        use crate::game::territory::{signed_area, split};
        use bevy::ecs::system::RunSystemOnce;

        let free = cut_box();
        let mut world = line_world(free.clone());
        let drag = [
            (-100., -340.),
            (-100., -310.),
            (-100., -250.),
            (-100., -200.),
            (100., -200.),
            (100., -250.),
            (100., -340.),
        ];
        for (k, (x, y)) in drag.into_iter().enumerate() {
            let p = Vec2::new(x, y);
            let step = if k == 0 {
                Pointer::Press(p)
            } else {
                Pointer::Move(p)
            };
            world.resource_mut::<InputState>().steps.push(step);
            world.run_system_once(handle_input).unwrap();
        }

        let lines: Vec<_> = world
            .resource_mut::<Messages<events::BuildPolygon>>()
            .drain()
            .collect();
        assert_eq!(lines.len(), 1);
        let path = &lines[0].0;
        assert_eq!(
            path[0],
            Vec2::new(-100., -300.),
            "the start slid onto the claim"
        );
        assert!(path.last().unwrap().distance(Vec2::new(100., -300.)) < 1e-3);
        let [_, notch] = split(&free, path);
        assert!((signed_area(&notch) - 200. * 100.).abs() < 0.1);
        assert_eq!(world.resource::<Messages<events::DrawLine>>().len(), 4);
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
