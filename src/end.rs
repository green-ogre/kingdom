use crate::animation::{set_world_to_black, DelayedSpawn};
use crate::music::MusicEvent;
use crate::state::KingdomState;
use crate::ui::{HeartUi, StatBar, UiNode};
use crate::GameState;
use bevy::audio::PlaybackMode;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_tweening::*;
use lens::TransformScaleLens;
use sickle_ui::prelude::*;
use std::time::Duration;

pub struct EndPlugin;

impl Plugin for EndPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Win), (set_world_to_black, enter_death))
            .add_systems(
                OnEnter(GameState::Loose),
                (set_world_to_black, enter_death).chain(),
            );
    }
}

#[derive(Component)]
struct BreathingSfx;

pub fn enter_death(
    mut commands: Commands,
    mut heart_sprite: Query<
        (Entity, &mut Transform, &mut Visibility),
        (With<HeartUi>, With<Sprite>),
    >,
    state: Res<KingdomState>,
    audio: Query<Entity, With<Handle<AudioSource>>>,
    stat_ui: Query<Entity, With<StatBar>>,
    mut music: EventWriter<MusicEvent>,
    server: Res<AssetServer>,
    ui: Query<Entity, (With<UiNode>, Without<HeartUi>)>,
) {
    music.send(MusicEvent::Pause);

    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        AudioBundle {
            source: server.load("audio/heavy-breathing-14431.mp3"),
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
        },
        BreathingSfx,
    ));

    for entity in stat_ui.iter() {
        commands.entity(entity).despawn();
    }

    let Ok((entity, mut heart, mut visibility)) = heart_sprite.get_single_mut() else {
        error!("could not retrieve heart sprite for loose animation");
        return;
    };

    for sink in audio.iter() {
        commands.entity(sink).despawn();
    }

    heart.translation = Vec3::new(0., 0., 999.);
    *visibility = Visibility::Hidden;

    if state.heart_size > 4. {
        let grow = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(2.),
            TransformScaleLens {
                start: heart.scale,
                end: heart.scale * 2.,
            },
        );

        let show_heart_with_rapid_beating = commands.register_one_shot_system(show_heart_fast);
        let loose_ui = commands.register_one_shot_system(spawn_loose_ui);
        commands.entity(entity).insert(Animator::new(
            Delay::new(Duration::from_secs_f32(2.25))
                .with_completed_system(show_heart_with_rapid_beating)
                .then(grow.with_completed_system(loose_ui)),
        ));
    } else if state.heart_size < 1. {
        let shrink = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(4.),
            TransformScaleLens {
                start: heart.scale,
                end: heart.scale / 2.,
            },
        );

        let show_heart_with_rapid_beating = commands.register_one_shot_system(show_heart_slow);
        let loose_ui = commands.register_one_shot_system(spawn_loose_ui);
        commands.entity(entity).insert(Animator::new(
            Delay::new(Duration::from_secs_f32(2.))
                .with_completed_system(show_heart_with_rapid_beating)
                .then(shrink.with_completed_system(loose_ui)),
        ));
    } else {
        panic!("lost without meeting loose condition");
    }
}

#[derive(Component)]
struct HeartAudio;

fn show_heart_fast(
    mut heart_sprite: Query<&mut Visibility, (With<HeartUi>, With<Sprite>)>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    let mut vis = heart_sprite.single_mut();
    *vis = Visibility::Visible;

    commands.spawn((
        AudioBundle {
            source: server.load("audio/heartbeat.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                speed: 1.3,
                ..Default::default()
            },
        },
        HeartAudio,
    ));
}

fn show_heart_slow(
    mut heart_sprite: Query<&mut Visibility, (With<HeartUi>, With<Sprite>)>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    let mut vis = heart_sprite.single_mut();
    *vis = Visibility::Visible;

    commands.spawn((
        AudioBundle {
            source: server.load("audio/heartbeat.wav"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                speed: 0.7,
                ..Default::default()
            },
        },
        HeartAudio,
    ));
}

fn spawn_loose_ui(
    mut commands: Commands,
    mut heart_sprite: Query<&mut Visibility, (With<HeartUi>, With<Sprite>)>,
    audio: Query<Entity, With<HeartAudio>>,
    server: Res<AssetServer>,
    breathing: Query<Entity, With<BreathingSfx>>,
    mut delay_spawn: ResMut<DelayedSpawn>,
) {
    commands.entity(breathing.single()).despawn();
    commands.entity(audio.single()).despawn();

    *heart_sprite.single_mut() = Visibility::Hidden;
    commands.spawn(AudioBundle {
        source: server.load("audio/mixkit-glass-break-with-hammer-thud-759.wav"),
        settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
    });

    let source = server.load("audio/body-fall-47877.mp3");
    delay_spawn.spawn_after(1.5, move |commands| {
        commands.spawn(AudioBundle {
            source,
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
        });
    })
}

fn spawn_win_ui(
    mut commands: Commands,
    mut heart_sprite: Query<&mut Visibility, (With<HeartUi>, With<Sprite>)>,
    audio: Query<Entity, With<HeartAudio>>,
) {
    commands.entity(audio.single()).despawn();
    *heart_sprite.single_mut() = Visibility::Hidden;
    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column.spawn((TextBundle::from_section(
                "You win!",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),));
        })
        .style()
        .justify_content(JustifyContent::Start);
}
