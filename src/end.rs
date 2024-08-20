use crate::animation::{
    set_world_to_black, AudioVolumeLens, DelayedSpawn, FadeFromBlack, FadeToBlack,
    FadeToBlackSprite,
};
use crate::character::{Character, CharacterSprite, SelectedCharacterSprite};
use crate::menu::ParallaxSprite;
use crate::music::MusicEvent;
use crate::pixel_perfect::HIGH_RES_LAYER;
use crate::state::{KingdomState, MAX_HEART_SIZE, MAX_PROSPERITY, MIN_PROSPERITY};
use crate::time_state::TimeState;
use crate::type_writer::TypeWriter;
use crate::ui::background::{
    BackgroundParticles, BackgroundTownNight, Crowd, CrowdAudio, CROWD_VOLUME,
};
use crate::ui::{hex_to_vec4, HeartUi, StatBar, UiNode, FONT_PATH, HEART_SCALE};
use crate::{GameState, SkipRemove};
use bevy::audio::PlaybackMode;
use bevy::audio::Volume;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_hanabi::prelude::*;
use bevy_hanabi::EffectAsset;
use bevy_tweening::*;
use lens::{TextColorLens, TransformPositionLens, TransformScaleLens};
use sickle_ui::prelude::*;
use sickle_ui::ui_commands::UpdateStatesExt;
use std::time::Duration;

pub struct EndPlugin;

impl Plugin for EndPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Win), enter_win)
            .add_systems(
                OnEnter(GameState::Loose),
                (
                    enter_death.run_if(should_die),
                    enter_not_enough_prosperity.run_if(should_not_die),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    handle_revolution.run_if(in_state(GameState::Revolution)),
                    handle_win.run_if(in_state(GameState::WinScreen)),
                ),
            );
    }
}

fn should_not_die(state: Res<KingdomState>) -> bool {
    !should_die(state)
}

fn should_die(state: Res<KingdomState>) -> bool {
    if state.heart_size <= 0. || state.heart_size >= MAX_HEART_SIZE {
        true
    } else if state.day == 2 {
        if state.prosperity() >= MIN_PROSPERITY {
            false
        } else {
            false
        }
    } else {
        unreachable!()
    }
}

fn enter_win(
    mut commands: Commands,
    stat_ui: Query<Entity, With<StatBar>>,
    mut music: EventWriter<MusicEvent>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
) {
    info!("you won");

    if let Ok(crowd_audio) = crowd_audio.get_single() {
        commands
            .entity(crowd_audio)
            .insert(Animator::new(Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(5.),
                AudioVolumeLens {
                    start: CROWD_VOLUME,
                    end: 0.0,
                },
            )));
    }

    let id = commands.register_one_shot_system(show_win);
    commands.insert_resource(FadeToBlack::new(0.5, 10, 0., id));

    music.send(MusicEvent::FadeOutSecs(5.));

    for entity in stat_ui.iter() {
        commands.entity(entity).despawn();
    }
}

fn enter_not_enough_prosperity(
    mut commands: Commands,
    stat_ui: Query<Entity, With<StatBar>>,
    mut music: EventWriter<MusicEvent>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
) {
    info!("you did not have enough prosperity");

    if let Ok(crowd_audio) = crowd_audio.get_single() {
        commands
            .entity(crowd_audio)
            .insert(Animator::new(Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(5.),
                AudioVolumeLens {
                    start: CROWD_VOLUME,
                    end: 0.0,
                },
            )));
    }

    let id = commands.register_one_shot_system(show_revolution);
    commands.insert_resource(FadeToBlack::new(0.5, 10, 0., id));

    music.send(MusicEvent::FadeOutSecs(5.));

    for entity in stat_ui.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn setup_background_particles_for_revolution(
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
        center: module.lit(Vec3::ZERO.with_y(-100.)),
        radius: module.lit(120.),
        dimension: ShapeDimension::Surface,
    };
    let init_vel = SetAttributeModifier {
        attribute: Attribute::VELOCITY,
        value: module.lit(Vec3::ZERO.with_x(5.).with_y(80.)),
    };
    let init_size = SetSizeModifier {
        size: CpuValue::Uniform((Vec2::splat(1.), Vec2::splat(2.))),
    };
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(4.));
    let effect = EffectAsset::new(vec![500], Spawner::rate(100.0.into()), module)
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .render(init_size)
        .render(ColorOverLifetimeModifier { gradient });
    let effect_asset = effects.add(effect);

    info!("spawning background particles for dream");
    commands.spawn((
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect_asset).with_z_layer_2d(Some(0.)),
            ..Default::default()
        },
        BackgroundParticles,
    ));

    for entity in prev_particles.iter() {
        commands.entity(entity).despawn();
    }
}

