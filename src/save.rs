//! The save file ([048]): the best score, lifetime stats and the settings, as JSON in the
//! app's data directory. It loads when the app starts (no file or a bad one: defaults) and
//! is written when it changes: at the start, at a game over and on a settings tap. A shot
//! run ([006]) neither reads nor writes it, unless `RIX_SAVE` names a file.

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    audio::Volumes,
    crt::PostProcessSettings,
    game::{events::DrawPolygon, setup::Run, skins::Skins, state::IsPaused},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    let file = save_file();
    let save = file.as_deref().map(load).unwrap_or_default();
    // A shot run keeps its picture: no tutorial and no tips unless `RIX_SAVE` names a file
    // ([049], [070]).
    #[cfg(feature = "dev_tools")]
    let save = if file.is_none() && crate::dev_tools::is_shot_run() {
        Save {
            tutorial_done: true,
            met: crate::game::tutorial::all_met(),
            ..save
        }
    } else {
        save
    };
    // `RIX_SKINS=laser,brick` shows those skins ([078]).
    #[cfg(feature = "dev_tools")]
    let save = match crate::dev_tools::forced_skins() {
        Some(skins) => Save { skins, ..save },
        None => save,
    };
    app.insert_resource(save)
        .insert_resource(SaveFile(file))
        .init_resource::<RunStats>()
        .add_systems(OnEnter(Screen::Gameplay), |mut stats: ResMut<RunStats>| {
            *stats = RunStats::default();
        })
        .add_systems(
            Update,
            (
                count_claims.run_if(in_state(Screen::Gameplay)),
                count_time.run_if(in_state(IsPaused::Running)),
                apply_settings.run_if(resource_changed::<Save>),
            ),
        )
        .add_systems(Last, write.run_if(resource_changed::<Save>));
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Save {
    pub best: u128,
    pub stats: Stats,
    pub settings: Settings,
    /// The first-run tutorial ended: a first claim or SKIP ([049]).
    pub tutorial_done: bool,
    /// The enemy and item kinds whose tip the player has seen ([070]).
    pub met: Vec<String>,
    /// The best level reached; it unlocks skins ([078]) and the soundtrack ([089]).
    pub best_level: usize,
    pub skins: Skins,
}

/// The soundtrack is a reward (D26, [089]): below this best level only the sound effects
/// play. Once it opens, the music still waits at 0 % for the player to turn it on.
pub const SOUNDTRACK_LEVEL: usize = 3;

impl Save {
    pub fn soundtrack_open(&self) -> bool {
        self.best_level >= SOUNDTRACK_LEVEL
    }
}

/// Lifetime totals over finished runs.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Stats {
    pub runs: u64,
    pub claims: u64,
    pub kills: u64,
    /// Levels cleared.
    pub levels: u64,
    /// Seconds of play, pauses left out.
    pub seconds: f64,
}

/// Volumes and the CRT strength in percent, in `STEP` steps.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub music: u8,
    pub sound: u8,
    pub crt: u8,
    pub shake: bool,
    pub haptics: bool,
    /// The white flash of a new claim.
    pub flashing: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            music: 0,
            sound: 100,
            crt: 100,
            shake: true,
            haptics: true,
            flashing: true,
        }
    }
}

/// A percent setting goes down by this much a tap, and from 0 back to 100.
pub const STEP: u8 = 20;

pub fn step_down(percent: u8) -> u8 {
    percent.checked_sub(STEP).unwrap_or(100)
}

/// The claims and the play time of the run in progress.
#[derive(Resource, Default, Debug)]
pub struct RunStats {
    claims: u64,
    seconds: f64,
}

/// Adds a finished run to the save; true when it beat the best score.
pub fn record_run(save: &mut Save, run: &Run, played: &RunStats) -> bool {
    let new_best = run.points.0 > save.best;
    save.best = save.best.max(run.points.0);
    let stats = &mut save.stats;
    stats.runs += 1;
    stats.claims += played.claims;
    stats.kills += run.kill_count as u64;
    stats.levels += run.level.saturating_sub(1) as u64;
    stats.seconds += played.seconds;
    new_best
}

#[derive(Resource)]
struct SaveFile(Option<PathBuf>);

fn save_file() -> Option<PathBuf> {
    #[cfg(feature = "dev_tools")]
    {
        if let Some(path) = std::env::var_os("RIX_SAVE") {
            return Some(path.into());
        }
        if crate::dev_tools::is_shot_run() {
            return None;
        }
    }
    Some(data_dir()?.join("save.json"))
}

#[cfg(target_os = "android")]
fn data_dir() -> Option<PathBuf> {
    bevy::android::ANDROID_APP.get()?.internal_data_path()
}

/// iOS gives each app its own `HOME`.
#[cfg(any(target_os = "macos", target_os = "ios"))]
fn data_dir() -> Option<PathBuf> {
    let home = PathBuf::from(std::env::var_os("HOME")?);
    Some(home.join("Library/Application Support/rix"))
}

#[cfg(target_os = "windows")]
fn data_dir() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("APPDATA")?).join("rix"))
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn data_dir() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(std::env::var_os("HOME")?).join(".local/share")))?;
    Some(data.join("rix"))
}

/// The web build keeps no save yet.
#[cfg(target_family = "wasm")]
fn data_dir() -> Option<PathBuf> {
    None
}

