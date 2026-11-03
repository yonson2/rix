//! Music and sound effects ([041]), after bevy_new_2d's audio module ([026]): two
//! categories, each with its own volume. The music crossfades when the track changes, and
//! all sound stops while the app is suspended ([023]). The sound effects ([042]): the game
//! triggers `Sfx`; this module also follows the line, the death and the level clear.

use bevy::{audio::Volume, platform::collections::HashMap, prelude::*, window::AppLifecycle};

use crate::{
    asset_tracking::LoadResource,
    game::{
        polygon::TempLineSegment,
        state::{IsPaused, Phase},
    },
    screens::Screen,
    theme::widget::Activate,
};

const CROSSFADE_SECS: f32 = 1.;
/// Many balls bounce, and many Shooters fire, at once late in the game: one bounce sound
/// and one shot sound per this time at most.
const REPEAT_GAP_SECS: f32 = 0.08;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Volumes>()
        .init_resource::<MusicTrack>()
        .load_resource::<AudioAssets>()
        .add_observer(start_at_volume)
        .add_observer(tap)
        .add_observer(play_sfx)
        .add_systems(
            OnEnter(IsPaused::Dying),
            |mut commands: Commands, audio: Res<AudioAssets>| {
                commands.spawn(sound_effect(audio.death.clone()));
            },
        )
        .add_systems(
            OnEnter(Phase::LevelTransition),
            |mut commands: Commands, audio: Res<AudioAssets>| {
                commands.spawn(sound_effect(audio.level_clear.clone()));
            },
        )
        .add_systems(
            OnEnter(Screen::Title),
            |audio: Res<AudioAssets>, mut track: ResMut<MusicTrack>| {
                track.0 = Some(audio.menu.clone());
            },
        )
        .add_systems(
            OnEnter(Screen::Gameplay),
            |audio: Res<AudioAssets>, mut track: ResMut<MusicTrack>| {
                track.0 = Some(audio.game.clone());
            },
        )
        .add_systems(
            Update,
            (
                switch_music.run_if(resource_changed::<MusicTrack>),
                fade_music,
                pause_on_suspend,
                log_started,
            )
                .chain(),
        )
        .add_systems(Update, hum_while_drawing.run_if(in_state(Screen::Gameplay)));
}

/// The sounds, loaded once on the loading screen ([027]).
#[derive(Resource, Asset, Clone, TypePath)]
pub struct AudioAssets {
    #[dependency]
    menu: Handle<AudioSource>,
    #[dependency]
    game: Handle<AudioSource>,
    #[dependency]
    tap: Handle<AudioSource>,
    #[dependency]
    line_start: Handle<AudioSource>,
    #[dependency]
    line_hum: Handle<AudioSource>,
    #[dependency]
    claim: Handle<AudioSource>,
    #[dependency]
    eat: Handle<AudioSource>,
    #[dependency]
    bomb: Handle<AudioSource>,
    #[dependency]
    bounce: Handle<AudioSource>,
    #[dependency]
    shot: Handle<AudioSource>,
    #[dependency]
    hatch: Handle<AudioSource>,
    #[dependency]
    multiplier: Handle<AudioSource>,
    #[dependency]
    time_stop: Handle<AudioSource>,
    #[dependency]
    auto_line: Handle<AudioSource>,
    #[dependency]
    death: Handle<AudioSource>,
    #[dependency]
    level_clear: Handle<AudioSource>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let sfx = |name: &str| assets.load(format!("audio/sfx/{name}.ogg"));
        Self {
            menu: assets.load("audio/music/menu.ogg"),
            game: assets.load("audio/music/game.ogg"),
            tap: sfx("tap"),
            line_start: sfx("line_start"),
            line_hum: sfx("line_hum"),
            claim: sfx("claim"),
            eat: sfx("eat"),
            bomb: sfx("bomb"),
            bounce: sfx("bounce"),
            shot: sfx("shot"),
            hatch: sfx("hatch"),
            multiplier: sfx("multiplier"),
            time_stop: sfx("time_stop"),
            auto_line: sfx("auto_line"),
            death: sfx("death"),
            level_clear: sfx("level_clear"),
        }
    }
}

/// The volume of each category, from 0 to 1; the settings change it ([048]).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct Volumes {
    pub music: f32,
    pub sfx: f32,
}

impl Default for Volumes {
    fn default() -> Self {
        Self {
            music: 0.6,
            sfx: 1.,
        }
    }
}

