use crate::animated_sprites::{AnimationIndices, AnimationTimer};
use crate::character::{self, Character, CharacterUi, Characters, SelectedCharacter};
use crate::music::MusicEvent;
use crate::pixel_perfect::{HIGH_RES_LAYER, PIXEL_PERFECT_LAYER, RES_HEIGHT, RES_WIDTH};
use crate::state::{EndDay, KingdomState, NewHeartSize};
use crate::type_writer::TypeWriter;
use bevy::audio::PlaybackMode;
use bevy::ecs::system::SystemId;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::ui::ContentSize;
use bevy::window::PrimaryWindow;
use bevy::{audio::Volume, prelude::*};
use bevy_tweening::*;
use lens::{SpriteColorLens, TransformRotateZLens, TransformScaleLens};
use rand::Rng;
use sickle_ui::ui_commands::UpdateStatesExt;
use sickle_ui::{prelude::*, SickleUiPlugin};
use std::ops::Deref;
use std::time::Duration;

use crate::GameState;

const INSIGHT_CHARGE_TIME: f32 = 2.0;
const CROWD_VOLUME: f32 = 0.025;
const CRICKET_VOLUME: f32 = 0.25;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TweeningPlugin, SickleUiPlugin))
            .add_systems(
                OnEnter(GameState::Main),
                (
                    startup,
                    setup,
                    setup_ui,
                    setup_heart_ui,
                    setup_courtroom,
                    setup_background,
                    // set_world_to_black,
                ),
            )
            .add_systems(
                Update,
                (heart_ui, mask_ui, display_insight_tooltip, display_insight)
                    .run_if(in_state(GameState::Day)),
            )
            .add_systems(
                Update,
                component_animator_system::<AudioSink>.in_set(AnimationSystem::AnimationUpdate),
            )
            .add_systems(OnEnter(GameState::Night), enter_night)
            .add_systems(Update, (fade_to_black, fade_from_black))
            .add_systems(
                PostUpdate,
                (aquire_insight,).run_if(in_state(GameState::Day)),
            )
            .add_systems(
                PreUpdate,
                (should_show_selection_ui).run_if(in_state(GameState::Day)),
            )
            .add_systems(OnEnter(GameState::Morning), enter_morning)
            .add_systems(FixedPreUpdate, (animate_clouds, animate_crowd))
            .add_systems(Update, selection_ui.run_if(in_state(GameState::Day)))
            .add_systems(OnEnter(GameState::Win), (set_world_to_black, end_animation))
            .add_systems(
                OnEnter(GameState::Loose),
                (set_world_to_black, end_animation).chain(),
            )
            .add_event::<AquireInsight>()
            // .insert_resource(
            //     ColorScheme::<Mana>::new().foreground_color(ForegroundColor::Static(Color::BLUE)),
            // )
            .insert_resource(DayNumberUi::default())
            .insert_resource(Insight::default());
    }
}

struct AudioVolumeLens {
    start: f32,
    end: f32,
}

use bevy_tweening::Lens;

impl Lens<AudioSink> for AudioVolumeLens {
    fn lerp(&mut self, target: &mut dyn Targetable<AudioSink>, ratio: f32) {
        let volume = self.start + (self.end - self.start) * ratio;
        target.set_volume(volume);
    }
}

fn enter_night(
    mut commands: Commands,
    mut music: EventWriter<MusicEvent>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
    cricket_audio: Query<Entity, With<CricketAudio>>,
) {
    let system = commands.register_one_shot_system(show_night);
    commands.insert_resource(FadeToBlack::new(0.5, 10, 0., system));
    music.send(MusicEvent::FadeOutSecs(5.));
    commands
        .entity(crowd_audio.single())
        .insert(Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: CROWD_VOLUME,
                end: 0.,
            },
        )));
    commands
        .entity(cricket_audio.single())
        .insert(Animator::new(Delay::new(Duration::from_secs_f32(3.)).then(
            Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(5.),
                AudioVolumeLens {
                    start: 0.,
                    end: CRICKET_VOLUME,
                },
            ),
        )));
    info!("entering night");
}

