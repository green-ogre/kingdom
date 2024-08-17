use crate::{
    type_writer::{self, TypeWriter},
    GameState, KingdomState,
};
use bevy::{
    ecs::system::SystemId,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_common_assets::yaml::YamlAssetPlugin;
use rand::Rng;
use serde::Deserialize;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YamlAssetPlugin::<Character>::new(&["character.yaml"]))
            .insert_resource(TypeWriter::default())
            .insert_resource(SelectedCharacter::default())
            .add_systems(
                OnEnter(GameState::Main),
                (load_characters, choose_new_character).chain(),
            )
            .add_systems(PreUpdate, (load_character_sprite, hide_characters))
            .add_systems(
                Update,
                (show_character, character_ui).run_if(in_state(GameState::Main)),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct CharacterAssets {
    #[asset(path = "characters/jeremy.character.yaml")]
    jeremy: Handle<Character>,
    #[asset(path = "characters/merideth.character.yaml")]
    merideth: Handle<Character>,
}

#[derive(Debug, Resource)]
pub struct Characters {
    pub table: HashMap<&'static str, Handle<Character>>,
    pub current_key: &'static str,
    pub choose_new_character: SystemId,
}

fn load_characters(mut commands: Commands, character_assets: Res<CharacterAssets>) {
    let mut characters = HashMap::default();
    characters.insert("jeremy", character_assets.jeremy.clone());
    characters.insert("merideth", character_assets.merideth.clone());

    let choose_new_character = commands.register_one_shot_system(choose_new_character);
    commands.insert_resource(Characters {
        table: characters,
        current_key: "jeremy",
        choose_new_character,
    });
}

fn choose_new_character(
    server: Res<AssetServer>,
    mut characters: ResMut<Characters>,
    mut selected_character: ResMut<SelectedCharacter>,
    mut character_assets: ResMut<Assets<Character>>,
    mut type_writer: ResMut<TypeWriter>,
) {
    let peasants = ["merideth", "jeremy"];

    for peasant in peasants.iter() {
        if *peasant != characters.current_key {
            info!("selecting new character: {}", peasant);
            characters.current_key = peasant;
            selected_character.0 = characters.table.get(peasant).unwrap().clone();

            let character = character_assets.get_mut(&selected_character.0).unwrap();
            character.set_rand_request();

            let sfx = server.load("audio/interface/Wav/Cursor_tones/cursor_style_2.wav");
            *type_writer = TypeWriter::new(character.request().text.clone(), 0.025, sfx);

            break;
        }
    }
}

fn load_character_sprite(
    mut commands: Commands,
    mut reader: EventReader<AssetEvent<Character>>,
    server: Res<AssetServer>,
    mut characters: ResMut<Assets<Character>>,
) {
    for character in reader.read() {
        match character {
            AssetEvent::Added { id } => {
                let character = characters.get_mut(*id).unwrap();
                let texture = server.load(character.sprite_path.trim().to_string());
                character.texture = Some(texture.clone());
                character.sprite = Some(
                    commands
                        .spawn((
                            SpriteBundle {
                                visibility: Visibility::Hidden,
                                texture,
                                ..Default::default()
                            },
                            CharacterSprite,
                        ))
                        .id(),
                );
            }
            _ => {}
        }
    }
}

#[derive(Component)]
struct CharacterSprite;

#[derive(Component)]
pub enum CharacterUi {
    Name,
    Request,
}

fn character_ui(
    mut commands: Commands,
    selected_character: Res<SelectedCharacter>,
    characters: Res<Assets<Character>>,
    mut character_ui: Query<(&mut Text, &CharacterUi)>,
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

    if let Some(character) = characters.get(&selected_character.0) {
        for (mut text, ui) in character_ui.iter_mut() {
            match ui {
                CharacterUi::Name => {
                    if selected_character.is_changed() {
                        text.sections[0].value = format!("Character: {:?}", character.name);
                    }
                }
                CharacterUi::Request => {
                    text.sections[0].value = type_writer.slice_with_line_wrap().into();
                }
            }
        }
    }
}

fn hide_characters(mut character_sprites: Query<&mut Visibility, With<CharacterSprite>>) {
    for mut vis in character_sprites.iter_mut() {
        *vis = Visibility::Hidden;
    }
}

fn show_character(
    selected_character: Res<SelectedCharacter>,
    characters: Res<Assets<Character>>,
    mut character_sprites: Query<&mut Visibility, With<Sprite>>,
) {
    if let Some(character) = characters.get(&selected_character.0) {
        if let Some(sprite) = character.sprite {
            if let Ok(mut vis) = character_sprites.get_mut(sprite) {
                *vis = Visibility::Visible;
            }
        }
    }
}

#[derive(Debug, Default, Resource, Reflect)]
pub struct SelectedCharacter(pub Handle<Character>);

#[derive(Debug, Deserialize, Asset, Component, Reflect)]
pub struct Character {
    pub name: String,
    pub class: Class,
    pub sprite_path: String,
    pub requests: Vec<Request>,
    #[serde(skip)]
    current_request: usize,
    #[serde(skip)]
    pub texture: Option<Handle<Image>>,
    #[serde(skip)]
    pub sprite: Option<Entity>,
}

impl Character {
    pub fn request(&self) -> &Request {
        &self.requests[self.current_request]
    }

    pub fn set_rand_request(&mut self) {
        self.current_request = rand::thread_rng().gen_range(0..self.requests.len());
    }
}

#[derive(Debug, Deserialize, Asset, Component, Reflect)]
pub enum Class {
    Peasant,
}

#[derive(Debug, Deserialize, Asset, Component, Reflect, Clone)]
pub struct Request {
    pub text: String,
    pub yes: KingdomState,
    pub no: KingdomState,
}
