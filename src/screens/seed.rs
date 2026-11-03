//! Play a seed ([067]): a keypad of the 32 code characters, DEL, PASTE, PLAY and BACK. A
//! keyboard types too (letters, digits, Backspace, Enter). PLAY starts a run with the
//! typed code's seed.

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use super::Screen;
use crate::{
    game::{
        seed::{self, CODE_LEN, MAX_CODE_LEN},
        setup::new_run,
    },
    share::{Paste, Pasted},
    theme::{
        FontAssets,
        palette::{BASE_GRAY, NEON_WHITE},
        widget::{self, Activate},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Typed>()
        .add_observer(on_pasted)
        .add_systems(OnEnter(Screen::Seed), setup)
        .add_systems(
            Update,
            (type_keys, show_typed)
                .chain()
                .run_if(in_state(Screen::Seed)),
        );
}

const TITLE_Y: f32 = 330.;
const CODE_Y: f32 = 235.;
const KEY_SIZE: Vec2 = Vec2::new(48., 56.);
const KEY_GAP: f32 = 5.;
const COLUMNS: usize = 8;
const FIRST_ROW_Y: f32 = 130.;
const EDIT_Y: f32 = -150.;
const EDIT_SIZE: Vec2 = Vec2::new(125., 64.);
const PLAY_Y: f32 = -245.;
const BACK_Y: f32 = -340.;

/// The typed code characters, upper case, no dashes.
#[derive(Resource, Default)]
struct Typed(String);

impl Typed {
    fn push(&mut self, c: char) {
        if self.0.len() < MAX_CODE_LEN
            && let Some(value) = seed::digit(c)
        {
            self.0.push(seed::char_of(value));
        }
    }
}

#[derive(Component)]
struct CodeText;

#[derive(Component)]
struct KeyChar(char);

fn setup(mut commands: Commands, fonts: Res<FontAssets>, mut typed: ResMut<Typed>) {
    typed.0.clear();
    let font = fonts.main.clone();
    let text = |text: &str, size: f32, color: Color, y: f32| {
        (
            DespawnOnExit(Screen::Seed),
            Text2d::new(text),
            TextFont {
                font: font.clone().into(),
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(color),
            Transform::from_xyz(0., y, 1.),
        )
    };
    commands.spawn(text("PLAY SEED", 54., NEON_WHITE, TITLE_Y));
    commands.spawn((CodeText, text(&shown(""), 48., NEON_WHITE, CODE_Y)));

    let step = KEY_SIZE + KEY_GAP;
    let left = -(COLUMNS as f32 - 1.) / 2. * step.x;
    for value in 0..32 {
        let (column, row) = (value as usize % COLUMNS, value as usize / COLUMNS);
        let position = Vec2::new(
            left + column as f32 * step.x,
            FIRST_ROW_Y - row as f32 * step.y,
        );
        let c = seed::char_of(value);
        commands
            .spawn((
                DespawnOnExit(Screen::Seed),
                KeyChar(c),
                widget::button_at(&c.to_string(), font.clone(), position, KEY_SIZE),
            ))
            .observe(press_key);
    }

    let half = (EDIT_SIZE.x + KEY_GAP * 2.) / 2.;
    commands
        .spawn((
            DespawnOnExit(Screen::Seed),
            widget::button_at("DEL", font.clone(), Vec2::new(-half, EDIT_Y), EDIT_SIZE),
        ))
        .observe(|_: On<Activate>, mut typed: ResMut<Typed>| {
            typed.0.pop();
        });
    commands
        .spawn((
            DespawnOnExit(Screen::Seed),
            widget::button_at("PASTE", font.clone(), Vec2::new(half, EDIT_Y), EDIT_SIZE),
        ))
        .observe(|_: On<Activate>, mut commands: Commands| commands.trigger(Paste));
    commands
        .spawn((
            DespawnOnExit(Screen::Seed),
            widget::button("PLAY", font.clone(), PLAY_Y),
        ))
        .observe(
            |_: On<Activate>,
             mut commands: Commands,
             typed: Res<Typed>,
             next: ResMut<NextState<Screen>>| {
                play(&mut commands, &typed, next);
            },
        );
    commands
        .spawn((
            DespawnOnExit(Screen::Seed),
            widget::button("BACK", font, BACK_Y),
        ))
        .observe(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::Title);
        });
}

/// The typed characters in groups, with blanks up to a full code.
fn shown(typed: &str) -> String {
    let blanks = CODE_LEN.saturating_sub(typed.len());
    seed::group(&format!("{typed}{}", "_".repeat(blanks)))
}

fn press_key(activate: On<Activate>, keys: Query<&KeyChar>, mut typed: ResMut<Typed>) {
    if let Ok(key) = keys.get(activate.entity) {
        typed.push(key.0);
    }
}

/// A typed code that is not one does nothing.
fn play(commands: &mut Commands, typed: &Typed, mut next: ResMut<NextState<Screen>>) {
    if let Some(seed) = seed::parse(&typed.0) {
        info!("playing seed {} ({seed})", seed::code(seed));
        new_run(commands, seed);
        next.set(Screen::Gameplay);
    }
}

fn on_pasted(pasted: On<Pasted>, screen: Res<State<Screen>>, mut typed: ResMut<Typed>) {
    if *screen.get() == Screen::Seed
        && let Some(code) = seed::find(&pasted.0)
    {
        typed.0 = code;
    }
}

fn type_keys(
    mut commands: Commands,
    mut keys: MessageReader<KeyboardInput>,
    mut typed: ResMut<Typed>,
    next: ResMut<NextState<Screen>>,
) {
    let mut enter = false;
    for key in keys.read().filter(|k| k.state == ButtonState::Pressed) {
        match &key.logical_key {
            Key::Character(text) => text.chars().for_each(|c| typed.push(c)),
            Key::Backspace => {
                typed.0.pop();
            }
            Key::Enter => enter = true,
            _ => {}
        }
    }
    if enter {
        play(&mut commands, &typed, next);
    }
}

fn show_typed(typed: Res<Typed>, mut code: Query<(&mut Text2d, &mut TextColor), With<CodeText>>) {
    if !typed.is_changed() {
        return;
    }
    let color = if seed::parse(&typed.0).is_some() {
        NEON_WHITE
    } else {
        BASE_GRAY
    };
    for (mut text, mut text_color) in &mut code {
        text.0 = shown(&typed.0);
        text_color.0 = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_keeps_code_characters_and_shows_blanks() {
        let mut typed = Typed::default();
        for c in "k3-m9 o!".chars() {
            typed.push(c);
        }
        assert_eq!(typed.0, "K3M90");
        assert_eq!(shown(&typed.0), "K3M9-0___");
        assert_eq!(shown(""), "____-____");
        for _ in 0..20 {
            typed.push('Z');
        }
        assert_eq!(typed.0.len(), MAX_CODE_LEN);
    }
}