fn show_revolution(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut background: Query<&mut Visibility, With<BackgroundTownNight>>,
    mut crowds: Query<(Entity, &mut Transform), With<Crowd>>,
    ui: Query<Entity, With<UiNode>>,
    mut type_writer: ResMut<TypeWriter>,
) {
    info!("revolution!");

    let id = commands.register_one_shot_system(setup_background_particles_for_revolution);
    commands.run_system(id);

    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }

    for (entity, mut transform) in crowds.iter_mut() {
        transform.translation.y -= 50.;
        commands.entity(entity).remove::<ParallaxSprite>();
    }

    // *background.single_mut() = Visibility::Visible;
    commands.spawn(SpriteBundle {
        texture: server.load("ui/burning_village.png"),
        transform: Transform::from_xyz(0., -1., -49.),
        ..Default::default()
    });

    let id = commands.register_one_shot_system(|mut commands: Commands| {
        commands.next_state(GameState::Revolution);
    });
    commands.insert_resource(FadeFromBlack::new(0.5, 10, 0., id));

    commands.spawn((
        AudioBundle {
            source: server.load("audio/fire-sound-efftect-21991.mp3"),
            settings: PlaybackSettings::LOOP.with_volume(Volume::new(0.5)),
        },
        Revolution,
        Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: 0.,
                end: 0.4,
            },
        )),
    ));

    commands.spawn((
        AudioBundle {
            source: server.load("audio/angry-mob-loop-6847.mp3"),
            settings: PlaybackSettings::LOOP.with_volume(Volume::new(0.3)),
        },
        Revolution,
        Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: 0.,
                end: 0.2,
            },
        )),
    ));

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font: server.load(FONT_PATH),
                font_size: 50.,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Percent(10.),
            // right: Val::Percent(70.),
            top: Val::Percent(80.),
            // max_width: Val::Px(1000.),
            ..Default::default()
        })
        .with_text_justify(JustifyText::Left),
        RevolutionText,
        Revolution,
    ));

    commands.insert_resource(EnterMainMenuTimer(
        Timer::new(Duration::from_secs_f32(10.), TimerMode::Repeating),
        0,
        false,
    ));

    let sfx = server.load("audio/cursor_style_2_rev.wav");
    *type_writer = TypeWriter::new(
        "Pity... Your kingdom, sullied by your own wretched calamity\nnow revolts against you."
            .into(),
        0.035,
        sfx,
    );
}

#[derive(Resource)]
struct EnterMainMenuTimer(Timer, u32, bool);

#[derive(Component)]
struct Revolution;

#[derive(Component)]
struct RevolutionText;

fn handle_revolution(
    mut commands: Commands,
    mut intro_text: Query<&mut Text, With<RevolutionText>>,
    mut type_writer: ResMut<TypeWriter>,
    mut reader: EventReader<KeyboardInput>,
    time: Res<Time>,
    mut timer: ResMut<EnterMainMenuTimer>,
    enitites: Query<Entity, (Without<PrimaryWindow>, Without<SkipRemove>)>,
    server: Res<AssetServer>,
) {
    let mut enter_next_state = || {
        let id = commands.register_one_shot_system(reset_game);
        commands.run_system(id);
    };

    if !timer.2 {
        reader.clear();
        timer.2 = true;
    }

    for input in reader.read() {
        if matches!(
            input,
            KeyboardInput {
                state,
                ..
            } if *state
                == ButtonState::Pressed
        ) {
            if !type_writer.is_finished {
                type_writer.finish();
                continue;
            }

            timer.1 += 1;
            timer.0.reset();

            if timer.1 >= 2 {
                enter_next_state();
                return;
            }

            if timer.1 >= 1 {
                let sfx = server.load("audio/cursor_style_2_rev.wav");
                *type_writer =
                    TypeWriter::new("You were no better than the last...".into(), 0.05, sfx);
            }
        }
    }

    timer.0.tick(time.delta());

    if timer.0.finished() {
        timer.1 += 1;

        if timer.1 >= 2 {
            enter_next_state();
            return;
        }

        if timer.1 >= 1 {
            let sfx = server.load("audio/cursor_style_2_rev.wav");
            *type_writer = TypeWriter::new("You were no better than the last...".into(), 0.05, sfx);
        }
    }

    type_writer.increment(&time);
    type_writer.try_play_sound(&mut commands);

    let mut text = intro_text.single_mut();
    text.sections[0].value = type_writer.slice_with_line_wrap().into();
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
    ui_images: Query<&mut UiImage, With<UiNode>>,
    ui_text: Query<&mut Text, With<UiNode>>,
    sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
) {
    set_world_to_black(ui_images, ui_text, sprite);

    music.send(MusicEvent::Pause);

    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }

    // commands.spawn((
    //     AudioBundle {
    //         source: server.load("audio/interface/Wav/Error_tones/style1/error_style_1_003.wav"),
    //         settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
    //     },
    //     BreathingSfx,
    // ));

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
                start: Vec3::splat(HEART_SCALE / 2.),
                end: Vec3::splat(HEART_SCALE / 4.),
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
    });

    delay_spawn.spawn_after(5., move |commands| {
        let id = commands.register_one_shot_system(reset_game);
        commands.run_system(id);
    });
}