fn show_night(
    mut commands: Commands,
    mut nigth_village_sprite: Query<&mut Visibility, With<BackgroundTownNight>>,
    mut crowds: Query<&mut Visibility, (With<Crowd>, Without<BackgroundTownNight>)>,
) {
    *nigth_village_sprite.single_mut() = Visibility::Visible;
    for mut vis in crowds.iter_mut() {
        *vis = Visibility::Hidden;
    }
    let system = commands.register_one_shot_system(handle_night);
    commands.insert_resource(FadeFromBlack::new(0.5, 10, 3., system));
}

fn handle_night(mut commands: Commands) {
    info!("entered night");
    commands.next_state(GameState::Morning);
    info!("exiting!")
}

#[derive(Resource, Default)]
struct DayNumberUi(Option<Timer>);

fn enter_morning(mut commands: Commands, server: Res<AssetServer>) {
    info!("enter morning");
    let system = commands.register_one_shot_system(handle_morning);
    commands.insert_resource(FadeToBlack::new(0.5, 4, 0., system));
    commands.spawn(AudioBundle {
        source: server.load("audio/church_bells.wav"),
        settings: PlaybackSettings::default().with_volume(Volume::new(0.5)),
    });
    // music.send(MusicEvent::FadeOutSecs(5.));
}

fn handle_morning(
    mut commands: Commands,
    mut next_day_ui: Query<(&mut Visibility, &mut Text), With<NextDayUi>>,
    mut day_number_ui: ResMut<DayNumberUi>,
    state: Res<KingdomState>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
    cricket_audio: Query<Entity, With<CricketAudio>>,
    mut nigth_village_sprite: Query<
        &mut Visibility,
        (With<BackgroundTownNight>, Without<NextDayUi>),
    >,
    mut crowds: Query<
        &mut Visibility,
        (
            With<Crowd>,
            Without<BackgroundTownNight>,
            Without<NextDayUi>,
        ),
    >,
) {
    info!("handle morning");

    commands
        .entity(cricket_audio.single())
        .insert(Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: CRICKET_VOLUME,
                end: 0.,
            },
        )));
    commands.entity(crowd_audio.single()).insert(Animator::new(
        Delay::new(Duration::from_secs_f32(3.)).then(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: 0.,
                end: CROWD_VOLUME,
            },
        )),
    ));

    *nigth_village_sprite.single_mut() = Visibility::Hidden;
    for mut vis in crowds.iter_mut() {
        *vis = Visibility::Visible;
    }

    let (mut vis, mut text) = next_day_ui.single_mut();
    *vis = Visibility::Visible;
    text.sections[0].value = state.day_name().to_string();

    let system = commands.register_one_shot_system(enter_day);
    commands.insert_resource(FadeFromBlack::new(0.5, 4, 3., system));
    day_number_ui.0 = None;

    let (mut vis, _) = next_day_ui.single_mut();
    *vis = Visibility::Hidden;
}

fn enter_day(
    mut commands: Commands,
    characters: Res<Characters>,
    mut music: EventWriter<MusicEvent>,
) {
    info!("enter day");
    commands.run_system(characters.choose_new_character);
    commands.next_state(GameState::Day);
    music.send(MusicEvent::Play);
}

#[derive(Resource)]
pub struct FadeToBlack {
    system_on_complete: SystemId,
    delay: f32,
    timer_per_step: Timer,
    total_steps: u32,
    steps: u32,
}

impl FadeToBlack {
    pub fn new(secs_per_step: f32, steps: u32, delay: f32, system_on_complete: SystemId) -> Self {
        Self {
            delay,
            system_on_complete,
            timer_per_step: Timer::from_seconds(secs_per_step, TimerMode::Repeating),
            total_steps: steps,
            steps,
        }
    }
}

pub fn set_world_to_black(
    mut ui_images: Query<&mut UiImage, With<UiNode>>,
    mut ui_text: Query<&mut Text, With<UiNode>>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
) {
    for mut image in ui_images.iter_mut() {
        image.color.set_alpha(0.);
    }

    for mut text in ui_text.iter_mut() {
        for section in text.sections.iter_mut() {
            let color = &mut section.style.color;
            color.set_alpha(0.);
        }
    }

    if let Ok(mut sprite) = sprite.get_single_mut() {
        sprite.color.set_alpha(1.);
    }
}

