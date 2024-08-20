use crate::animated_sprites::{AnimationIndices, AnimationTimer};
use crate::character::{CharacterUi, SelectedCharacter};
use crate::pixel_perfect::{HIGH_RES_LAYER, PIXEL_PERFECT_LAYER, RES_HEIGHT, RES_WIDTH};
use crate::state::{KingdomState, NewHeartSize, MAX_HAPPINESS, MAX_HEART_SIZE, MAX_WEALTH};
use crate::time_state::TimeState;
use crate::{CharacterSet, GameState};
use background::BackgroundPlugin;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState;
use bevy::{audio::Volume, prelude::*};
use bevy_tweening::*;
use decision::{DecisionPlugin, ShowSelectionUi};
use insight::{Insight, InsightPlugin};
use lens::{SpriteColorLens, TransformRotateZLens, TransformScaleLens};
use serde::Deserialize;
use sickle_ui::SickleUiPlugin;
use std::time::Duration;

pub mod background;
pub mod decision;
pub mod insight;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TweeningPlugin,
            SickleUiPlugin,
            InsightPlugin,
            DecisionPlugin,
            BackgroundPlugin,
        ))
        .add_systems(
            OnEnter(GameState::Main),
            (
                startup_debug,
                startup,
                setup_ui,
                setup_state_bars,
                setup_heart_ui,
            ),
        )
        .add_systems(
            Update,
            (heart_ui, mask_ui, display_state_bars).in_set(CharacterSet),
        )
        .add_systems(
            Update,
            component_animator_system::<AudioSink>.in_set(AnimationSystem::AnimationUpdate),
        )
        .add_event::<AquireInsight>();
    }
}

pub fn hex_to_vec4(hex: u32) -> Vec4 {
    Vec4::new(
        ((hex >> 16) & 0xFF) as f32 / 255.,
        ((hex >> 8) & 0xFF) as f32 / 255.,
        (hex & 0xFF) as f32 / 255.,
        1.0,
    )
}

#[derive(Component)]
pub struct HeartUi;

pub const HEART_SCALE: f32 = 2.;
pub const FONT_PATH: &'static str = "ui/small_pixel-7.ttf";

#[derive(Component)]
pub struct UiNode;

fn startup_debug(mut commands: Commands, mut state: ResMut<KingdomState>) {
    // commands.next_state(TimeState::Evening);
    // commands.next_state(GameState::Loose);
    // state.heart_size = 0.;
    // commands.next_state(TimeState::Day);

    // state.day = 0;
}

fn startup(mut commands: Commands, mut state: ResMut<KingdomState>, server: Res<AssetServer>) {
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
}

