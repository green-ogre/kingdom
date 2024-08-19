use crate::{state::EndDay, GameState};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
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

#[derive(Resource, Reflect, Default)]
struct Music {
    playing: bool,
    position: f64,
}

#[derive(Debug, Event, PartialEq)]
pub enum MusicEvent {
    Play,
    Pause,
    FadeOutSecs(f32),
    FadeInSecs(f32),
}

fn start_music(mut event_writer: EventWriter<MusicEvent>) {
    event_writer.send(MusicEvent::Play);
}

fn test_music(
    mut key: EventReader<KeyboardInput>,
    mut event_writer: EventWriter<MusicEvent>,
    mut end_day: EventWriter<EndDay>,
) {
    for event in key.read() {
        if event.key_code == KeyCode::KeyJ {
            event_writer.send(MusicEvent::Play);
        }

        if event.key_code == KeyCode::KeyK {
            event_writer.send(MusicEvent::Pause);
        }

        if event.key_code == KeyCode::KeyL {
            event_writer.send(MusicEvent::FadeOutSecs(5.));
        }

        if event.key_code == KeyCode::Semicolon {
            event_writer.send(MusicEvent::FadeInSecs(5.));
        }

        if event.key_code == KeyCode::Quote {
            end_day.send(EndDay);
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
    let volume = 0.333;
    let loop_start: f64 = 6.15;
    let loop_end: f64 = 60. + 27.592;
    let path = "audio/court-day.wav";
    let start_tween = AudioTween::new(
        std::time::Duration::from_millis(100),
        AudioEasing::InPowi(2),
    );

    if !music.playing {
        let last = event_reader
            .read()
            .filter(|e| matches!(e, MusicEvent::FadeInSecs(_) | MusicEvent::Play))
            .last();

        match last {
            Some(MusicEvent::Play) => {
                music.playing = true;
                audio
                    .play(assets.load(path))
                    .with_volume(volume)
                    .start_from(music.position)
                    .fade_in(start_tween);
            }
            Some(MusicEvent::FadeInSecs(s)) => {
                music.playing = true;
                audio
                    .play(assets.load(path))
                    .with_volume(volume)
                    .start_from(music.position)
                    .fade_in(AudioTween::new(
                        std::time::Duration::from_secs_f32(*s),
                        AudioEasing::OutPowi(2),
                    ));
            }
            _ => {}
        }
    } else {
        music.position += time.delta_seconds_f64();

        let last_event = event_reader
            .read()
            .filter(|e| **e != MusicEvent::Play)
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

        if music.position >= loop_end {
            audio
                .play(assets.load(path))
                .with_volume(volume)
                .start_from(loop_start)
                .fade_in(start_tween);

            music.position = loop_start;
        }
    }
}