fn fade_to_black(
    mut commands: Commands,
    mut ui_images: Query<&mut UiImage, With<UiNode>>,
    mut ui_text: Query<&mut Text, With<UiNode>>,
    fade_to_black: Option<ResMut<FadeToBlack>>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
    time: Res<Time>,
) {
    if let Some(mut fade) = fade_to_black {
        fade.delay -= time.delta_seconds();
        if fade.delay > 0.0 {
            return;
        }

        fade.timer_per_step.tick(time.delta());
        if fade.timer_per_step.finished() {
            fade.steps = fade.steps.saturating_sub(1);

            for mut image in ui_images.iter_mut() {
                let a = image.color.alpha();
                image.color.set_alpha(a - 1.0 / fade.total_steps as f32);
            }

            for mut text in ui_text.iter_mut() {
                for section in text.sections.iter_mut() {
                    let color = &mut section.style.color;
                    let a = color.alpha();
                    color.set_alpha(a - 1.0 / fade.total_steps as f32);
                }
            }

            let mut sprite = sprite.single_mut();
            let alpha = fade.steps as f32 * (1.0 / fade.total_steps as f32);
            let alpha = 1. - alpha.powi(2);
            sprite.color.set_alpha(alpha);

            if fade.steps == 0 {
                commands.remove_resource::<FadeToBlack>();
                commands.run_system(fade.system_on_complete);
                return;
            }
        }
    }
}

#[derive(Resource)]
pub struct FadeFromBlack {
    system_on_complete: SystemId,
    delay: f32,
    timer_per_step: Timer,
    total_steps: u32,
    steps: u32,
}

impl FadeFromBlack {
    pub fn new(secs_per_step: f32, steps: u32, delay: f32, system_on_complete: SystemId) -> Self {
        Self {
            system_on_complete,
            delay,
            timer_per_step: Timer::from_seconds(secs_per_step, TimerMode::Repeating),
            total_steps: steps,
            steps,
        }
    }
}

fn fade_from_black(
    mut commands: Commands,
    mut ui_images: Query<&mut UiImage, With<UiNode>>,
    mut ui_text: Query<&mut Text, With<UiNode>>,
    fade_from_black: Option<ResMut<FadeFromBlack>>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
    time: Res<Time>,
) {
    if let Some(mut fade) = fade_from_black {
        fade.delay -= time.delta_seconds();
        if fade.delay > 0.0 {
            return;
        }

        fade.timer_per_step.tick(time.delta());
        if fade.timer_per_step.finished() {
            fade.steps = fade.steps.saturating_sub(1);

            for mut image in ui_images.iter_mut() {
                let a = image.color.alpha();
                image.color.set_alpha(a + 1.0 / fade.total_steps as f32);
            }

            for mut text in ui_text.iter_mut() {
                for section in text.sections.iter_mut() {
                    let color = &mut section.style.color;
                    let a = color.alpha();
                    color.set_alpha(a + 1.0 / fade.total_steps as f32);
                }
            }

            let mut sprite = sprite.single_mut();

            let alpha = fade.steps as f32 * (1.0 / fade.total_steps as f32);
            let alpha = 1. - (1. - alpha).powi(2);
            sprite.color.set_alpha(alpha);

            if fade.steps == 0 {
                commands.remove_resource::<FadeFromBlack>();
                commands.run_system(fade.system_on_complete);
                return;
            }
        }
    }
}

#[derive(Component)]
struct HeartUi;

pub const HEART_SCALE: f32 = 16.;
pub const FONT_PATH: &'static str = "ui/small_pixel-7.ttf";

#[derive(Component)]
pub struct UiNode;

#[derive(Component)]
pub struct FadeToBlackSprite;

#[derive(Component)]
struct NextDayUi;

fn startup(mut commands: Commands, mut state: ResMut<KingdomState>) {
    commands.next_state(GameState::Night);

    let id = commands.register_one_shot_system(spawn_insight);
    commands.insert_resource(SpawnInsight(id));
    // commands.run_system(id);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1000.0, 1000.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 900.0),
            ..default()
        },
        FadeToBlackSprite,
    ));
}

