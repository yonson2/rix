use bevy::{
    input::{ButtonState, mouse::MouseButtonInput, touch::TouchPhase},
    prelude::*,
};
use bevy_prototype_lyon::{
    draw::Stroke,
    entity::Shape,
    prelude::{ShapeBuilder, ShapeBuilderBase},
    shapes::{self, BorderRadii},
};

use crate::{
    colors::{BASE_GRAY, NEON_PURPLE, NEON_WHITE, PURPLE},
    common::Transition,
    rix::{despawn_screen, state::AppState},
};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup);
        app.add_systems(
            Update,
            (flicker_text, on_input).run_if(in_state(AppState::Menu)),
        );
        app.add_systems(OnExit(AppState::Menu), despawn_screen::<MenuComponent>);
    }
}

#[derive(Component)]
struct MenuComponent;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    //TODO: load assets in load step.

    let font_size = 119.;

    // Use a regular rounded rectangle instead of a rounded polygon
    let rounded_rectangle = shapes::Rectangle {
        extents: Vec2::new(font_size * 2., font_size * 1.5),
        origin: shapes::RectangleOrigin::Center,
        radii: Some(BorderRadii::single(10.)),
    };

    let stroke = Stroke::new(NEON_PURPLE, 3.0);
    let shape = ShapeBuilder::with(&rounded_rectangle)
        .fill(PURPLE)
        .stroke(stroke)
        .build();

    let font = asset_server.load("fonts/main.ttf");

    commands.spawn((MenuComponent, shape, Transform::from_xyz(0., 0., 0.)));
    commands.spawn((
        MenuComponent,
        Text2d::new("RIX"),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(Color::NONE),
        Transform::from_xyz(0., 0., 1.),
        Transition(Timer::from_seconds(2.6, TimerMode::Once)),
    ));

    commands.spawn((
        MenuComponent,
        Text2d::new("Tap to start playing."),
        TextFont {
            // font,
            font_size: 25.,
            ..default()
        },
        TextColor(BASE_GRAY),
        Transform::from_xyz(0., -font_size, 1.),
    ));
}

fn flicker_text(
    mut commands: Commands,
    time: Res<Time>,
    mut transition_q: Query<(Entity, &mut Transition), With<MenuComponent>>,
    mut shape_q: Query<&mut Shape>,
    mut text_q: Query<&mut TextColor, With<Transition>>,
) {
    for (entity, mut transition) in transition_q.iter_mut() {
        transition.0.tick(time.delta());
        let t = transition.0.fraction();
        let easing = EasingCurve::new(0.0, 1.0, EaseFunction::BounceInOut);
        let alpha = easing.sample(t).unwrap();
        let mut current_color = NEON_WHITE;
        current_color.set_alpha(alpha);

        for mut text_color in &mut text_q {
            text_color.0 = current_color;
        }
        for mut shape in &mut shape_q {
            shape.stroke = shape.stroke.map(|mut s| {
                s.color = current_color;
                s
            });
        }

        if transition.0.finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

fn on_input(
    mut next_state: ResMut<NextState<AppState>>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    mut touch_events: EventReader<TouchInput>,
) {
    for event in mouse_button_events.read() {
        if event.button == MouseButton::Left {
            if let ButtonState::Pressed = event.state {
                next_state.set(AppState::InGame);
            }
        }
    }

    for event in touch_events.read() {
        if let TouchPhase::Started = event.phase {
            next_state.set(AppState::InGame);
        }
    }
}
