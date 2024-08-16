use bevy::prelude::*;
use std::ops::Range;

pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_sprite);
    }
}

#[derive(Debug, Bundle)]
pub struct AnimatedSpriteBundle {
    animation: SpriteAnimation,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
}

#[derive(Debug, Component)]
struct SpriteAnimation {
    speed: f32,
    accumulator: f32,
    range: Range<u32>,
    current_frame: i32,
    transform: Transform,
}

impl SpriteAnimation {
    pub fn new(range: Range<u32>, speed: f32, transform: Transform) -> Self {
        Self {
            current_frame: range.start as i32,
            speed: speed.max(0.),
            range,
            transform,
            accumulator: 0.,
        }
    }

    pub fn set_speed(&mut self, new_speed: f32) {
        self.speed = new_speed;
    }

    pub fn advance(&mut self, time: &Time) {
        self.accumulator += self.speed.abs() * time.delta_seconds();

        let frame_increment = if self.speed >= 0. { 1 } else { -1 };

        while self.accumulator >= 1. {
            self.accumulator -= 1.;

            self.current_frame += frame_increment;

            if self.current_frame >= self.range.end as i32 {
                self.current_frame = self.range.start as i32;
            } else if self.current_frame <= self.range.start as i32 {
                self.current_frame = self.range.end as i32 - 1;
            }
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut animated_sprites: Query<(&mut TextureAtlas, &mut SpriteAnimation)>,
) {
    for (mut atlas, mut animation) in animated_sprites.iter_mut() {
        atlas.index = animation.current_frame as usize;
        animation.advance(&time);
    }
}
