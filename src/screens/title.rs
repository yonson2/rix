use bevy::{
    input::{ButtonState, mouse::MouseButtonInput, touch::TouchPhase},
    prelude::*,
};

use crate::{
    common::Transition,
    screens::Screen,
    shape::{self, Shape},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, NEON_PURPLE, NEON_WHITE, PURPLE},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), setup);
    app.add_systems(
        Update,
        (flicker_text, on_input).run_if(in_state(Screen::Title)),
    );
}

#[derive(Component)]
struct MenuComponent;

fn setup(mut commands: Commands, fonts: Res<FontAssets>) {
    let font_size = 119.;

    let shape = Shape::new(shape::rounded_rectangle(
        Vec2::new(font_size * 2., font_size * 1.5),
        10.,
    ))
    .fill(PURPLE)
    .stroke(NEON_PURPLE, 3.0);

    commands.spawn((
        MenuComponent,
        DespawnOnExit(Screen::Title),
        shape,
        Transform::from_xyz(0., 0., 0.),
    ));
    commands.spawn((
        MenuComponent,
        DespawnOnExit(Screen::Title),
        Text2d::new("RIX"),
        TextFont {
            font: fonts.main.clone().into(),
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(Color::NONE),
        Transform::from_xyz(0., 0., 1.),
        Transition(Timer::from_seconds(2.6, TimerMode::Once)),
    ));

    commands.spawn((
        MenuComponent,
        DespawnOnExit(Screen::Title),
        Text2d::new("Tap to start playing."),
        TextFont {
            // font,
            font_size: FontSize::Px(25.),
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
            if let Some(stroke) = &mut shape.stroke {
                stroke.color = current_color;
            }
        }

        if transition.0.is_finished() {
            commands.entity(entity).remove::<Transition>();
        }
    }
}

fn on_input(
    mut next_state: ResMut<NextState<Screen>>,
    mut mouse_button_events: MessageReader<MouseButtonInput>,
    mut touch_events: MessageReader<TouchInput>,
) {
    for event in mouse_button_events.read() {
        if event.button == MouseButton::Left
            && let ButtonState::Pressed = event.state
        {
            next_state.set(Screen::Gameplay);
        }
    }

    for event in touch_events.read() {
        if let TouchPhase::Started = event.phase {
            next_state.set(Screen::Gameplay);
        }
    }
}
