use crate::animated_sprites::{AnimationIndices, AnimationTimer};
use crate::character::CharacterUi;
use crate::pixel_perfect::PIXEL_PERFECT_LAYER;
use crate::state::{KingdomState, NewHeartSize};
use crate::type_writer::TypeWriter;
use bevy::audio::PlaybackMode;
use bevy::render::view::RenderLayers;
use bevy::{audio::Volume, prelude::*};
use bevy_tweening::*;
use lens::{TransformRotateZLens, TransformScaleLens};
use rand::Rng;
use sickle_ui::{prelude::*, SickleUiPlugin};
use std::time::Duration;

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TweeningPlugin, SickleUiPlugin))
            .add_systems(
                OnEnter(GameState::Main),
                (
                    setup,
                    setup_ui,
                    setup_heart_ui,
                    setup_courtroom,
                    setup_background,
                ),
            )
            .add_systems(
                Update,
                (heart_ui, mask_ui).run_if(in_state(GameState::Main)),
            )
            .add_systems(
                PreUpdate,
                should_show_selection_ui.run_if(in_state(GameState::Main)),
            )
            .add_systems(FixedPreUpdate, (animate_clouds, animate_crowd))
            .add_systems(Update, selection_ui.run_if(in_state(GameState::Main)))
            .add_systems(OnEnter(GameState::Win), win_ui)
            .add_systems(OnEnter(GameState::Loose), loose_ui)
            .register_type::<Decision>();
    }
}

#[derive(Component)]
struct HeartUi;

pub const HEART_SCALE: f32 = 16.;
pub const FONT_PATH: &'static str = "ui/alagard.ttf";

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column.spawn((
                TextBundle::from_section(
                    "Heart: {}",
                    TextStyle {
                        font: server.load(FONT_PATH),
                        font_size: 30.0,
                        ..default()
                    },
                ),
                HeartUi,
            ));
            column.spawn((
                TextBundle::from_section(
                    "Character: {}",
                    TextStyle {
                        font: server.load(FONT_PATH),
                        font_size: 30.0,
                        ..default()
                    },
                ),
                CharacterUi::Name,
            ));
        })
        .style()
        .justify_content(JustifyContent::End);

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column
                .row(|row| {
                    row.spawn((ButtonBundle::default(), Decision::Yes))
                        .entity_commands()
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "I concur.",
                                TextStyle {
                                    font: server.load(FONT_PATH),
                                    font_size: 30.0,
                                    ..default()
                                },
                            ));
                        });

                    row.spawn((ButtonBundle::default(), Decision::No))
                        .entity_commands()
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "I do not concur.",
                                TextStyle {
                                    font: server.load(FONT_PATH),
                                    font_size: 30.0,
                                    ..default()
                                },
                            ));
                        });
                })
                .insert((DecisionUi, Visibility::Visible));
        })
        .style()
        .justify_content(JustifyContent::Center);

    commands
        .ui_builder(UiRoot)
        .column(|column| {})
        .style()
        .justify_content(JustifyContent::Start);

    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font: server.load(FONT_PATH),
                font_size: 30.0,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(550.),
            top: Val::Px(900.),
            max_width: Val::Px(750.),
            ..Default::default()
        }),
        CharacterUi::Request,
    ));

    // commands
    //     .ui_builder(UiRoot)
    //     .column(|column| {
    //         column.spawn((
    //             // TextBundle::from_section(
    //             //     "",
    //             //     TextStyle {
    //             //         font: server.load(FONT_PATH),
    //             //         font_size: 30.0,
    //             //         ..default()
    //             //     },
    //             // ),
    //             // CharacterUi::Request,
    //             SpriteBundle {
    //                 texture: server.load("ui/ui.png"),
    //                 transform: Transform::from_scale(Vec3::splat(8.))
    //                     .with_translation(Vec3::default().with_y(-540.)),
    //                 ..Default::default()
    //             },
    //         ));
    //     })
    //     .style()
    //     .justify_content(JustifyContent::Start);
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
            transform: Transform::from_translation(Vec3::new(0., 0., -20.)),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 0,
        },
        Crowd::One(Timer::from_seconds(0.3, TimerMode::Repeating)),
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        AudioBundle {
            source: server.load("audio/crowd.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.025),
                ..Default::default()
            },
        },
        CrowdAudio,
    ));
}

