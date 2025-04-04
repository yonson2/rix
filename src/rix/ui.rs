use bevy::prelude::*;

use super::{
    RixComponent,
    setup::LevelInfo,
    state::AppState,
    wall::{WALL_THICKNESS, WALL_THICKNESS_TOP},
};

const MARGIN: Val = Val::Px(6.);
const MARGIN_TOP: Val = Val::Px(WALL_THICKNESS_TOP * 1.35);
const MARGIN_SIDES: Val = Val::Px(WALL_THICKNESS * 1.5);
const POINTS_FONT_SIZE: f32 = 30.;
const SMALL_FONT_SIZE: f32 = 20.;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_ui);
        app.add_systems(
            Update,
            update_ui.run_if(resource_changed::<LevelInfo>.and(in_state(AppState::InGame))),
        );
    }
}

#[derive(Component)]
struct UiLevel;
#[derive(Component)]
struct UiPoints;
#[derive(Component)]
struct UiProgress;

fn setup_ui(mut commands: Commands, level_info: Res<LevelInfo>, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/main.ttf");
    commands
        .spawn((
            RixComponent,
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Start,
                padding: UiRect::axes(MARGIN_SIDES, MARGIN_TOP),
                row_gap: MARGIN,
                ..Default::default()
            },
            BackgroundColor(Color::NONE),
            PickingBehavior {
                should_block_lower: false,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder.spawn(Node { ..default() }).with_child((
                Text::new(format!("Level: {}", level_info.level)),
                TextFont {
                    font: font.clone(),
                    font_size: SMALL_FONT_SIZE,
                    ..default()
                },
                TextColor::WHITE,
                UiLevel,
            ));

            builder
                .spawn(Node {
                    margin: UiRect::top(Val::Vh(1.)),
                    ..default()
                })
                .with_child((
                    Text::new(level_info.points.to_string()),
                    TextFont {
                        font: font.clone(),
                        font_size: POINTS_FONT_SIZE,
                        ..default()
                    },
                    TextColor::WHITE,
                    UiPoints,
                ));

            builder.spawn(Node { ..default() }).with_child((
                Text::new(format!(
                    "{}/{}%",
                    (level_info.current * 100.) as u8,
                    (level_info.target * 100.) as u8
                )),
                TextFont {
                    font,
                    font_size: SMALL_FONT_SIZE,
                    ..default()
                },
                TextColor::WHITE,
                UiProgress,
            ));
        });
}

#[allow(clippy::type_complexity)]
fn update_ui(
    mut level_q: Query<&mut Text, (With<UiLevel>, Without<UiPoints>, Without<UiProgress>)>,
    mut points_q: Query<&mut Text, (With<UiPoints>, Without<UiLevel>, Without<UiProgress>)>,
    mut progress_q: Query<&mut Text, (With<UiProgress>, Without<UiPoints>, Without<UiLevel>)>,
    level_info: Res<LevelInfo>,
) {
    if let Ok(mut text) = level_q.get_single_mut() {
        **text = format!("Level: {}", level_info.level);
    }

    if let Ok(mut text) = points_q.get_single_mut() {
        **text = level_info.points.to_string();
    }

    if let Ok(mut text) = progress_q.get_single_mut() {
        **text = format!(
            "{}/{}%",
            (level_info.current * 100.) as u8,
            (level_info.target * 100.) as u8
        );
    }
}