fn load(path: &Path) -> Save {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Save::default();
    };
    match serde_json::from_str(&text) {
        Ok(save) => {
            info!("save loaded from {}", path.display());
            save
        }
        Err(err) => {
            warn!("save {} unreadable, using defaults: {err}", path.display());
            Save::default()
        }
    }
}

/// Through a temporary file, so a crash mid-write keeps the old save.
fn write(save: Res<Save>, file: Res<SaveFile>) {
    let Some(path) = &file.0 else {
        return;
    };
    let result = (|| {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(&*save)?)?;
        std::fs::rename(&tmp, path)
    })();
    match result {
        Ok(()) => info!("save written to {}", path.display()),
        Err(err) => warn!("save {} not written: {err}", path.display()),
    }
}

fn count_claims(mut claims: MessageReader<DrawPolygon>, mut stats: ResMut<RunStats>) {
    stats.claims += claims.read().count() as u64;
}

fn count_time(time: Res<Time>, mut stats: ResMut<RunStats>) {
    stats.seconds += time.delta_secs_f64();
}

fn apply_settings(
    save: Res<Save>,
    mut volumes: ResMut<Volumes>,
    mut crt: Query<&mut PostProcessSettings>,
) {
    let settings = save.settings;
    let music = if save.soundtrack_open() {
        settings.music
    } else {
        0
    };
    *volumes = Volumes {
        music: f32::from(music) / 100.,
        sfx: f32::from(settings.sound) / 100.,
    };
    // A shot run makes no sound, but it plays the sounds ([041]).
    #[cfg(feature = "dev_tools")]
    if crate::dev_tools::is_shot_run() {
        *volumes = Volumes { music: 0., sfx: 0. };
    }
    for mut post in &mut crt {
        post.intensity = f32::from(settings.crt) / 100.;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::setup::Points;

    #[test]
    fn a_save_survives_a_round_trip_and_a_partial_file_fills_in_defaults() {
        let save = Save {
            best: u128::MAX,
            stats: Stats {
                runs: 3,
                seconds: 12.5,
                ..default()
            },
            settings: Settings {
                crt: 40,
                haptics: false,
                ..default()
            },
            tutorial_done: true,
            met: vec!["Ball".into()],
            best_level: 12,
            skins: Skins {
                line: crate::game::skins::LineSkin::Laser,
                ..default()
            },
        };
        let text = serde_json::to_string_pretty(&save).unwrap();
        assert_eq!(serde_json::from_str::<Save>(&text).unwrap(), save);

        let old: Save = serde_json::from_str(r#"{"best": 7, "settings": {"music": 0}}"#).unwrap();
        assert_eq!(old.best, 7);
        assert_eq!(old.settings.music, 0);
        assert_eq!(old.settings.sound, 100);
        assert_eq!(old.stats, Stats::default());
        assert!(!old.tutorial_done, "an old save shows the tutorial once");
    }

    #[test]
    fn the_music_plays_only_once_the_soundtrack_is_open_and_turned_on() {
        use bevy::ecs::system::RunSystemOnce;

        let music = |save: Save| {
            let mut world = World::new();
            world.insert_resource(save);
            world.insert_resource(Volumes { music: 1., sfx: 1. });
            world.run_system_once(apply_settings).unwrap();
            world.resource::<Volumes>().music
        };
        let mut save = Save::default();
        assert_eq!(music(save.clone()), 0., "a fresh save: sound effects only");
        save.settings.music = 60;
        assert_eq!(music(save.clone()), 0., "locked, whatever the save holds");
        save.best_level = SOUNDTRACK_LEVEL;
        save.settings.music = 0;
        assert_eq!(music(save.clone()), 0., "open, but off until turned on");
        save.settings.music = 60;
        assert_eq!(music(save), 0.6);
    }

    #[test]
    fn a_file_is_written_and_read_back_and_a_bad_one_gives_defaults() {
        let dir = std::env::temp_dir().join(format!("rix-save-{}", std::process::id()));
        let path = dir.join("nested/save.json");
        let mut app = App::new();
        app.insert_resource(Save {
            best: 99,
            ..default()
        })
        .insert_resource(SaveFile(Some(path.clone())))
        .add_systems(Update, write);
        app.update();
        assert_eq!(load(&path).best, 99);

        std::fs::write(&path, "not json").unwrap();
        assert_eq!(load(&path), Save::default());
        assert_eq!(load(&dir.join("missing.json")), Save::default());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_run_adds_its_stats_and_only_a_higher_score_is_a_new_best() {
        let mut save = Save {
            best: 500,
            ..default()
        };
        let mut run = Run::with_seed(1);
        run.level = 3;
        run.kill_count = 4;
        run.points = Points(500);
        let played = RunStats {
            claims: 6,
            seconds: 30.,
        };
        assert!(
            !record_run(&mut save, &run, &played),
            "a tie is not a new best"
        );
        run.points = Points(501);
        assert!(record_run(&mut save, &run, &played));
        assert_eq!(save.best, 501);
        assert_eq!(
            save.stats,
            Stats {
                runs: 2,
                claims: 12,
                kills: 8,
                levels: 4,
                seconds: 60.,
            }
        );
    }

    #[test]
    fn a_percent_setting_steps_down_and_wraps_to_full() {
        assert_eq!(step_down(100), 80);
        assert_eq!(step_down(20), 0);
        assert_eq!(step_down(0), 100);
        assert_eq!(step_down(60), 40);
    }
}
