use crate::{
    pixel_perfect::{OuterCamera, HIGH_RES_LAYER, PIXEL_PERFECT_LAYER},
    type_writer::{self, TypeWriter},
    ui::{Cursor, Insight, InsightToolTip, UiNode},
    GameState,
};
use bevy::{
    audio::{PlaybackMode, Volume},
    ecs::world,
    input::{keyboard::KeyboardInput, ButtonState},
    math::VectorSpace,
    prelude::*,
    render::view::RenderLayers,
    ui::ContentSize,
    window::PrimaryWindow,
};
use bevy_hanabi::prelude::*;
use sickle_ui::prelude::*;

pub const FONT_PATH: &'static str = "ui/alagard.ttf";

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), (setup_effect, setup))
            .add_systems(
                Update,
                (parallax_sprites, update_text).run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(Update, crate::ui::update_cursor)
            .add_plugins(HanabiPlugin);
    }
}

#[derive(Component)]
struct ParallaxSprite(f32);

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
                        transform: Transform::from_scale(Vec3::splat(8.)),
                        z_index: ZIndex::Global(100),
                        style: Style {
                            // max_size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            // align_items: AlignItems::Center,
                            // justify_content: JustifyContent::SpaceAround,
                            ..default()
                        },
                        calculated_size: ContentSize::fixed_size(Vec2::new(240., 125.)),
                        ..Default::default()
                    },
                    Cursor,
                ));

                row.row(|row| {
                    row.insert(InsightToolTip);
                    row.spawn((
                        UiNode,
                        ImageBundle {
                            image: UiImage::new(server.load("ui/tool_tip_mouse.png")),
                            transform: Transform::from_scale(Vec3::splat(8.)),
                            z_index: ZIndex::Global(100),
                            style: Style {
                                // max_size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                align_items: AlignItems::Start,
                                // justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            // calculated_size: ContentSize::fixed_size(Vec2::new(240., 125.)),
                            ..Default::default()
                        },
                    ));
                    row.spawn((
                        UiNode,
                        TextBundle::from_section(
                            "Aquire Insight",
                            TextStyle {
                                font_size: 30.0,
                                font: server.load(FONT_PATH),
                                ..Default::default()
                            },
                        ),
                    ));
                })
                .insert(Visibility::Hidden)
                .style()
                .column_gap(Val::Percent(20.))
                // .row_gap(Val::Percent(20.))
                // .position_type(PositionType::Absolute)
                .justify_items(JustifyItems::Start);
            });
        })
        .style()
        .column_gap(Val::Px(200.))
        .row_gap(Val::Px(200.))
        .justify_content(JustifyContent::End);
}

fn setup(mut commands: Commands, server: Res<AssetServer>, mut type_writer: ResMut<TypeWriter>) {
    commands.spawn(AudioBundle {
        source: server.load("audio/birds-19624.mp3"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.5),
            ..Default::default()
        },
    });

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
        UiNode,
    ));

    let sfx = server.load("audio/interface/Wav/Cursor_tones/cursor_style_2.wav");
    *type_writer = TypeWriter::new("This is some cool text isn't it?".into(), 0.035, sfx);

    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/1.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-22.)),
            ..Default::default()
        },
        HIGH_RES_LAYER,
    ));
    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/2.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-21.)),
            ..Default::default()
        },
        ParallaxSprite(0.005),
        HIGH_RES_LAYER,
    ));
    commands.spawn((
        SpriteBundle {
            texture: server.load("Nature Landscapes Free Pixel Art/nature_4/3.png"),
            transform: Transform::from_scale(Vec3::splat(1.))
                .with_translation(Vec3::default().with_z(-20.)),
            ..Default::default()
        },
        ParallaxSprite(0.001),
        HIGH_RES_LAYER,
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
) {
    type_writer.increment(&time);
    type_writer.try_play_sound(&mut commands);

    for input in reader.read() {
        if matches!(input, KeyboardInput { key_code,  state, .. } if *key_code == KeyCode::Space && *state == ButtonState::Pressed)
        {
            type_writer.finish();
        }
    }

    let mut text = intro_text.single_mut();
    // text.sections[0].style.color.set_alpha(1.);
    text.sections[0].value = type_writer.slice_with_line_wrap().into();
}