fn setup_ui(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: server.load("ui/ui.png"),
            transform: Transform::from_xyz(0., 0., 10.),
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

#[derive(Component, Default, Reflect)]
pub struct InsightBar(f32);

#[derive(Component)]
pub struct InsightBarBorder;

#[derive(Event)]
pub struct AquireInsight;

#[derive(Component)]
pub struct InsightChargeSfx;

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
    mut insight_bar_border: Query<
        (Entity, &mut Transform),
        (With<InsightBarBorder>, Without<InsightBar>),
    >,
    insight_sfx: Query<Entity, With<InsightChargeSfx>>,
    state: Res<KingdomState>,
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

        tool_tip_style.left = Val::Px(left + 200.);
        tool_tip_style.top = Val::Px(top + 100.);

        // println!("{top:?}");
        if top < -350. && !selected_character.is_empty() {
            commands.entity(entity).insert(CursorCanDecide);
            insight.grace.tick(time.delta());

            if let Ok((_, mut fill_transform, mut bar)) = insight_bar.get_single_mut() {
                if let Ok((_, mut border_transform)) = insight_bar_border.get_single_mut() {
                    bar.0 = insight.grace.remaining().as_secs_f32()
                        / insight.grace.duration().as_secs_f32();
                    fill_transform.scale.x = (1.0 - bar.0) * 0.5;

                    border_transform.translation.x = world_position.x / 8. - RES_WIDTH as f32 / 2.;
                    border_transform.translation.y =
                        -world_position.y / 8. + RES_HEIGHT as f32 / 2. - 10.;

                    fill_transform.translation.x =
                        world_position.x / 8. - RES_WIDTH as f32 / 2. - (74. / 2. * bar.0);
                    fill_transform.translation.y =
                        -world_position.y / 8. + RES_HEIGHT as f32 / 2. - 10.;
                }
            }

            for input in reader.read() {
                if input.button == MouseButton::Right || input.button == MouseButton::Left {
                    match input.state {
                        ButtonState::Pressed => {
                            if insight.is_held == false
                                && insight.character.as_ref()
                                    != selected_character.iter().next().map(|s| &s.1 .0)
                                && state.day > 0
                            {
                                let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar x1.png";
                                commands.spawn((
                                    SpriteBundle {
                                        texture: server.load(bar_path),
                                        transform: Transform::from_xyz(
                                            world_position.x / 8. - RES_WIDTH as f32 / 2.,
                                            -world_position.y / 8. + RES_HEIGHT as f32 / 2. - 10.,
                                            310.,
                                        )
                                        .with_scale(Vec3::splat(0.5)),
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
                                        transform: Transform::from_xyz(
                                            world_position.x / 8. - RES_WIDTH as f32 / 2.,
                                            -world_position.y / 8. + RES_HEIGHT as f32 / 2. - 10.,
                                            310.,
                                        )
                                        .with_scale(Vec3::new(0., 0.5, 0.5)),
                                        ..Default::default()
                                    },
                                    InsightBar(0.),
                                    HIGH_RES_LAYER,
                                ));
                                let sfx_path = "audio/sci-fi-sound-effect-designed-circuits-sfx-tonal-15-202059.mp3";
                                commands.spawn((
                                    AudioBundle {
                                        source: server.load(sfx_path),
                                        settings: PlaybackSettings::default()
                                            .with_volume(Volume::new(0.4)),
                                    },
                                    InsightChargeSfx,
                                ));
                            }

                            insight.is_held = true;
                        }
                        ButtonState::Released => {
                            insight.is_held = false;
                            if let Ok((entity, _, _)) = insight_bar.get_single() {
                                commands.entity(entity).despawn();
                            }
                            if let Ok((entity, _)) = insight_bar_border.get_single() {
                                commands.entity(entity).despawn();
                            }
                            insight.grace.reset();
                            for sfx in insight_sfx.iter() {
                                commands.entity(sfx).despawn();
                            }
                        }
                    }

                    insight.grace.reset();
                }
            }

            if insight.grace.finished() && insight.is_held && state.day > 0 {
                writer.send(AquireInsight);
                insight.is_held = false;
                if let Ok((entity, _, _)) = insight_bar.get_single() {
                    commands.entity(entity).despawn();
                }
                if let Ok((entity, _)) = insight_bar_border.get_single() {
                    commands.entity(entity).despawn();
                }
                insight.grace.reset();
                for sfx in insight_sfx.iter() {
                    commands.entity(sfx).despawn();
                }
            }

            if insight.character.as_ref() != selected_character.iter().next().map(|s| &s.1 .0) {
                if !selected_character.is_empty() && state.day > 0 {
                    *visibility = Visibility::Visible;
                }
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            commands.entity(entity).remove::<CursorCanDecide>();
            if let Ok((entity, _, _)) = insight_bar.get_single() {
                commands.entity(entity).despawn();
            }
            if let Ok((entity, _)) = insight_bar_border.get_single() {
                commands.entity(entity).despawn();
            }
            insight.is_held = false;
            insight.grace.reset();
            for sfx in insight_sfx.iter() {
                commands.entity(sfx).despawn();
            }

            *visibility = Visibility::Hidden;
        }
    }
}

#[derive(Component)]
pub enum StatBar {
    Wealth,
    Happiness,
    Heart,
}

#[derive(Component)]
struct Filler;

fn setup_state_bars(mut commands: Commands, server: Res<AssetServer>) {
    const BAR_X: f32 = -76. - 74. / 2.;

    let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar x1.png";
    commands.spawn((
        StatBar::Heart,
        SpriteBundle {
            texture: server.load(bar_path),
            transform: Transform::from_xyz(-76., -10., 310.).with_scale(Vec3::splat(0.5)),
            ..Default::default()
        },
        Name::new("Stat bar"),
        HIGH_RES_LAYER,
    ));
    commands.spawn((
        StatBar::Heart,
        SpriteBundle {
            texture: server.load("ui/Skill Tree/Icons/Unlocked/x1/Unlocked11.png"),
            transform: Transform::from_xyz(-116., -10., 310.).with_scale(Vec3::splat(0.25)),
            ..Default::default()
        },
        Name::new("Heart"),
        HIGH_RES_LAYER,
    ));
    let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar_filler x1.png";
    commands.spawn((
        StatBar::Heart,
        Filler,
        SpriteBundle {
            texture: server.load(bar_path),
            transform: Transform::from_xyz(BAR_X, -10., 309.).with_scale(Vec3::new(0., 0.5, 0.5)),
            ..Default::default()
        },
        Name::new("Stat filler"),
        HIGH_RES_LAYER,
    ));
    let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar x1.png";
    commands.spawn((
        StatBar::Happiness,
        SpriteBundle {
            texture: server.load(bar_path),
            transform: Transform::from_xyz(-76., 0., 310.).with_scale(Vec3::splat(0.5)),
            ..Default::default()
        },
        Name::new("Stat bar"),
        HIGH_RES_LAYER,
    ));
    commands.spawn((
        StatBar::Happiness,
        SpriteBundle {
            texture: server.load("ui/happiness.png"),
            transform: Transform::from_xyz(-116., 0., 310.).with_scale(Vec3::splat(0.25 / 2.)),
            ..Default::default()
        },
        HIGH_RES_LAYER,
    ));
    let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar_filler x1.png";
    commands.spawn((
        StatBar::Happiness,
        Filler,
        SpriteBundle {
            texture: server.load(bar_path),
            transform: Transform::from_xyz(BAR_X, 0., 309.).with_scale(Vec3::new(0., 0.5, 0.5)),
            ..Default::default()
        },
        Name::new("Stat filler"),
        HIGH_RES_LAYER,
    ));
    let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar x1.png";
    commands.spawn((
        StatBar::Wealth,
        SpriteBundle {
            texture: server.load(bar_path),
            transform: Transform::from_xyz(-76., 10., 310.).with_scale(Vec3::splat(0.5)),
            ..Default::default()
        },
        Name::new("Stat bar"),
        HIGH_RES_LAYER,
    ));
    commands.spawn((
        StatBar::Wealth,
        SpriteBundle {
            texture: server.load("ui/wealth.png"),
            transform: Transform::from_xyz(-116., 10., 310.).with_scale(Vec3::splat(0.25 / 2.)),
            ..Default::default()
        },
        HIGH_RES_LAYER,
    ));
    let bar_path = "ui/Boss bar/Mini Boss bar/mioni_boss_bar_filler x1.png";
    commands.spawn((
        StatBar::Wealth,
        Filler,
        SpriteBundle {
            texture: server.load(bar_path),
            transform: Transform::from_xyz(BAR_X, 10., 309.).with_scale(Vec3::new(0., 0.5, 0.5)),
            ..Default::default()
        },
        Name::new("Stat filler"),
        HIGH_RES_LAYER,
    ));
}

