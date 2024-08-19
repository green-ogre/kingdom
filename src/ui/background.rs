use std::time::Duration;

use super::UiNode;
use crate::{pixel_perfect::PIXEL_PERFECT_LAYER, GameState};
use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use rand::Rng;

pub const CROWD_VOLUME: f32 = 0.025;
pub const CRICKET_VOLUME: f32 = 0.25;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Main), setup_background)
            .add_systems(FixedPreUpdate, (animate_clouds, animate_crowd));
    }
}

fn setup_background(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        SpriteBundle {
            texture: server.load("clouds.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::new(-256., 10., -100.)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(256. * 4., 176.)),
                ..Default::default()
            },
            ..Default::default()
        },
        BackgroundClouds,
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.,
        },
        PIXEL_PERFECT_LAYER,
        UiNode,
        AudioBundle {
            source: server.load("audio/wind.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(1.),
                ..Default::default()
            },
        },
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("town.png"),
            transform: Transform::from_translation(Vec3::default().with_z(-50.)),
            ..Default::default()
        },
        BackgroundTown,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/night_background.png"),
            transform: Transform::from_translation(Vec3::default().with_z(-49.).with_y(-1.)),
            visibility: Visibility::Hidden,
            ..Default::default()
        },
        BackgroundTownNight,
        PIXEL_PERFECT_LAYER,
    ));

    let layout = TextureAtlasLayout::from_grid(UVec2::new(300, 135), 2, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        SpriteBundle {
            texture: server.load("crowd_layer_3.png"),
            transform: Transform::from_translation(Vec3::new(0., 0., -40.)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        Crowd::Three(Timer::from_seconds(0.3, TimerMode::Repeating)),
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("crowd_layer_2.png"),
            transform: Transform::from_translation(Vec3::new(0., 0., -30.)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        Crowd::Two(Timer::from_seconds(0.3, TimerMode::Repeating)),
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("crowd_layer_1.png"),
            transform: Transform::from_translation(Vec3::new(0., -1., -20.)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        Crowd::One(Timer::from_seconds(0.3, TimerMode::Repeating)),
        UiNode,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        AudioBundle {
            source: server.load("audio/crowd.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(CROWD_VOLUME),
                ..Default::default()
            },
        },
        CrowdAudio,
    ));

    commands.spawn((
        AudioBundle {
            source: server.load("audio/cricket-chirp-56209.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.),
                ..Default::default()
            },
        },
        CricketAudio,
    ));
}

#[derive(Component)]
struct BackgroundClouds;

#[derive(Component)]
struct BackgroundTown;

#[derive(Component)]
pub struct BackgroundTownNight;

fn animate_clouds(mut clouds: Query<&mut Transform, With<BackgroundClouds>>, time: Res<Time>) {
    const SPEED: f32 = 0.5;

    if let Ok(mut clouds) = clouds.get_single_mut() {
        if clouds.translation.x >= 256. {
            clouds.translation.x = -256.;
        }
        clouds.translation.x += time.delta_seconds() * SPEED;
    }
}

#[derive(Component)]
pub enum Crowd {
    One(Timer),
    Two(Timer),
    Three(Timer),
}

#[derive(Component)]
pub struct CrowdAudio;

#[derive(Component)]
pub struct CricketAudio;

fn animate_crowd(mut crowds: Query<(&mut Crowd, &mut TextureAtlas)>, time: Res<Time>) {
    for (crowd, mut atlas) in crowds.iter_mut() {
        let duration = rand::thread_rng().gen_range(1.2..1.5);
        let timer = match crowd.into_inner() {
            Crowd::One(timer) => timer,
            Crowd::Two(timer) => timer,
            Crowd::Three(timer) => timer,
        };

        timer.tick(time.delta());

        if timer.finished() {
            timer.set_duration(Duration::from_secs_f32(duration));
            atlas.index += 1;
            if atlas.index >= 2 {
                atlas.index = 0;
            }
        }
    }
}
