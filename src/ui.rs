use crate::animated_sprites::{AnimationIndices, AnimationTimer};
use crate::character::CharacterUi;
use crate::state::{KingdomState, NewHeartSize};
use crate::type_writer::TypeWriter;
use bevy::{audio::Volume, prelude::*};
use bevy_tweening::*;
use lens::{TransformRotateZLens, TransformScaleLens};
use sickle_ui::{prelude::*, SickleUiPlugin};
use std::time::Duration;

use crate::GameState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TweeningPlugin, SickleUiPlugin))
            .add_systems(OnEnter(GameState::Main), (setup, setup_heart_ui))
            .add_systems(Update, heart_ui.run_if(in_state(GameState::Main)))
            .add_systems(
                PreUpdate,
                should_show_selection_ui.run_if(in_state(GameState::Main)),
            )
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

fn setup_heart_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = server.load("ui/heart_sprite_sheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 5 };

    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::from_xyz(-700., -250., 100.)
                .with_scale(Vec3::splat(HEART_SCALE * (50. / 130.))),
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_indices.first,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        HeartUi,
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

        transform.scale = Vec3::splat(HEART_SCALE * (new_size.0 / 130.));

        commands.spawn(AudioBundle {
            source: server.load("audio/heartbeat.wav"),
            settings: PlaybackSettings::default(),
        });

        let pulse = Tween::new(
            // Use a quadratic easing on both endpoints.
            EaseFunction::QuadraticInOut,
            // Animation time (one way only; for ping-pong it takes 2 seconds
            // to come back to start).
            Duration::from_secs_f32(0.1),
            // The lens gives the Animator access to the Transform component,
            // to animate it. It also contains the start and end values associated
            // with the animation ratios 0. and 1.
            TransformScaleLens {
                start: transform.scale,
                end: transform.scale * Vec3::new(1.1, 1.05, 1.),
            },
        )
        .with_repeat_count(RepeatCount::Finite(4))
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