/// The track that should play; the old one fades out as the new one fades in.
#[derive(Resource, Default)]
struct MusicTrack(Option<Handle<AudioSource>>);

/// A playing track and its fade: `gain` moves toward `target` in `CROSSFADE_SECS`.
#[derive(Component)]
struct Music {
    gain: f32,
    target: f32,
}

#[derive(Component)]
pub struct SoundEffect;

/// A sound that plays once at the sound effect volume.
pub fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (SoundEffect, AudioPlayer(handle), PlaybackSettings::DESPAWN)
}

/// Before the sound starts, so a muted category never clicks.
fn start_at_volume(
    add: On<Add<SoundEffect>>,
    volumes: Res<Volumes>,
    mut settings: Query<&mut PlaybackSettings>,
) {
    if let Ok(mut settings) = settings.get_mut(add.entity) {
        settings.volume = Volume::Linear(volumes.sfx);
    }
}

fn tap(_: On<Activate>, mut commands: Commands, audio: Res<AudioAssets>) {
    commands.spawn(sound_effect(audio.tap.clone()));
}

/// A game event with a sound.
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sfx {
    Claim,
    /// A kill: an eaten or bombed enemy.
    Eat,
    Bomb,
    Bounce,
    /// A Shooter fires ([045]).
    Shot,
    /// A Splitter's hatch turns into soldiers ([044]).
    Hatch,
    /// The items of [046].
    Multiplier,
    TimeStop,
    AutoLine,
}

fn play_sfx(
    sfx: On<Sfx>,
    mut commands: Commands,
    audio: Res<AudioAssets>,
    time: Res<Time<Real>>,
    mut last: Local<HashMap<Sfx, f32>>,
) {
    let handle = match *sfx {
        Sfx::Claim => &audio.claim,
        Sfx::Eat => &audio.eat,
        Sfx::Bomb => &audio.bomb,
        Sfx::Bounce => &audio.bounce,
        Sfx::Shot => &audio.shot,
        Sfx::Hatch => &audio.hatch,
        Sfx::Multiplier => &audio.multiplier,
        Sfx::TimeStop => &audio.time_stop,
        Sfx::AutoLine => &audio.auto_line,
    };
    if matches!(*sfx, Sfx::Bounce | Sfx::Shot) {
        let now = time.elapsed_secs();
        if last.get(&*sfx).is_some_and(|t| now - t < REPEAT_GAP_SECS) {
            return;
        }
        last.insert(*sfx, now);
    }
    commands.spawn(sound_effect(handle.clone()));
}

/// The hum while a line is drawn.
#[derive(Component)]
struct LineHum;

/// A line starts with a zap and hums until it ends: a claim, a cancel or a hit.
fn hum_while_drawing(
    mut commands: Commands,
    audio: Res<AudioAssets>,
    paused: Option<Res<State<IsPaused>>>,
    line: Query<(), With<TempLineSegment>>,
    hum: Query<Entity, With<LineHum>>,
) {
    let running = paused.is_some_and(|p| *p.get() == IsPaused::Running);
    let drawing = running && !line.is_empty();
    match (drawing, hum.single()) {
        (true, Err(_)) => {
            commands.spawn(sound_effect(audio.line_start.clone()));
            commands.spawn((
                LineHum,
                SoundEffect,
                AudioPlayer(audio.line_hum.clone()),
                PlaybackSettings::LOOP,
                DespawnOnExit(Screen::Gameplay),
            ));
        }
        (false, Ok(hum)) => commands.entity(hum).despawn(),
        _ => {}
    }
}

fn switch_music(
    mut commands: Commands,
    track: Res<MusicTrack>,
    mut playing: Query<(&AudioPlayer, &mut Music)>,
) {
    let mut found = false;
    for (player, mut music) in &mut playing {
        let current = track.0.as_ref() == Some(&player.0);
        music.target = if current { 1. } else { 0. };
        found |= current;
    }
    if let (false, Some(handle)) = (found, &track.0) {
        commands.spawn((
            Music {
                gain: 0.,
                target: 1.,
            },
            AudioPlayer(handle.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::SILENT),
        ));
    }
}

