//! The first-run tutorial ([049]): on level 1, until the player's first claim, a ghost
//! finger shows the drag, then one line of text per rule as it first happens. It does not
//! stop the game, so a seed plays the same with it. The first claim or SKIP ends it for good
//! (`Save::tutorial_done`). A pause hides it; the resume shows it again.

use bevy::prelude::*;

use crate::{
    game::{
        AppSystems, PausableSystems,
        events::DrawPolygon,
        polygon::TempLineSegment,
        setup::{LevelProgress, Run},
        state::{IsPaused, Phase},
        wall::Field,
    },
    save::Save,
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::NEON_WHITE,
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(IsPaused::Running), show)
        .add_systems(OnExit(Phase::Playing), |mut commands: Commands| {
            commands.remove_resource::<Tutorial>();
        })
        .add_systems(
            Update,
            (advance, move_finger)
                .run_if(resource_exists::<Tutorial>)
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

/// A rule stays this long before the next one when no event moves it on.
const TIP_SECS: f32 = 3.;
const TEXT_Y: f32 = 200.;
const FINGER_SIZE: f32 = 44.;
/// The ghost finger crosses the field in `DRAG_SECS`, then hides until `CYCLE_SECS`.
const DRAG_SECS: f32 = 1.6;
const CYCLE_SECS: f32 = 2.4;
const SKIP_SIZE: Vec2 = Vec2::new(96., 40.);
const Z: f32 = 10.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Drag,
    Line,
    Claim,
    Target,
}

impl Step {
    fn text(self, target: f32) -> String {
        match self {
            Step::Drag => "DRAG A LINE\nFROM WALL TO WALL".into(),
            Step::Line => "AN ENEMY THAT TOUCHES\nTHE LINE KILLS YOU".into(),
            Step::Claim => "THE SIDE WITHOUT\nENEMIES IS YOURS".into(),
            Step::Target => format!("CLAIM {:.0}%\nTO CLEAR THE LEVEL", target * 100.),
        }
    }

    /// The step after this frame; `None` ends the tutorial.
    fn next(self, drawing: bool, claimed: bool, tip_over: bool) -> Option<Step> {
        Some(match self {
            Step::Drag | Step::Line if claimed => Step::Claim,
            Step::Drag if drawing => Step::Line,
            Step::Claim if tip_over => Step::Target,
            Step::Target if tip_over => return None,
            step => step,
        })
    }
}

#[derive(Resource)]
struct Tutorial {
    step: Step,
    tip: Timer,
}

#[derive(Component)]
struct Overlay;

#[derive(Component)]
struct TipText;

#[derive(Component)]
struct GhostFinger;

/// Starts the tutorial on level 1 of a player who has not claimed yet, and shows it again
/// after a pause.
fn show(
    mut commands: Commands,
    tutorial: Option<Res<Tutorial>>,
    save: Res<Save>,
    run: Res<Run>,
    progress: Res<LevelProgress>,
    field: Res<Field>,
    fonts: Res<FontAssets>,
) {
    let step = match tutorial {
        Some(tutorial) => tutorial.step,
        None if run.level == 1 && !save.tutorial_done => {
            commands.insert_resource(Tutorial {
                step: Step::Drag,
                tip: Timer::from_seconds(TIP_SECS, TimerMode::Once),
            });
            Step::Drag
        }
        None => return,
    };
    let corners = field.inner_corners();
    let mut overlay = commands.spawn((
        Overlay,
        DespawnOnExit(IsPaused::Running),
        Transform::from_xyz(0., 0., Z),
        Visibility::default(),
    ));
    overlay.with_children(|parent| {
        parent.spawn((
            TipText,
            Text2d::new(step.text(progress.target)),
            TextFont {
                font: fonts.main.clone().into(),
                font_size: FontSize::Px(24.),
                ..default()
            },
            TextLayout::justify(Justify::Center),
            TextColor(NEON_WHITE),
            Transform::from_xyz(0., corners.top_left.y - TEXT_Y, 0.),
        ));
        if step == Step::Drag {
            parent.spawn((
                GhostFinger,
                Shape::new(shape::rounded_rectangle(
                    Vec2::splat(FINGER_SIZE),
                    FINGER_SIZE / 2.,
                ))
                .fill(NEON_WHITE.with_alpha(0.2))
                .stroke(NEON_WHITE.with_alpha(0.7), 3.),
                Transform::default(),
                Visibility::Hidden,
            ));
        }
        let skip = corners.top_right + Vec2::new(-SKIP_SIZE.x / 2. - 8., -SKIP_SIZE.y / 2. - 8.);
        parent
            .spawn(widget::button_at(
                "SKIP",
                fonts.main.clone(),
                skip,
                SKIP_SIZE,
            ))
            .observe(skip_tutorial);
    });
}

fn skip_tutorial(
    _: On<Activate>,
    mut commands: Commands,
    mut save: ResMut<Save>,
    overlay: Query<Entity, With<Overlay>>,
) {
    save.tutorial_done = true;
    commands.remove_resource::<Tutorial>();
    overlay.iter().for_each(|e| commands.entity(e).despawn());
}

