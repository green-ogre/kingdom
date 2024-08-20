use crate::{
    pixel_perfect::HIGH_RES_LAYER,
    type_writer::{self, TypeWriter},
    ui::{Cursor, InsightToolTip, UiNode, FONT_PATH},
    GameState, SkipRemove,
};
use bevy::{
    audio::{PlaybackMode, Volume},
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    render::view::RenderLayers,
    ui::ContentSize,
    window::PrimaryWindow,
};
use bevy_hanabi::prelude::*;
use sickle_ui::{prelude::*, ui_commands::UpdateStatesExt};
use std::time::Duration;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::MainMenu),
            (setup_effect, setup, setup_cursor),
        )
        .add_systems(Update, parallax_sprites)
        .add_systems(Update, (update_text,).run_if(in_state(GameState::MainMenu)))
        .add_systems(Update, crate::ui::update_cursor)
        .add_plugins(HanabiPlugin);
    }
}

#[derive(Component)]
pub struct ParallaxSprite(pub f32);

pub fn setup_cursor(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let window = &mut primary_window.single_mut();
    window.cursor.visible = false;

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column.row(|row| {
                row.spawn((
                    UiNode,
                    ImageBundle {
                        image: UiImage::new(server.load("ui/cursor.png")),
                        // transform: Transform::from_scale(Vec3::splat(8.)),
                        z_index: ZIndex::Global(100),
                        style: Style {
                            // max_size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            // align_items: AlignItems::Center,
                            // justify_content: JustifyContent::SpaceAround,
                            ..default()
                        },
                        // calculated_size: ContentSize::fixed_size(Vec2::new(240., 125.)),
                        ..Default::default()
                    },
                    Cursor,
                ))
                .style()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.));
            });
        })
        .style()
        .width(Val::Percent(100.))
        .height(Val::Percent(100.))
        .justify_content(JustifyContent::End);

    commands
        .ui_builder(UiRoot)
        .column(|column| {
            column
                .row(|row| {
                    row.insert(InsightToolTip);
                    row.spawn((
                        UiNode,
                        ImageBundle {
                            image: UiImage::new(server.load("ui/tool_tip_mouse.png")),
                            transform: Transform::from_scale(Vec3::splat(8.)),
                            z_index: ZIndex::Global(100),
                            style: Style {
                                // max_size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                // align_items: AlignItems::Start,
                                // justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            // calculated_size: ContentSize::fixed_size(Vec2::new(240., 125.)),
                            ..Default::default()
                        },
                    ));
                })
                .insert(Visibility::Hidden)
                .style()
                .width(Val::Percent(100.))
                .height(Val::Percent(100.));
        })
        .style()
        .width(Val::Percent(100.))
        .height(Val::Percent(100.))
        .justify_content(JustifyContent::End);
}

#[derive(Resource)]
struct EnterMorningTimer(Timer, u32, bool);

#[derive(Component)]
struct Intro;

fn setup(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut type_writer: ResMut<TypeWriter>,
    mut cursor: Query<&mut Visibility, With<Cursor>>,
) {
    for mut vis in cursor.iter_mut() {
        info!("showing cursor");
        *vis = Visibility::Visible;
    }

    commands.insert_resource(EnterMorningTimer(
        Timer::from_seconds(5., TimerMode::Repeating),
        0,
        false,
    ));

    commands.spawn((
        AudioBundle {
            source: server.load("audio/birds-19624.mp3"),
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(0.5),
                ..Default::default()
            },
        },
        Intro,
    ));

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
            left: Val::Px(600.),
            top: Val::Px(200.),
            // max_width: Val::Px(1000.),
            ..Default::default()
        }),
        IntroText,
        Intro,
    ));

    let sfx = server.load("audio/cursor_style_2_rev.wav");
    *type_writer = TypeWriter::new(
        "Your heart, dear King, it weighs the will of one\nWho seeks of you a choice, a thing undone. "
            .into(),
        0.05,
        sfx,
    );

    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/1.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-22.)),
            ..Default::default()
        },
        // HIGH_RES_LAYER,
        Intro,
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
        Intro,
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
        Intro,
    ));
}

fn parallax_sprites(
    windows: Query<&Window>,
    mut sprites: Query<(&mut Transform, &ParallaxSprite)>,
) {
    let window = windows.single();

    if let Some(world_position) = window.cursor_position() {
        for (mut transform, parallax) in sprites.iter_mut() {
            transform.translation.x = (world_position.x - 960.) * parallax.0;
            transform.translation.y = (world_position.y - 540.) * parallax.0;
        }
    }
}

#[derive(Component)]
struct MainMenuParticles;

fn setup_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..Default::default()
            },
            ..Default::default()
        },
        RenderLayers::layer(2),
    ));

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
    .with_simulation_space(SimulationSpace::Local)
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
            effect: ParticleEffect::new(effect_asset),
            transform: Transform::from_translation(Vec3::default().with_z(300.))
                .with_scale(Vec3::splat(1.)),
            ..Default::default()
        },
        RenderLayers::layer(2),
        MainMenuParticles,
        Intro,
        ParallaxSprite(0.045),
    ));
}

#[derive(Component)]
struct IntroText;

fn update_text(
    mut commands: Commands,
    mut intro_text: Query<&mut Text, With<IntroText>>,
    mut type_writer: ResMut<TypeWriter>,
    mut reader: EventReader<KeyboardInput>,
    time: Res<Time>,
    mut timer: ResMut<EnterMorningTimer>,
    enitites: Query<Entity, With<Intro>>,
    server: Res<AssetServer>,
) {
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

            if timer.1 == 2 {
                let sfx = server.load("audio/cursor_style_2_rev.wav");
                let line = "Closely must You watch this beating sieve;\nToo much, too little, and Your heart will give.";
                *type_writer = TypeWriter::new(line.into(), 0.035, sfx);
                timer.0.set_duration(Duration::from_secs_f32(7.));
            }

            if timer.1 == 3 {
                commands.next_state(GameState::Main);
                for entity in enitites.iter() {
                    commands.entity(entity).despawn();
                }
                return;
            }
        }
    }

    timer.0.tick(time.delta());

    if timer.0.finished() {
        timer.1 += 1;

        if timer.1 == 3 {
            let sfx = server.load("audio/cursor_style_2_rev.wav");
            let line = "Closely must You watch this beating sieve;\nToo much, too little, and Your heart will give.";
            *type_writer = TypeWriter::new(line.into(), 0.05, sfx);
            // timer.0.set_duration(Duration::from_secs_f32(2.));
        }

        if timer.1 == 3 {
            commands.next_state(GameState::Main);
            for entity in enitites.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }

    if timer.1 > 0 {
        type_writer.increment(&time);
        type_writer.try_play_sound(&mut commands);

        let mut text = intro_text.single_mut();
        text.sections[0].value = type_writer.slice_with_line_wrap().into();
    }
}