fn reset_game(
    mut prev_sel_sprite: Query<
        (Entity, &mut Transform, &CharacterSprite),
        With<SelectedCharacterSprite>,
    >,
    mut commands: Commands,
    entities: Query<Entity, (Without<PrimaryWindow>, Without<SkipRemove>)>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
) {
    for (entity, mut transform, info) in prev_sel_sprite.iter_mut() {
        commands
            .entity(entity)
            .remove::<SelectedCharacterSprite>()
            .remove::<ParallaxSprite>();

        if *info == CharacterSprite::Body {
            transform.translation.x = 300.;
        }
    }

    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
    sprite.single_mut().color.set_alpha(0.);

    commands.next_state(GameState::MainMenu);
    commands.next_state(TimeState::None);
}

fn show_win(
    mut commands: Commands,
    mut heart_sprite: Query<&mut Visibility, (With<HeartUi>, With<Sprite>)>,
    audio: Query<Entity, With<HeartAudio>>,
    stat_ui: Query<Entity, With<StatBar>>,
    mut music: EventWriter<MusicEvent>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
    ui: Query<Entity, With<UiNode>>,
    crowds: Query<Entity, With<Crowd>>,
    server: Res<AssetServer>,
) {
    for entity in ui.iter() {
        commands.entity(entity).despawn();
    }

    for entity in crowds.iter() {
        commands.entity(entity).despawn()
    }

    // commands.entity(audio.single()).despawn();
    // *heart_sprite.single_mut() = Visibility::Hidden;
    // commands
    //     .ui_builder(UiRoot)
    //     .column(|column| {
    //         column.spawn((TextBundle::from_section(
    //             "You win!",
    //             TextStyle {
    //                 font_size: 30.0,
    //                 ..default()
    //             },
    //         ),));
    //     })
    //     .style()
    //     .justify_content(JustifyContent::Start);

    info!("win!");

    let id = commands.register_one_shot_system(setup_win);
    commands.run_system(id);

    // for entity in ui.iter() {
    //     commands.entity(entity).despawn();
    // }

    let id = commands.register_one_shot_system(|mut commands: Commands| {
        commands.next_state(GameState::WinScreen);
    });
    commands.insert_resource(FadeFromBlack::new(0.5, 10, 0., id));

    // commands.spawn((
    //     TextBundle::from_section(
    //         "",
    //         TextStyle {
    //             font: server.load(FONT_PATH),
    //             font_size: 50.,
    //             ..default()
    //         },
    //     )
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         left: Val::Percent(10.),
    //         // right: Val::Percent(70.),
    //         top: Val::Percent(80.),
    //         // max_width: Val::Px(1000.),
    //         ..Default::default()
    //     })
    //     .with_text_justify(JustifyText::Left),
    //     RevolutionText,
    //     Revolution,
    // ));
}

#[derive(Component)]
struct Win;

