use super::UiNode;
use crate::{
    menu::ParallaxSprite,
    pixel_perfect::{HIGH_RES_LAYER, PIXEL_PERFECT_LAYER},
    state::KingdomState,
    time_state::TimeState,
    ui::hex_to_vec4,
    GameState,
};
use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use bevy_hanabi::prelude::*;
use bevy_hanabi::EffectAsset;
use rand::Rng;
use std::time::Duration;

pub const CROWD_VOLUME: f32 = 0.025;
pub const CRICKET_VOLUME: f32 = 0.25;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Main), setup_background)
            .add_systems(FixedPreUpdate, (animate_clouds, animate_crowd));
        // .add_systems(OnEnter(TimeState::Morning), setup_background_particles)
        // .add_systems(
        //     OnEnter(TimeState::Evening),
        //     setup_background_particles_for_dream,
        // );
    }
}

#[derive(Component)]
pub struct BackgroundParticles;

pub fn setup_background_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    prev_particles: Query<Entity, With<BackgroundParticles>>,
    state: Res<KingdomState>,
) {
    let mut module = Module::default();

    let mut gradient = Gradient::new();

    let color = match state.day {
        0 => hex_to_vec4(0xa8ca58),
        1 => hex_to_vec4(0xcf573c),
        3 => hex_to_vec4(0xebede9),
        _ => hex_to_vec4(0xa8ca58),
    };

    gradient.add_key(0.0, color);
    gradient.add_key(1.0, color.with_w(0.));
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(150.),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetVelocityTangentModifier {
        origin: module.lit(Vec3::new(100., 100., 0.)),
        axis: module.lit(Vec3::new(0., 0., 1.)),
        speed: module.lit(20.),
    };
    let init_size = SetSizeModifier {
        size: CpuValue::Uniform((Vec2::splat(1.), Vec2::splat(2.))),
    };
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(10.));
    let effect = EffectAsset::new(vec![100], Spawner::rate(5.0.into()), module)
        .with_simulation_space(SimulationSpace::Local)
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .render(init_size)
        .render(ColorOverLifetimeModifier { gradient });
    let effect_asset = effects.add(effect);

    info!("spawning background particles");
    commands.spawn((
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect_asset).with_z_layer_2d(Some(-20.)),
            ..Default::default()
        },
        ParallaxSprite(0.0045),
        BackgroundParticles,
    ));

    for entity in prev_particles.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn setup_background_particles_for_dream(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    prev_particles: Query<Entity, With<BackgroundParticles>>,
) {
    let mut module = Module::default();

    let mut gradient = Gradient::new();
    let color = hex_to_vec4(0xFF0000);
    gradient.add_key(0.0, color);
    gradient.add_key(1.0, color.with_w(0.));
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO.with_x(-100.)),
        radius: module.lit(80.),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetAttributeModifier {
        attribute: Attribute::VELOCITY,
        value: module.lit(Vec3::ZERO.with_x(100.).with_y(-10.)),
    };
    let init_size = SetSizeModifier {
        size: CpuValue::Uniform((Vec2::splat(1.), Vec2::splat(2.))),
    };
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(4.));
    let effect = EffectAsset::new(vec![500], Spawner::rate(10.0.into()), module)
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .render(init_size)
        .render(ColorOverLifetimeModifier { gradient });
    let effect_asset = effects.add(effect);

    info!("spawning background particles for dream");
    commands.spawn((
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect_asset).with_z_layer_2d(Some(-20.)),
            ..Default::default()
        },
        BackgroundParticles,
    ));

    for entity in prev_particles.iter() {
        commands.entity(entity).despawn();
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
        ParallaxSprite(0.001),
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
        ParallaxSprite(0.001),
        PIXEL_PERFECT_LAYER,
    ));

    let layout = TextureAtlasLayout::from_grid(UVec2::new(300, 160), 2, 1, None, None);
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
        ParallaxSprite(0.002),
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
        ParallaxSprite(0.003),
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
        ParallaxSprite(0.004),
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
