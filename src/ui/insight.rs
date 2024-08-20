use bevy::{audio::PlaybackMode, ecs::system::SystemId, prelude::*, window::PrimaryWindow};
use sickle_ui::prelude::*;

use crate::{
    character::{Character, SelectedCharacter},
    pixel_perfect::{HIGH_RES_LAYER, RES_HEIGHT, RES_WIDTH},
    state::KingdomState,
    CharacterSet, GameState,
};

use super::{AquireInsight, FONT_PATH};

pub struct InsightPlugin;

impl Plugin for InsightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Main), startup)
            .add_systems(PostUpdate, aquire_insight.in_set(CharacterSet))
            .insert_resource(Insight::default());
    }
}

fn startup(mut commands: Commands) {
    let id = commands.register_one_shot_system(spawn_insight);
    commands.insert_resource(SpawnInsight(id));
    let id = commands.register_one_shot_system(despawn_insight);
    commands.insert_resource(DespawnInsight(id));
}

#[derive(Resource)]
pub struct Insight {
    pub grace: Timer,
    pub is_held: bool,
    pub character: Option<Handle<Character>>,
    pub charge: f32,
}

pub const INSIGHT_CHARGE_TIME: f32 = 2.0;

impl Default for Insight {
    fn default() -> Self {
        Self {
            grace: Timer::from_seconds(INSIGHT_CHARGE_TIME, TimerMode::Repeating),
            is_held: false,
            character: None,
            charge: 0.,
        }
    }
}

#[derive(Resource)]
struct SpawnInsight(SystemId);

#[derive(Resource)]
pub struct DespawnInsight(pub SystemId);

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
        state.heart_size -= 1.;
    }
}

#[derive(Component)]
pub struct InsightNode;

pub fn despawn_insight(mut commands: Commands, nodes: Query<Entity, With<InsightNode>>) {
    for node in nodes.iter() {
        commands.entity(node).despawn();
    }
}

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
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..Default::default()
        },
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

    let get_leader = |val: f32| {
        if val >= 0. {
            "+"
        } else {
            "-"
        }
    };

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
                    InsightNode,
                    TextBundle::from_section(
                        &format!(
                            " {}{}",
                            get_leader(request.no.heart_size),
                            request.no.heart_size.abs() as u32
                        ),
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
                    InsightNode,
                    TextBundle::from_section(
                        &format!(
                            " {}{}",
                            get_leader(request.no.happiness),
                            request.no.happiness.abs() as u32
                        ),
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
                    InsightNode,
                    TextBundle::from_section(
                        &format!(
                            " {}{}",
                            get_leader(request.no.wealth),
                            request.no.wealth.abs() as u32
                        ),
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

                let prosp = KingdomState::calculate_prosperity(
                    request.no.happiness.abs(),
                    request.no.wealth.abs(),
                );

                row.spawn((
                    InsightNode,
                    TextBundle::from_section(
                        &format!(" {}{prosp}", get_leader(prosp)),
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
                    InsightNode,
                    TextBundle::from_section(
                        &format!(
                            "{}{} ",
                            get_leader(request.yes.heart_size),
                            request.yes.heart_size.abs() as u32
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
                    InsightNode,
                    TextBundle::from_section(
                        &format!(
                            "{}{} ",
                            get_leader(request.yes.happiness),
                            request.yes.happiness.abs() as u32
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
                    InsightNode,
                    TextBundle::from_section(
                        &format!(
                            "{}{} ",
                            get_leader(request.yes.wealth),
                            request.yes.wealth.abs() as u32
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
                        image: UiImage::new(server.load("ui/wealth.png")),
                        z_index: ZIndex::Global(100),
                        style: Style { ..default() },
                        ..Default::default()
                    },
                ))
                .style()
                .justify_content(JustifyContent::End);
            });

            let prosp = KingdomState::calculate_prosperity(
                request.yes.happiness.abs(),
                request.yes.wealth.abs(),
            );

            column.row(|row| {
                row.spawn((
                    InsightNode,
                    TextBundle::from_section(
                        &format!("{}{prosp} ", get_leader(prosp)),
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
