//! Music and sound effects ([041]), after bevy_new_2d's audio module ([026]): two
//! categories, each with its own volume. The music crossfades when the track changes, and
//! all sound stops while the app is suspended ([023]).

use bevy::{audio::Volume, prelude::*, window::AppLifecycle};

use crate::{asset_tracking::LoadResource, screens::Screen, theme::widget::Activate};

const CROSSFADE_SECS: f32 = 1.;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Volumes>()
        .init_resource::<MusicTrack>()
        .load_resource::<AudioAssets>()
        .add_observer(start_at_volume)
        .add_observer(tap)
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
        );
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
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            menu: assets.load("audio/music/menu.ogg"),
            game: assets.load("audio/music/game.ogg"),
            tap: assets.load("audio/sfx/tap.ogg"),
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
}