fn setup(mut commands: Commands, server: Res<AssetServer>, mut writer: EventWriter<EndDay>) {
    // writer.send(EndDay);

    commands
        .spawn((
            TextBundle::from_section(
                "Day 1",
                TextStyle {
                    font: server.load(FONT_PATH),
                    font_size: 128.0,
                    ..default()
                },
            )
            .with_text_justify(JustifyText::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(50.),
                left: Val::Percent(41.),
                ..default()
            }),
            NextDayUi,
        ))
        .insert(Visibility::Hidden);

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            // column.spawn((
            //     TextBundle::from_section(
            //         "Heart: {}",
            //         TextStyle {
            //             font: server.load(FONT_PATH),
            //             font_size: 30.0,
            //             ..default()
            //         },
            //     ),
            //     HeartUi,
            // ));
            // column.spawn((
            //     TextBundle::from_section(
            //         "Character: {}",
            //         TextStyle {
            //             font: server.load(FONT_PATH),
            //             font_size: 30.0,
            //             ..default()
            //         },
            //     ),
            //     CharacterUi::Name,
            // ));
        })
        .style()
        .justify_content(JustifyContent::End);

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column
                .row(|row| {
                    row.spawn((ButtonBundle::default(), DecisionType::Yes))
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

                    row.spawn((ButtonBundle::default(), DecisionType::No))
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
                font_size: 49.,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(472.),
            top: Val::Px(702.7),
            max_width: Val::Px(980.7),
            ..Default::default()
        }),
        Name::new("Character Text"),
        CharacterUi::Request,
        UiNode,
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
            transform: Transform::from_translation(Vec3::default().with_z(-49.)),
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
            transform: Transform::from_translation(Vec3::new(0., 0., -20.)),
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
struct BackgroundTownNight;

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

#[derive(Component)]
struct CricketAudio;

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

fn setup_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/ui.png"),
            transform: Transform::from_xyz(0., 0., 10.),
            // .with_scale(Vec3::splat(HEART_SCALE * (50. / 130.))),
            ..Default::default()
        },
        UiNode,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/happy_mask.png"),
            transform: Transform::from_xyz(0., 0., 20.),
            ..Default::default()
        },
        Mask::Happy,
        UiNode,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/neutral_mask.png"),
            transform: Transform::from_xyz(0., 0., 20.),
            ..Default::default()
        },
        Mask::Neutral,
        UiNode,
        PIXEL_PERFECT_LAYER,
    ));

    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/sad_mask.png"),
            transform: Transform::from_xyz(0., 0., 20.),
            ..Default::default()
        },
        Mask::Sad,
        UiNode,
        PIXEL_PERFECT_LAYER,
    ));
}

#[derive(Component)]
pub struct Cursor;

#[derive(Component)]
pub struct InsightToolTip;

#[derive(Component)]
pub struct CursorCanDecide;

#[derive(Resource)]
pub struct Insight {
    grace: Timer,
    is_held: bool,
    character: Option<Handle<Character>>,
}

#[derive(Component, Default, Reflect)]
pub struct InsightBar(f32);

#[derive(Component)]
pub struct InsightBarBorder;

impl Default for Insight {
    fn default() -> Self {
        Self {
            grace: Timer::from_seconds(INSIGHT_CHARGE_TIME, TimerMode::Repeating),
            is_held: false,
            character: None,
        }
    }
}

#[derive(Event)]
pub struct AquireInsight;

