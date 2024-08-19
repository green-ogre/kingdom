use bevy::prelude::*;
use rand::Rng;
use std::ops::Range;

#[derive(Debug, Event, Default)]
struct TypeWriterTimeout;

#[derive(Debug, Default, Resource)]
pub struct TypeWriter {
    pub is_finished: bool,
    pub timer: Timer,
    pub string: String,
    pub slice_range: Range<usize>,
    pub last_len: usize,
    pub sfx: Handle<AudioSource>,
    pub pitch_offset: f32,
}

impl TypeWriter {
    pub fn new(string: String, speed: f32, sfx: Handle<AudioSource>) -> Self {
        Self {
            timer: Timer::from_seconds(speed, TimerMode::Repeating),
            string: string.trim().into(),
            slice_range: 0..0,
            last_len: 0,
            is_finished: false,
            pitch_offset: 0.,
            sfx,
        }
    }

    pub fn increment(&mut self, time: &Time) {
        self.timer.tick(time.delta());

        if self.timer.just_finished() {
            self.last_len += 1;
            if self.last_len >= self.string.len() {
                self.is_finished = true;
                self.last_len = self.string.len();
            }
            self.slice_range = 0..self.last_len;
        }
    }

    pub fn try_play_sound(&self, commands: &mut Commands) {
        if !self.is_finished && self.timer.just_finished() {
            commands.spawn(AudioBundle {
                source: self.sfx.clone(),
                settings: PlaybackSettings {
                    speed: rand::thread_rng()
                        .gen_range((0.95 + self.pitch_offset)..(1.05 + self.pitch_offset)),
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..Default::default()
                },
            });
        }
    }

    pub fn finish(&mut self) {
        self.is_finished = true;
        self.last_len = self.string.len();
    }

    pub fn slice(&self) -> &str {
        &self.string[self.slice_range.clone()]
    }

    pub fn slice_with_line_wrap(&self) -> String {
        let mut slice = self.string[self.slice_range.clone()].to_owned();
        let padding = self
            .string
            .chars()
            .enumerate()
            .skip(slice.len())
            .find_map(|(i, c)| {
                if c == ' ' {
                    Some(i - slice.len())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| 0);
        if slice.chars().last() != Some(' ') {
            for _ in 0..padding {
                slice.push(' ');
            }
        }

        slice
    }
}
