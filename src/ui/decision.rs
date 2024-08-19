use super::{Cursor, InsightToolTip, UiNode, FONT_PATH};
use crate::{
    character::{Character, SelectedCharacter},
    type_writer::TypeWriter,
    CharacterSet,
};
use bevy::{
    audio::Volume,
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};

pub struct DecisionPlugin;

impl Plugin for DecisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, should_show_selection_ui.in_set(CharacterSet))
            .add_systems(Update, selection_ui.in_set(CharacterSet))
            .add_event::<Decision>();
    }
}

#[derive(Component)]
struct DecisionUi;

#[derive(Component)]
pub struct ShowSelectionUi;

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

#[derive(Component)]
enum DecisionBox {
    Yes,
    No,
}

fn selection_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut writer: EventWriter<Decision>,
    selected_character: Query<&SelectedCharacter, With<ShowSelectionUi>>,
    mut decision_boxes: Query<(&DecisionBox, &mut TextureAtlas)>,
    mut decision_box_entities: Query<Entity, With<DecisionBox>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut tool_tip: Query<
        (Entity, &mut Style, &mut Visibility),
        (With<InsightToolTip>, Without<Cursor>),
    >,
    windows: Query<&Window>,
    mut input: EventReader<MouseButtonInput>,
) {
    let Ok(selected_character) = selected_character.get_single() else {
        for entity in decision_box_entities.iter() {
            commands.entity(entity).despawn();
        }

        return;
    };

    let window = windows.single();

    let (tool_tip_entity, mut tool_tip_style, mut visibility) = tool_tip.single_mut();

    if decision_boxes.is_empty() {
        let layout = TextureAtlasLayout::from_grid(UVec2::new(240, 135), 2, 1, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);
        commands.spawn((
            SpriteBundle {
                texture: server.load("ui/menu_box.png"),
                transform: Transform::from_xyz(-75., 40., 400.),
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
            DecisionBox::No,
            UiNode,
        ));
        commands.spawn((
            SpriteBundle {
                texture: server.load("ui/menu_box.png"),
                transform: Transform::from_xyz(75., 40., 400.),
                ..Default::default()
            },
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
            DecisionBox::Yes,
            UiNode,
        ));
        commands.spawn((
            DecisionBox::No,
            TextBundle::from_section(
                "I Do Not Concur",
                TextStyle {
                    font_size: 70.0,
                    font: server.load(FONT_PATH),
                    ..Default::default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(180.),
                left: Val::Px(140.),
                ..Default::default()
            })
            .with_text_justify(JustifyText::Center),
            UiNode,
        ));
        commands.spawn((
            DecisionBox::Yes,
            TextBundle::from_section(
                "I Concur",
                TextStyle {
                    font_size: 70.0,
                    font: server.load(FONT_PATH),
                    ..Default::default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(180.),
                left: Val::Percent(76.),
                ..Default::default()
            })
            .with_text_justify(JustifyText::Center),
            UiNode,
        ));
    }

    if let Some(mouse) = window.cursor_position() {
        let did_click = input
            .read()
            .any(|i| i.state == ButtonState::Pressed && i.button == MouseButton::Left);

        let click_sfx = "/home/shane/dev/kingdom/assets/audio/stamp-102627.mp3";

        if mouse.x > 128. && mouse.x < 591. && mouse.y > 141. && mouse.y < 290. {
            if did_click {
                commands.spawn(AudioBundle {
                    source: server.load(click_sfx),
                    settings: PlaybackSettings::default()
                        .with_volume(Volume::new(0.5))
                        .with_speed(1.8),
                });
                writer.send(Decision::No(selected_character.0.clone()));
            }

            *visibility = Visibility::Hidden;

            for (box_ty, mut atlas) in decision_boxes.iter_mut() {
                match box_ty {
                    DecisionBox::No => {
                        atlas.index = 1;
                    }
                    _ => {}
                }
            }
        } else if mouse.x > 1330. && mouse.x < 1791. && mouse.y > 142. && mouse.y < 290. {
            if did_click {
                commands.spawn(AudioBundle {
                    source: server.load(click_sfx),
                    settings: PlaybackSettings::default()
                        .with_volume(Volume::new(0.5))
                        .with_speed(1.8),
                });
                writer.send(Decision::Yes(selected_character.0.clone()));
            }

            *visibility = Visibility::Hidden;

            for (box_ty, mut atlas) in decision_boxes.iter_mut() {
                match box_ty {
                    DecisionBox::Yes => {
                        atlas.index = 1;
                    }
                    _ => {}
                }
            }
        } else {
            for (_, mut atlas) in decision_boxes.iter_mut() {
                atlas.index = 0;
            }
        }
    }
}