#[derive(Component)]
struct BackgroundClouds;

#[derive(Component)]
struct BackgroundTown;

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
enum Crowd {
    One(Timer),
    Two(Timer),
    Three(Timer),
}

#[derive(Component)]
struct CrowdAudio;

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

fn setup_ui(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/ui.png"),
            transform: Transform::from_xyz(0., 0., 10.),
            // .with_scale(Vec3::splat(HEART_SCALE * (50. / 130.))),
            ..Default::default()
        },
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/happy_mask.png"),
            transform: Transform::from_xyz(0., 0., 20.),
            ..Default::default()
        },
        Mask::Happy,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/neutral_mask.png"),
            transform: Transform::from_xyz(0., 0., 20.),
            ..Default::default()
        },
        Mask::Neutral,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/sad_mask.png"),
            transform: Transform::from_xyz(0., 0., 20.),
            ..Default::default()
        },
        Mask::Sad,
        PIXEL_PERFECT_LAYER,
    ));

    commands.insert_resource(ActiveMask(Mask::Happy));
}

#[derive(Component, PartialEq, Eq)]
pub enum Mask {
    Happy,
    Neutral,
    Sad,
}

#[derive(Resource)]
pub struct ActiveMask(pub Mask);

fn mask_ui(
    active_mask: Option<Res<ActiveMask>>,
    mut sprites: Query<(&mut Visibility, &Mask), With<Sprite>>,
) {
    if let Some(active_mask) = active_mask {
        for (mut vis, mask) in sprites.iter_mut() {
            if *mask == active_mask.0 {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    } else {
        for (mut vis, _mask) in sprites.iter_mut() {
            *vis = Visibility::Hidden;
        }
    }
}

fn setup_heart_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = server.load("ui/heart_sprite_sheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 5 };
    let transform = Transform::from_xyz(-90., -45., 100.);

    let pulse = Tween::new(
        // Use a quadratic easing on both endpoints.
        EaseFunction::QuadraticInOut,
        // Animation time (one way only; for ping-pong it takes 2 seconds
        // to come back to start).
        Duration::from_secs_f32(1.0),
        // The lens gives the Animator access to the Transform component,
        // to animate it. It also contains the start and end values associated
        // with the animation ratios 0. and 1.
        TransformScaleLens {
            start: transform.scale,
            end: transform.scale * Vec3::new(1.1, 1.05, 1.),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

    commands.spawn((
        SpriteBundle {
            texture,
            transform,
            // .with_scale(Vec3::splat(HEART_SCALE * (50. / 130.))),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_indices.first,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        HeartUi,
        Animator::new(pulse),
        PIXEL_PERFECT_LAYER,
    ));
}

fn heart_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    state: Res<KingdomState>,
    mut reader: EventReader<NewHeartSize>,
    mut heart: Query<(Entity, &mut Transform), (With<Sprite>, With<HeartUi>)>,
    mut heart_ui: Query<&mut Text, With<HeartUi>>,
) {
    if let Ok(mut text) = heart_ui.get_single_mut() {
        text.sections[0].value = format!("Heart size: {:?}", state.heart_size);
    }

    if let Ok((entity, mut transform)) = heart.get_single_mut() {
        let Some(new_size) = reader.read().next() else {
            return;
        };

        // transform.scale = Vec3::splat(HEART_SCALE * (new_size.0 / 130.));

        commands.spawn(AudioBundle {
            source: server.load("audio/heartbeat.wav"),
            settings: PlaybackSettings::default(),
        });

        let pulse = Tween::new(
            // Use a quadratic easing on both endpoints.
            EaseFunction::QuadraticInOut,
            // Animation time (one way only; for ping-pong it takes 2 seconds
            // to come back to start).
            Duration::from_secs_f32(1.0),
            // The lens gives the Animator access to the Transform component,
            // to animate it. It also contains the start and end values associated
            // with the animation ratios 0. and 1.
            TransformScaleLens {
                start: transform.scale,
                end: transform.scale * Vec3::new(1.1, 1.05, 1.),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        let rotate = Tween::new(
            // Use a quadratic easing on both endpoints.
            EaseFunction::QuadraticInOut,
            // Animation time (one way only; for ping-pong it takes 2 seconds
            // to come back to start).
            Duration::from_secs_f32(0.1),
            // The lens gives the Animator access to the Transform component,
            // to animate it. It also contains the start and end values associated
            // with the animation ratios 0. and 1.
            TransformRotateZLens {
                start: 0.,
                end: 0.05,
            },
        )
        .with_repeat_count(RepeatCount::Finite(4))
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands
            .entity(entity)
            .insert(Animator::new(Tracks::new([pulse, rotate])));
    }
}

fn setup_courtroom(mut commands: Commands, server: Res<AssetServer>) {
    // commands.spawn((
    //     SpriteBundle {
    //         texture: server.load("court_room/simplified/Level_0/_composite.png"),
    //         transform: Transform::default().with_scale(Vec3::splat(8.)),
    //         ..Default::default()
    //     },
    //     RenderLayers::layer(1),
    // ));

    // commands.spawn((
    //     Camera2dBundle {
    //         camera: Camera {
    //             hdr: true,
    //             order: -1,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     },
    //     RenderLayers::layer(1),
    //     CourtRoomCamera,
    // ));
}

// #[derive(Component)]
// struct CourtRoomSprite;
//
// fn update_courtroom(
//     windows: Query<&Window>,
//     court_room: Query<&mut Transform, With<CourtRoomSprite>>,
// ) {
//     let window = windows.single();
//
//     const PARALLAX_FACTOR: f32 = 0.05;
//
//     if let Some(world_position) = window.cursor_position() {
//         transform.translation.x = (world_position.x - 960.) * PARALLAX_FACTOR;
//         transform.translation.y = (world_position.y - 540.) * PARALLAX_FACTOR;
//     }
// }

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Decision {
    Yes,
    No,
}

#[derive(Component)]
struct DecisionUi;

#[derive(Resource)]
pub struct ShowSelectionUi;

fn should_show_selection_ui(
    mut commands: Commands,
    type_writer: Res<TypeWriter>,
    show_selection: Option<Res<ShowSelectionUi>>,
) {
    if type_writer.is_finished && show_selection.is_none() {
        commands.insert_resource(ShowSelectionUi);
        info!("character finished dialogue, displaying selection ui");
    } else if show_selection.is_some() && !type_writer.is_finished {
        commands.remove_resource::<ShowSelectionUi>();
    }
}

fn selection_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Decision),
        (With<Button>, Changed<Interaction>),
    >,
    // mut text_query: Query<&mut Text>,
    mut writer: EventWriter<Decision>,
    show: Option<Res<ShowSelectionUi>>,
    mut root_ui: Query<&mut Visibility, With<DecisionUi>>,
) {
    if show.is_some() {
        let mut vis = root_ui.single_mut();
        *vis = Visibility::Visible;

        for (interaction, mut color, decision) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    // *color = PRESSED_BUTTON.into();
                    // text.sections[0].value = "Press".to_string();
                    *color = NORMAL_BUTTON.into();
                    // border_color.0 = RED.into();

                    let decision_variation = if *decision == Decision::No { -0.25 } else { 0. };
                    commands.spawn(AudioBundle {
                        source: server.load(
                            "audio/retro/GameSFX/Weapon/reload/Retro Weapon Reload Best A 03.wav",
                        ),
                        settings: PlaybackSettings::default()
                            .with_volume(Volume::new(0.5))
                            .with_speed(1.8 - decision_variation),
                    });

                    writer.send(*decision);
                }
                Interaction::Hovered => {
                    // text.sections[0].value = "Hover".to_string();
                    *color = HOVERED_BUTTON.into();
                    // border_color.0 = Color::WHITE;
                }
                Interaction::None => {
                    // text.sections[0].value = "I concur.".to_string();
                    *color = NORMAL_BUTTON.into();
                    // border_color.0 = Color::BLACK;
                }
            }
        }
    } else {
        let mut vis = root_ui.single_mut();
        *vis = Visibility::Hidden;
    }
}

fn loose_ui(mut commands: Commands) {
    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column.spawn((TextBundle::from_section(
                "You loose",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            ),));
        })
        .style()
        .justify_content(JustifyContent::Start);
}

fn win_ui(mut commands: Commands) {
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