pub fn update_cursor(
    windows: Query<&Window>,
    mut cursor: Query<(Entity, &mut Style), (With<Cursor>, Without<InsightToolTip>)>,
    mut tool_tip: Query<
        (Entity, &mut Style, &mut Visibility),
        (With<InsightToolTip>, Without<Cursor>),
    >,
    mut reader: EventReader<MouseButtonInput>,
    mut writer: EventWriter<AquireInsight>,
    mut insight: ResMut<Insight>,
    mut commands: Commands,
    time: Res<Time>,
    mut selected_character: Query<(Entity, &SelectedCharacter)>,
    server: Res<AssetServer>,
    mut insight_bar: Query<(Entity, &mut Transform, &mut InsightBar)>,
    mut insight_bar_border: Query<Entity, With<InsightBarBorder>>,
) {
    let window = windows.single();
    let Ok((entity, mut style)) = cursor.get_single_mut() else {
        return;
    };
    let (tool_tip_entity, mut tool_tip_style, mut visibility) = tool_tip.single_mut();

    if let Some(world_position) = window.cursor_position() {
        let left = world_position.x - 125.;
        let top = world_position.y - 1080. + 40.;

        style.left = Val::Px(left);
        style.top = Val::Px(top);

        tool_tip_style.left = Val::Px(left);
        tool_tip_style.top = Val::Px(top);

        // println!("{top:?}");
        if top < -350. && !selected_character.is_empty() {
            commands.entity(entity).insert(CursorCanDecide);
            insight.grace.tick(time.delta());

            if let Ok((_, mut sprite_transform, mut bar)) = insight_bar.get_single_mut() {
                bar.0 = insight.grace.remaining().as_secs_f32()
                    / insight.grace.duration().as_secs_f32();
                sprite_transform.scale.x = bar.0 / 1.0;
            }

            for input in reader.read() {
                if input.button == MouseButton::Right {
                    match input.state {
                        ButtonState::Pressed => {
                            if insight.is_held == false
                                && insight.character.as_ref()
                                    != selected_character.iter().next().map(|s| &s.1 .0)
                            {
                                let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar x1.png";
                                commands.spawn((
                                    SpriteBundle {
                                        texture: server.load(bar_path),
                                        transform: Transform::from_xyz(0., 0., 310.),
                                        ..Default::default()
                                    },
                                    InsightBarBorder,
                                    HIGH_RES_LAYER,
                                ));
                                let bar_path =
                                    "ui/Boss bar/Mini Boss bar/mioni_boss_bar_filler x1.png";
                                commands.spawn((
                                    SpriteBundle {
                                        texture: server.load(bar_path),
                                        transform: Transform::from_xyz(0., 0., 300.)
                                            .with_scale(Vec3::new(0., 1., 1.)),
                                        ..Default::default()
                                    },
                                    InsightBar(0.),
                                    HIGH_RES_LAYER,
                                ));
                                let sfx_path = "audio/sci-fi-sound-effect-designed-circuits-sfx-tonal-15-202059.mp3";
                                commands.spawn(AudioBundle {
                                    source: server.load(sfx_path),
                                    settings: PlaybackSettings::default()
                                        .with_volume(Volume::new(0.4)),
                                });
                            }

                            insight.is_held = true;
                        }
                        ButtonState::Released => {
                            insight.is_held = false;
                            if let Ok((entity, _, _)) = insight_bar.get_single() {
                                commands.entity(entity).despawn();
                            }
                            if let Ok(entity) = insight_bar_border.get_single() {
                                commands.entity(entity).despawn();
                            }
                            insight.grace.reset();
                        }
                    }

                    insight.grace.reset();
                }
            }

            if insight.grace.finished() && insight.is_held {
                writer.send(AquireInsight);
                insight.is_held = false;
                if let Ok((entity, _, _)) = insight_bar.get_single() {
                    commands.entity(entity).despawn();
                }
                if let Ok(entity) = insight_bar_border.get_single() {
                    commands.entity(entity).despawn();
                }
                insight.grace.reset();
            }

            *visibility = Visibility::Visible;
        } else {
            commands.entity(entity).remove::<CursorCanDecide>();
            if let Ok((entity, _, _)) = insight_bar.get_single() {
                commands.entity(entity).despawn();
            }
            if let Ok(entity) = insight_bar_border.get_single() {
                commands.entity(entity).despawn();
            }
            insight.is_held = false;
            insight.grace.reset();

            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Resource)]
struct SpawnInsight(SystemId);

fn aquire_insight(
    mut reader: EventReader<AquireInsight>,
    mut state: ResMut<KingdomState>,
    selected_character: Query<&SelectedCharacter>,
    mut insight: ResMut<Insight>,
    mut commands: Commands,
    spawn_insight: Res<SpawnInsight>,
) {
    for _ in reader.read() {
        let Ok(selected_character) = &selected_character.get_single() else {
            error!("tried to aquired insight without a selected character");
            return;
        };

        if insight.character.as_ref() == Some(&selected_character.0) {
            info!("insight already aquired, returning");
            return;
        }

        insight.character = Some(selected_character.0.clone());

        commands.run_system(spawn_insight.0);

        info!("aquiring insight");
        state.heart_size -= 10.;
    }
}

#[derive(Component)]
struct InsightNode;

pub fn spawn_insight(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    insight: Res<Insight>,
    characters: Res<Assets<Character>>,
    state: Res<KingdomState>,
) {
    let window = &mut primary_window.single_mut();
    // window.cursor.visible = false;

    let character = characters.get(insight.character.as_ref().unwrap()).unwrap();
    let request = character.request(state.day).unwrap();

    commands.spawn(AudioBundle {
        source: server.load("audio/heartbeat.wav"),
        settings: PlaybackSettings::default(),
    });

    commands.spawn((
        InsightNode,
        SpriteBundle {
            texture: server.load("ui/insight_box.png"),
            transform: Transform::from_xyz(RES_WIDTH as f32 / -2., RES_HEIGHT as f32 / 2., 500.),
            ..Default::default()
        },
        HIGH_RES_LAYER,
    ));

    commands.spawn((
        InsightNode,
        SpriteBundle {
            texture: server.load("ui/insight_box.png"),
            transform: Transform::from_xyz(RES_WIDTH as f32 / 2., RES_HEIGHT as f32 / 2., 500.),
            ..Default::default()
        },
        HIGH_RES_LAYER,
    ));

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column.row(|row| {
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(
                            server.load("ui/Skill Tree/Icons/Unlocked/x2/Unlocked11.png"),
                        ),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::Start);
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!(" -{}", request.no.heart_size.abs() as u32),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
            });

            column.row(|row| {
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(server.load("ui/happiness.png")),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::Start);
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!(" -{}", request.no.happiness.abs() as u32),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
            });

            column.row(|row| {
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(server.load("ui/wealth.png")),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::Start);
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!(" -{}", request.no.wealth.abs() as u32),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
            });

            column.row(|row| {
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(
                            server.load("ui/Skill Tree/Icons/Unlocked/x2/Unlocked2.png"),
                        ),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::Start);
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!(
                            " -{}",
                            KingdomState::calculate_prosperity(
                                request.no.happiness.abs(),
                                request.no.wealth.abs()
                            )
                        ),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
            });
        })
        .style()
        // .column_gap(Val::Px(200.))
        // .row_gap(Val::Px(200.))
        .justify_content(JustifyContent::Start);

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column.row(|row| {
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!("+{} ", request.yes.heart_size.abs() as u32),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(
                            server.load("ui/Skill Tree/Icons/Unlocked/x2/Unlocked11.png"),
                        ),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::End);
            });

            column.row(|row| {
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!("+{} ", request.yes.happiness.abs() as u32),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(server.load("ui/happiness.png")),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::End);
            });

            column.row(|row| {
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!("+{} ", request.yes.wealth.abs() as u32),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(server.load("ui/wealth.png")),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::End);
            });

            column.row(|row| {
                row.spawn((
                    UiNode,
                    TextBundle::from_section(
                        &format!(
                            "+{} ",
                            KingdomState::calculate_prosperity(
                                request.yes.happiness.abs(),
                                request.yes.wealth.abs()
                            )
                        ),
                        TextStyle {
                            font_size: 30.0,
                            font: server.load(FONT_PATH),
                            ..Default::default()
                        },
                    ),
                ));
                row.spawn((
                    InsightNode,
                    ImageBundle {
                        image: UiImage::new(
                            server.load("ui/Skill Tree/Icons/Unlocked/x2/Unlocked2.png"),
                        ),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::End);
            });
        })
        .style()
        .justify_content(JustifyContent::Start)
        .right(Val::Percent(-95.2));
}