fn advance(
    mut commands: Commands,
    time: Res<Time>,
    mut tutorial: ResMut<Tutorial>,
    mut save: ResMut<Save>,
    mut claims: MessageReader<DrawPolygon>,
    line: Query<(), With<TempLineSegment>>,
    progress: Res<LevelProgress>,
    overlay: Query<Entity, With<Overlay>>,
    finger: Query<Entity, With<GhostFinger>>,
    mut text: Query<&mut Text2d, With<TipText>>,
) {
    let claimed = claims.read().count() > 0;
    if claimed && !save.tutorial_done {
        save.tutorial_done = true;
    }
    let tip_over = tutorial.tip.tick(time.delta()).is_finished();
    match tutorial.step.next(!line.is_empty(), claimed, tip_over) {
        None => {
            commands.remove_resource::<Tutorial>();
            overlay.iter().for_each(|e| commands.entity(e).despawn());
        }
        Some(step) if step != tutorial.step => {
            tutorial.step = step;
            tutorial.tip.reset();
            finger.iter().for_each(|e| commands.entity(e).despawn());
            for mut text in &mut text {
                text.0 = step.text(progress.target);
            }
        }
        Some(_) => {}
    }
}

/// Draws only: the ghost finger drags from the left wall to the right wall, over and over.
fn move_finger(
    time: Res<Time>,
    field: Res<Field>,
    mut finger: Query<(&mut Transform, &mut Visibility), With<GhostFinger>>,
) {
    let corners = field.inner_corners();
    let x = drag_x(time.elapsed_secs(), corners.top_left.x, corners.top_right.x);
    for (mut transform, mut visibility) in &mut finger {
        *visibility = if x.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        transform.translation.x = x.unwrap_or_default();
    }
}

/// Where the ghost finger is at `secs` between the walls at `left` and `right`; `None`
/// while it hides between two drags.
fn drag_x(secs: f32, left: f32, right: f32) -> Option<f32> {
    let t = secs % CYCLE_SECS / DRAG_SECS;
    (t <= 1.).then_some(left + (right - left) * t)
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{screens::Screen, theme::widget::NeonButton};

    fn app(save: Save, level: usize) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin, plugin))
            .add_message::<DrawPolygon>()
            .init_state::<Screen>()
            .add_sub_state::<Phase>()
            .add_sub_state::<IsPaused>()
            .insert_state(Screen::Gameplay)
            .init_resource::<Field>()
            .insert_resource(save)
            .insert_resource(Run {
                level,
                ..Run::with_seed(1)
            })
            .insert_resource(LevelProgress {
                target: 0.75,
                current: 0.,
            })
            .insert_resource(FontAssets {
                main: Handle::default(),
            });
        app.update();
        app
    }

    fn shown(app: &mut App) -> bool {
        let world = app.world_mut();
        world.query::<&Overlay>().iter(world).count() > 0
    }

    fn set_paused(app: &mut App, paused: IsPaused) {
        app.world_mut()
            .resource_mut::<NextState<IsPaused>>()
            .set(paused);
        app.update();
    }

    #[test]
    fn a_fresh_save_shows_it_a_pause_hides_it_and_skip_ends_it_for_good() {
        let mut app = app(Save::default(), 1);
        assert!(shown(&mut app));
        set_paused(&mut app, IsPaused::Paused);
        assert!(!shown(&mut app));
        set_paused(&mut app, IsPaused::Running);
        assert!(shown(&mut app), "the resume shows it again");

        let world = app.world_mut();
        let skip = world
            .query_filtered::<Entity, With<NeonButton>>()
            .single(world)
            .unwrap();
        world.trigger(Activate { entity: skip });
        app.update();
        assert!(!shown(&mut app));
        assert!(app.world().resource::<Save>().tutorial_done);
        set_paused(&mut app, IsPaused::Paused);
        set_paused(&mut app, IsPaused::Running);
        assert!(!shown(&mut app), "skipped for good");
    }

    #[test]
    fn a_done_tutorial_or_a_later_level_shows_nothing() {
        let done = Save {
            tutorial_done: true,
            ..default()
        };
        assert!(!shown(&mut app(done, 1)));
        assert!(!shown(&mut app(Save::default(), 2)));
    }

    #[test]
    fn a_line_start_and_a_claim_bring_their_rules_then_the_target_ends_it() {
        let (idle, drawing, claimed, tip_over) = (
            (false, false, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
        );
        let next = |step: Step, (d, c, t): (bool, bool, bool)| step.next(d, c, t);

        assert_eq!(next(Step::Drag, idle), Some(Step::Drag));
        assert_eq!(
            next(Step::Drag, tip_over),
            Some(Step::Drag),
            "it waits for a line"
        );
        assert_eq!(next(Step::Drag, drawing), Some(Step::Line));
        assert_eq!(next(Step::Line, idle), Some(Step::Line), "a cancelled line");
        assert_eq!(next(Step::Line, claimed), Some(Step::Claim));
        assert_eq!(next(Step::Drag, claimed), Some(Step::Claim));
        assert_eq!(next(Step::Claim, drawing), Some(Step::Claim));
        assert_eq!(next(Step::Claim, tip_over), Some(Step::Target));
        assert_eq!(next(Step::Target, tip_over), None);
        assert_eq!(Step::Target.text(0.75), "CLAIM 75%\nTO CLEAR THE LEVEL");
    }

    #[test]
    fn the_finger_crosses_from_wall_to_wall_then_hides() {
        assert_eq!(drag_x(0., -190., 190.), Some(-190.));
        assert_eq!(drag_x(DRAG_SECS / 2., -190., 190.), Some(0.));
        assert_eq!(drag_x(DRAG_SECS + 0.1, -190., 190.), None);
        assert_eq!(drag_x(CYCLE_SECS, -190., 190.), Some(-190.));
    }
}
