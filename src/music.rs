use crate::{GameState, TimeState};
use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_kira_audio::prelude::*;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Music::default())
            .add_event::<MusicEvent>()
            .add_systems(Update, (handle_music_playback, test_music))
            .add_systems(OnEnter(GameState::Main), start_music);
    }
}

const MUSIC_VOL: f64 = 0.333;

pub fn play_final_stinger(commands: &mut Commands, assets: &AssetServer) {
    commands.spawn(AudioBundle {
        source: assets.load("audio/game-complete.wav"),
        settings: PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            ..Default::default()
        }
        .with_volume(bevy::audio::Volume::new(MUSIC_VOL as f32 * 0.75)),
    });
}

#[derive(Resource, Reflect, Default)]
struct Music {
    playing: bool,
    position: f64,
    kind: MusicKind,
}

#[derive(Debug, PartialEq, Reflect, Default)]
pub enum MusicKind {
    #[default]
    Day,
    Dream,
}

#[derive(Debug, Event, PartialEq)]
pub enum MusicEvent {
    Play(MusicKind),
    Pause,
    FadeOutSecs(f32),
    FadeInSecs(MusicKind, f32),
}

fn start_music(mut event_writer: EventWriter<MusicEvent>) {
    event_writer.send(MusicEvent::Play(MusicKind::Day));
}

fn test_music(
    mut key: EventReader<KeyboardInput>,
    mut event_writer: EventWriter<MusicEvent>,
    // mut end_day: EventWriter<EndDay>,
    mut time_state: ResMut<NextState<TimeState>>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    #[cfg(debug_assertions)]
    {
        for event in key.read() {
            if event.state == ButtonState::Released {
                continue;
            }

            if event.key_code == KeyCode::KeyJ {
                event_writer.send(MusicEvent::Play(MusicKind::Dream));
            }

            if event.key_code == KeyCode::KeyK {
                event_writer.send(MusicEvent::Pause);
            }

            if event.key_code == KeyCode::KeyL {
                event_writer.send(MusicEvent::FadeOutSecs(5.));
            }

            if event.key_code == KeyCode::Semicolon {
                event_writer.send(MusicEvent::Play(MusicKind::Day));
            }

            if event.key_code == KeyCode::Quote {
                time_state.set(TimeState::Evening);
            }

            if event.key_code == KeyCode::Comma {
                play_final_stinger(&mut commands, &server);
            }
        }
    }
}

fn handle_music_playback(
    audio: Res<Audio>,
    mut music: ResMut<Music>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut event_reader: EventReader<MusicEvent>,
) {
    let loop_start: f64 = 6.15;
    let loop_end: f64 = 60. + 27.592;
    let day_path = "audio/court-day.wav";
    let dream_path = "audio/court-dream.wav";
    let start_tween = AudioTween::new(
        std::time::Duration::from_millis(100),
        AudioEasing::InPowi(2),
    );

    if !music.playing {
        let last = event_reader
            .read()
            .filter(|e| matches!(e, MusicEvent::FadeInSecs(_, _) | MusicEvent::Play(_)))
            .last();

        match last {
            Some(MusicEvent::Play(MusicKind::Day)) => {
                music.playing = true;
                music.kind = MusicKind::Day;
                audio
                    .play(assets.load(day_path))
                    .with_volume(MUSIC_VOL)
                    .start_from(music.position)
                    .fade_in(AudioTween::new(
                        std::time::Duration::from_millis(50),
                        AudioEasing::OutPowi(2),
                    ));
            }
            Some(MusicEvent::FadeInSecs(MusicKind::Day, s)) => {
                music.playing = true;
                music.kind = MusicKind::Day;
                audio
                    .play(assets.load(day_path))
                    .with_volume(MUSIC_VOL)
                    .start_from(music.position)
                    .fade_in(AudioTween::new(
                        std::time::Duration::from_secs_f32(*s),
                        AudioEasing::OutPowi(2),
                    ));
            }
            Some(MusicEvent::Play(MusicKind::Dream)) => {
                music.playing = true;
                music.kind = MusicKind::Dream;
                audio
                    .play(assets.load(dream_path))
                    .with_volume(MUSIC_VOL)
                    .looped()
                    .fade_in(AudioTween::new(
                        std::time::Duration::from_millis(750),
                        AudioEasing::OutPowi(2),
                    ));
            }
            Some(MusicEvent::FadeInSecs(MusicKind::Dream, s)) => {
                music.playing = true;
                music.kind = MusicKind::Dream;
                audio
                    .play(assets.load(dream_path))
                    .with_volume(MUSIC_VOL)
                    .looped()
                    .fade_in(AudioTween::new(
                        std::time::Duration::from_secs_f32(*s),
                        AudioEasing::OutPowi(2),
                    ));
            }
            _ => {}
        }
    } else {
        if music.kind == MusicKind::Day {
            music.position += time.delta_seconds_f64();
        } else {
            music.position = 0.;
        }

        let last_event = event_reader
            .read()
            .filter(|e| !matches!(e, MusicEvent::Play(_) | MusicEvent::FadeInSecs(_, _)))
            .last();

        match last_event {
            Some(MusicEvent::Pause) => {
                music.playing = false;
                audio.stop().fade_out(AudioTween::new(
                    std::time::Duration::from_millis(50),
                    AudioEasing::OutPowi(2),
                ));
                return;
            }
            Some(MusicEvent::FadeOutSecs(s)) => {
                audio.stop().fade_out(AudioTween::new(
                    std::time::Duration::from_secs_f32(*s),
                    AudioEasing::OutPowi(2),
                ));
                music.playing = false;
                music.position = 0.;
                return;
            }
            _ => {}
        }

        if music.position >= loop_end && music.kind == MusicKind::Day {
            audio
                .play(assets.load(day_path))
                .with_volume(MUSIC_VOL)
                .start_from(loop_start)
                .fade_in(start_tween);

            music.position = loop_start;
        }
    }
}