fn display_insight(
    insight: Res<Insight>,
    characters: Res<Assets<Character>>,
    selected_character: Query<&SelectedCharacter>,
) {
    if let Some(character) = &insight.character {
        if let Ok(selected_character) = selected_character.get_single() {
            if selected_character.0 == *character {
                if let Some(character) = characters.get(character) {
                    // println!("displaying insight: {:?}", character.name);
                }
            }
        }
    }
}

fn display_insight_tooltip(cursor: Query<&Style, With<CursorCanDecide>>) {}

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
        UiNode,
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

#[derive(Debug, Event, Clone, PartialEq, Eq, Reflect)]
pub enum Decision {
    Yes(Handle<Character>),
    No(Handle<Character>),
}

impl From<&Decision> for DecisionType {
    fn from(value: &Decision) -> Self {
        match *value {
            Decision::Yes(_) => DecisionType::Yes,
            Decision::No(_) => DecisionType::No,
        }
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect)]
pub enum DecisionType {
    Yes,
    No,
}

#[derive(Component)]
struct DecisionUi;

#[derive(Component)]
pub struct ShowSelectionUi;

fn should_show_selection_ui(
    mut commands: Commands,
    type_writer: Res<TypeWriter>,
    // show_selection: Option<Res<ShowSelectionUi>>,
    selected_player: Query<(Entity, Has<ShowSelectionUi>), With<SelectedCharacter>>,
) {
    let Ok((entity, show_selection)) = selected_player.get_single() else {
        return;
    };

    if type_writer.is_finished && !show_selection {
        commands.entity(entity).insert(ShowSelectionUi);
        info!("character finished dialogue, displaying selection ui");
    } else if show_selection && !type_writer.is_finished {
        commands.entity(entity).remove::<ShowSelectionUi>();
    }
}