/// Real time: the music goes on while the game is paused or the player dies.
fn fade_music(
    mut commands: Commands,
    time: Res<Time<Real>>,
    volumes: Res<Volumes>,
    mut tracks: Query<(Entity, &mut Music, Option<&mut AudioSink>)>,
) {
    let step = time.delta_secs() / CROSSFADE_SECS;
    for (entity, mut music, sink) in &mut tracks {
        music.gain = if music.gain < music.target {
            (music.gain + step).min(music.target)
        } else {
            (music.gain - step).max(music.target)
        };
        if music.gain == 0. && music.target == 0. {
            commands.entity(entity).despawn();
        } else if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(volumes.music * music.gain));
        }
    }
}

fn pause_on_suspend(mut lifecycle: MessageReader<AppLifecycle>, sinks: Query<&AudioSink>) {
    for event in lifecycle.read() {
        match event {
            AppLifecycle::Suspended => sinks.iter().for_each(|sink| sink.pause()),
            AppLifecycle::Running => sinks.iter().for_each(|sink| sink.play()),
            _ => {}
        }
    }
}

/// Each sound that starts, for checks without speakers (emulator, simulator, shot runs).
fn log_started(started: Query<&AudioPlayer, Added<AudioSink>>) {
    for player in &started {
        info!(
            "audio: {} starts",
            player.0.path().map_or("?".into(), |p| p.to_string())
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use bevy::time::TimeUpdateStrategy;

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<AudioSource>()
            .add_message::<AppLifecycle>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1. / 60.,
            )))
            .init_resource::<Volumes>()
            .init_resource::<MusicTrack>()
            .add_observer(start_at_volume)
            .add_systems(
                Update,
                (
                    switch_music.run_if(resource_changed::<MusicTrack>),
                    fade_music,
                )
                    .chain(),
            );
        app
    }

    fn track(app: &mut App) -> Handle<AudioSource> {
        app.world_mut()
            .resource_mut::<Assets<AudioSource>>()
            .add(AudioSource {
                bytes: Arc::from([]),
            })
    }

    /// Each playing track's handle and gain.
    fn playing(app: &mut App) -> Vec<(Handle<AudioSource>, f32)> {
        app.world_mut()
            .query::<(&AudioPlayer, &Music)>()
            .iter(app.world())
            .map(|(player, music)| (player.0.clone(), music.gain))
            .collect()
    }

    #[test]
    fn a_new_track_crossfades_from_the_old_one() {
        let mut app = app();
        let (menu, game) = (track(&mut app), track(&mut app));
        app.world_mut().resource_mut::<MusicTrack>().0 = Some(menu.clone());
        for _ in 0..70 {
            app.update();
        }
        assert_eq!(playing(&mut app), [(menu.clone(), 1.)]);

        app.world_mut().resource_mut::<MusicTrack>().0 = Some(game.clone());
        for _ in 0..30 {
            app.update();
        }
        let both = playing(&mut app);
        assert_eq!(both.len(), 2, "both play halfway through");
        assert!(both.iter().all(|(_, gain)| (0.3..0.7).contains(gain)));
        for _ in 0..40 {
            app.update();
        }
        assert_eq!(playing(&mut app), [(game, 1.)], "the old track is gone");
    }

    #[test]
    fn the_same_track_keeps_playing() {
        let mut app = app();
        let menu = track(&mut app);
        app.world_mut().resource_mut::<MusicTrack>().0 = Some(menu.clone());
        for _ in 0..70 {
            app.update();
        }
        app.world_mut().resource_mut::<MusicTrack>().0 = Some(menu.clone());
        app.update();
        assert_eq!(playing(&mut app), [(menu, 1.)]);
    }

    #[test]
    fn a_sound_effect_starts_at_its_category_volume() {
        let mut app = app();
        app.world_mut().resource_mut::<Volumes>().sfx = 0.25;
        let sound = track(&mut app);
        let effect = app.world_mut().spawn(sound_effect(sound)).id();
        let settings = app.world().get::<PlaybackSettings>(effect).unwrap();
        assert_eq!(settings.volume, Volume::Linear(0.25));
    }

    #[test]
    fn many_bounces_or_shots_at_once_play_one_sound_each() {
        let mut app = app();
        app.init_resource::<AudioAssets>().add_observer(play_sfx);
        for sfx in [
            Sfx::Shot,
            Sfx::Shot,
            Sfx::Bounce,
            Sfx::Bounce,
            Sfx::Hatch,
            Sfx::Hatch,
        ] {
            app.world_mut().trigger(sfx);
        }
        app.world_mut().flush();
        let sounds = app
            .world_mut()
            .query::<&AudioPlayer>()
            .iter(app.world())
            .count();
        assert_eq!(sounds, 4);
    }
}