fn display_state_bars(
    mut bars: Query<(&mut Transform, &StatBar, &mut Visibility, Has<Filler>)>,
    state: Res<KingdomState>,
    time_state: Res<State<TimeState>>,
) {
    if *time_state.get() != TimeState::Day && *time_state.get() != TimeState::Night {
        for (_, _, mut vis, _) in bars.iter_mut() {
            *vis = Visibility::Hidden;
        }
    } else {
        for (mut sprite_transform, bar, mut vis, filler) in bars.iter_mut() {
            *vis = Visibility::Visible;

            if filler {
                let new_scale = match bar {
                    StatBar::Wealth => (state.wealth / MAX_WEALTH * 0.5).clamp(0., 0.5),
                    StatBar::Heart => (state.heart_size / MAX_HEART_SIZE * 0.5).clamp(0., 0.5),
                    StatBar::Happiness => (state.happiness / MAX_HAPPINESS * 0.5).clamp(0., 0.5),
                };

                let diff = sprite_transform.scale.x - new_scale;
                sprite_transform.scale.x = new_scale;
                sprite_transform.translation.x -= diff * 74.;
            }
        }
    }
}

#[derive(Debug, Default, Deserialize, Asset, Component, Reflect, Clone, PartialEq, Eq, Copy)]
pub enum Mask {
    Happy,
    Neutral,
    Sad,
    #[default]
    None,
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

#[derive(Component)]
struct HeartFlash;

fn setup_heart_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = server.load("ui/heart_sprite_sheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 7 };
    let transform = Transform::from_xyz(-90., -45., 100.).with_scale(Vec3::splat(HEART_SCALE / 2.));

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
            ..Default::default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_indices.first,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        HeartUi,
        UiNode,
        // Animator::new(pulse),
        PIXEL_PERFECT_LAYER,
    ));

    let pulse = Tween::new(
        // Use a quadratic easing on both endpoints.
        EaseFunction::QuadraticInOut,
        // Animation time (one way only; for ping-pong it takes 2 seconds
        // to come back to start).
        Duration::from_secs_f32(0.5),
        // The lens gives the Animator access to the Transform component,
        // to animate it. It also contains the start and end values associated
        // with the animation ratios 0. and 1.
        SpriteColorLens {
            start: LinearRgba::new(165. / 255., 48. / 255., 48. / 255., 1.).into(),
            end: LinearRgba::new(165. / 255., 48. / 255., 48. / 255., 0.).into(),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

    commands.spawn((
        HeartFlash,
        SpriteBundle {
            texture: server.load("ui/heart_flash.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(100.)),
            visibility: Visibility::Hidden,
            // transform,
            // .with_scale(Vec3::splat(HEART_SCALE * (50. / 130.))),
            ..Default::default()
        },
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
    mut heart_flash: Query<&mut Visibility, With<HeartFlash>>,
) {
    if let Ok(mut vis) = heart_flash.get_single_mut() {
        if state.heart_size == 5. || state.heart_size == 1. {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    if let Ok((entity, mut transform)) = heart.get_single_mut() {
        for new_size in reader.read() {
            transform.scale = Vec3::splat(HEART_SCALE * (new_size.0 / MAX_HEART_SIZE));

            commands.spawn(AudioBundle {
                source: server.load("audio/heartbeat.wav"),
                settings: PlaybackSettings::DESPAWN,
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
                .insert(Animator::new(Tracks::new([rotate])));
        }
    }
}