fn selection_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &DecisionType),
        (With<Button>, Changed<Interaction>),
    >,
    // mut text_query: Query<&mut Text>,
    mut writer: EventWriter<Decision>,
    // show: Option<Res<ShowSelectionUi>>,
    mut root_ui: Query<&mut Visibility, With<DecisionUi>>,
    selected_character: Query<&SelectedCharacter, With<ShowSelectionUi>>,
) {
    let Ok(selected_character) = selected_character.get_single() else {
        return;
    };

    let mut vis = root_ui.single_mut();
    *vis = Visibility::Visible;

    for (interaction, mut color, decision) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // *color = PRESSED_BUTTON.into();
                // text.sections[0].value = "Press".to_string();
                *color = NORMAL_BUTTON.into();
                // border_color.0 = RED.into();

                // let decision_variation = if *decision == Decision::No(_) {
                //     -0.25
                // } else {
                //     0.
                // };
                commands.spawn(AudioBundle {
                    source: server.load(
                        "audio/retro/GameSFX/Weapon/reload/Retro Weapon Reload Best A 03.wav",
                    ),
                    settings: PlaybackSettings::default()
                        .with_volume(Volume::new(0.5))
                        .with_speed(1.8 - 0.),
                });

                match decision {
                    DecisionType::Yes => {
                        writer.send(Decision::Yes(selected_character.0.clone()));
                    }
                    DecisionType::No => {
                        writer.send(Decision::No(selected_character.0.clone()));
                    }
                }
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
}

fn end_animation(
    mut commands: Commands,
    mut heart_sprite: Query<
        (Entity, &mut Transform, &mut Visibility),
        (With<HeartUi>, With<Sprite>),
    >,
    state: Res<KingdomState>,
    audio: Query<Entity, With<Handle<AudioSource>>>,
    server: Res<AssetServer>,
) {
    let Ok((entity, mut heart, mut visibility)) = heart_sprite.get_single_mut() else {
        error!("could not retrieve heart sprite for loose animation");
        return;
    };

    for sink in audio.iter() {
        commands.entity(sink).despawn();
    }

    heart.translation = Vec3::new(0., 0., 999.);
    *visibility = Visibility::Hidden;

    if state.heart_size > 90. {
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
    } else if state.heart_size < 10. {
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
) {
    commands.entity(audio.single()).despawn();
    *heart_sprite.single_mut() = Visibility::Hidden;
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
