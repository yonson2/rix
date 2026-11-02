use bevy::{
    input::{ButtonState, mouse::MouseButtonInput, touch::TouchPhase},
    prelude::*,
};

use crate::colors::{BASE_GRAY, BURGUNDY};

use super::{setup::LevelInfo, state::Screen};

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::GameOver), setup);
        app.add_systems(Update, on_input.run_if(in_state(Screen::GameOver)));
    }
}

const FONT_SIZE_TITLE: FontSize = FontSize::Px(70.0);
const FONT_SIZE_SCORE: FontSize = FontSize::Px(45.0);
const FONT_SIZE_STATS: FontSize = FontSize::Px(30.0);

const COLOR_TITLE: Color = BURGUNDY;
const COLOR_SCORE: Color = Color::WHITE;
const COLOR_STATS: Color = Color::WHITE;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, level_info: Res<LevelInfo>) {
    let font = asset_server.load("fonts/main.ttf");

    commands
        .spawn((
            DespawnOnExit(Screen::GameOver),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("YOU DIED!"),
                TextFont {
                    font: font.into(),
                    font_size: FONT_SIZE_TITLE,
                    ..default()
                },
                TextColor(COLOR_TITLE),
            ));

            parent.spawn((
                Text::new(format!("Score: {}", level_info.points)),
                TextFont {
                    font_size: FONT_SIZE_SCORE,
                    ..default()
                },
                TextColor(COLOR_SCORE),
            ));

            parent.spawn((
                Text::new(format!("Level Reached: {}", level_info.level)),
                TextFont {
                    font_size: FONT_SIZE_STATS,
                    ..default()
                },
                TextColor(COLOR_STATS),
            ));

            parent.spawn((
                Text::new(format!("Enemies Killed: {}", level_info.kill_count)),
                TextFont {
                    font_size: FONT_SIZE_STATS,
                    ..default()
                },
                TextColor(COLOR_STATS),
            ));

            let mut color = BASE_GRAY;
            color.set_alpha(0.5);
            parent.spawn((
                Text::new("Tap to play again."),
                TextFont {
                    font_size: FONT_SIZE_STATS,
                    ..default()
                },
                TextColor(color),
            ));
        });
}

fn on_input(
    mut next_state: ResMut<NextState<Screen>>,
    mut mouse_button_events: MessageReader<MouseButtonInput>,
    mut touch_events: MessageReader<TouchInput>,
    mut level_info: ResMut<LevelInfo>,
) {
    for event in mouse_button_events.read() {
        if event.button == MouseButton::Left
            && let ButtonState::Pressed = event.state
        {
            next_state.set(Screen::Gameplay);
            level_info.reset();
        }
    }

    for event in touch_events.read() {
        if let TouchPhase::Started = event.phase {
            next_state.set(Screen::Gameplay);
            level_info.reset();
        }
    }
}