fn setup_win(mut commands: Commands, server: Res<AssetServer>, mut spawner: ResMut<DelayedSpawn>) {
    let id = commands.register_one_shot_system(setup_win_effect);
    commands.run_system(id);

    commands.spawn((
        AudioBundle {
            source: server.load("audio/birds-19624.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.5),
                ..Default::default()
            },
        },
        Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: 0.0,
                end: 0.5,
            },
        )),
        Win,
    ));

    let source = server.load("audio/game-complete.wav");
    let texture = server.load("ui/Popup Screen/Blurry_popup.png");
    spawner.spawn_after(5., move |commands| {
        commands.spawn((
            AudioBundle {
                source,
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(0.5),
                    ..Default::default()
                },
            },
            Win,
        ));
        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(0., 0., 300.),
                ..Default::default()
            },
            Animator::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs_f32(1.5),
                TransformPositionLens {
                    start: Vec3::default().with_x(300.),
                    end: Vec3::default(),
                },
            )),
        ));
    });

    let texture = server.load("ui/Skill Tree/Icons/Unlocked/x1/Unlocked2.png");
    spawner.spawn_after(6.5, move |commands| {
        commands.spawn((
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(300., 0., 0.),
                ..Default::default()
            },
            ProsperityIcon,
            Animator::new(Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs_f32(1.5),
                TransformPositionLens {
                    start: Vec3::default().with_x(300.),
                    end: Vec3::default(),
                },
            )),
        ));
    });

    let source =
        server.load("audio/interface/Wav/Confirm_tones/style5/confirm_style_5_echo_003.wav");
    let id = commands.register_one_shot_system(animate_prosperity_display);
    spawner.spawn_after(8., move |commands| {
        commands.run_system(id);
        commands.spawn((
            AudioBundle {
                source,
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(0.5),
                    ..Default::default()
                },
            },
            Win,
        ));
    });

    // commands.spawn((
    //     TextBundle::from_section(
    //         "",
    //         TextStyle {
    //             font: server.load(FONT_PATH),
    //             font_size: 49.,
    //             ..default()
    //         },
    //     )
    //     .with_text_justify(JustifyText::Left)
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         left: Val::Px(600.),
    //         top: Val::Px(200.),
    //         // max_width: Val::Px(1000.),
    //         ..Default::default()
    //     }),
    //     // IntroText,
    //     // Intro,
    // ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/1.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-22.)),
            ..Default::default()
        },
        // HIGH_RES_LAYER,
        Win,
    ));
    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/2.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-21.)),
            ..Default::default()
        },
        ParallaxSprite(0.001),
        // HIGH_RES_LAYER,
        Win,
    ));
    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/3.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-20.)),
            ..Default::default()
        },
        ParallaxSprite(0.005),
        // HIGH_RES_LAYER,
        Win,
    ));
}

#[derive(Component)]
struct ProsperityIcon;

fn animate_prosperity_display(
    mut commands: Commands,
    icon: Query<Entity, With<ProsperityIcon>>,
    state: Res<KingdomState>,
    server: Res<AssetServer>,
) {
    commands
        .entity(icon.single())
        .insert(Animator::new(Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(1.5),
            TransformPositionLens {
                start: Vec3::default(),
                end: Vec3::default().with_x(-20.),
            },
        )));

    commands
        .spawn(TextBundle::from_section(
            &format!("{}/{}", state.prosperity(), MAX_PROSPERITY),
            TextStyle {
                font: server.load(FONT_PATH),
                font_size: 80.,
                color: Srgba::new(1., 1., 1., 0.).into(),
            },
        ))
        .insert(Animator::new(
            Delay::new(Duration::from_secs_f32(1.5)).then(Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(1.5),
                TextColorLens {
                    start: Srgba::new(1., 1., 1., 0.).into(),
                    end: Srgba::new(1., 1., 1., 1.).into(),
                    section: 0,
                },
            )),
        ))
        .insert(Style {
            left: Val::Percent(46.),
            top: Val::Percent(45.5),
            ..Default::default()
        });
}

fn setup_win_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0., 0.8, 0.2, 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO.with_z(200.)),
        radius: module.lit(800.),
        dimension: ShapeDimension::Surface,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::new(-200., -200., 0.)),
        speed: module.lit(20.),
    };

    let init_size = SetSizeModifier {
        size: CpuValue::Uniform((Vec2::splat(4.), Vec2::splat(16.))),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(10.); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., 0., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    let effect = EffectAsset::new(
        // Maximum number of particles alive at a time
        vec![100],
        // Spawn at a rate of 5 particles per second
        Spawner::rate(5.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("MyEffect")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    .render(init_size)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(ColorOverLifetimeModifier { gradient });

    // Insert into the asset system
    let effect_asset = effects.add(effect);

    commands.spawn((
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect_asset).with_z_layer_2d(Some(100.)),
            transform: Transform::from_translation(Vec3::default().with_z(300.))
                .with_scale(Vec3::splat(1.)),
            ..Default::default()
        },
        Win,
    ));
}

#[derive(Resource)]
struct HaveClearedInput;

fn handle_win(
    mut commands: Commands,
    mut reader: EventReader<KeyboardInput>,
    enitites: Query<Entity, (Without<PrimaryWindow>, Without<SkipRemove>)>,
    have_cleared_input: Option<Res<HaveClearedInput>>,
) {
    if have_cleared_input.is_none() {
        reader.clear();
        commands.insert_resource(HaveClearedInput);
    }

    for input in reader.read() {
        if matches!(
            input,
            KeyboardInput {
                state,
                ..
            } if *state
                == ButtonState::Pressed
        ) {
            let id = commands.register_one_shot_system(reset_game);
            commands.run_system(id);

            return;
        }
    }
}
