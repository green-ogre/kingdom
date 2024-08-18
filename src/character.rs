use crate::GameState;
use crate::{state::KingdomState, type_writer::TypeWriter, StateUpdate};
use bevy::{
    ecs::system::SystemId,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_common_assets::yaml::YamlAssetPlugin;
use foldhash::HashSet;
use rand::{seq::IteratorRandom, thread_rng, Rng};
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
                (update_character_sprite, character_ui).run_if(in_state(GameState::Main)),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct CharacterAssets {
    #[asset(path = "characters/jeremy.character.yaml")]
    jeremy: Handle<Character>,
    #[asset(path = "characters/merideth.character.yaml")]
    merideth: Handle<Character>,
    #[asset(path = "characters/prince.character.yaml")]
    prince: Handle<Character>,
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
    characters.insert("prince", character_assets.prince.clone());

    let choose_new_character = commands.register_one_shot_system(choose_new_character);
    commands.insert_resource(Characters {
        table: characters,
        current_key: "jeremy",
        choose_new_character,
    });
}

fn choose_new_character(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut characters: ResMut<Characters>,
    mut selected_character: ResMut<SelectedCharacter>,
    mut character_assets: ResMut<Assets<Character>>,
    mut type_writer: ResMut<TypeWriter>,
    prev_sel_sprite: Query<Entity, With<SelectedCharacterSprite>>,
    state: Res<KingdomState>,
) {
    let mut rng = thread_rng();
    let (new_character, new_handle, (request_index, request)) = characters
        .table
        .iter()
        // filter out characters whose requests have all been heard
        .filter_map(|(key, handle)| {
            if *key == characters.current_key {
                return None;
            }

            let character = character_assets.get(handle).unwrap();
            character
                .sample_requests(state.day, &mut rng)
                .map(|r| (*key, handle.clone(), r))
        })
        .choose(&mut thread_rng())
        .expect("Do something when all options are exhausted");

    info!("selecting new character: {:?}", new_character);
    characters.current_key = new_character;

    for entity in prev_sel_sprite.iter() {
        commands.entity(entity).remove::<SelectedCharacterSprite>();
    }

    let sfx = server.load("audio/interface/Wav/Cursor_tones/cursor_style_2.wav");
    *type_writer = TypeWriter::new(request.text.clone(), 0.025, sfx);

    let character = character_assets.get_mut(&new_handle).unwrap();
    character.set_used(state.day, request_index);
    selected_character.0 = new_handle;
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
                            // SpriteBundle {
                            //     visibility: Visibility::Hidden,
                            //     transform: Transform::from_scale(Vec3::splat(0.01)),
                            //     texture,
                            //     ..Default::default()
                            // },
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
struct SelectedCharacterSprite;

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

fn update_character_sprite(
    windows: Query<&Window>,
    mut character_sprite: Query<(&mut Transform, &mut Visibility), With<SelectedCharacterSprite>>,
) {
    let window = windows.single();
    let Ok((mut sprite, mut vis)) = character_sprite.get_single_mut() else {
        return;
    };

    *vis = Visibility::Visible;

    if let Some(world_position) = window.cursor_position() {
        if world_position.y > 100. && world_position.y < 400. {
            sprite.scale = Vec3::splat(1.2);
        } else {
            sprite.scale = Vec3::splat(1.0);
        }
    }
}

#[derive(Debug, Default, Resource)]
pub struct SelectedCharacter(pub Handle<Character>);

#[derive(Debug, Deserialize, Asset, Component, TypePath)]
pub struct Character {
    pub name: String,
    pub class: Class,
    pub sprite_path: String,
    pub requests: Vec<Vec<Request>>,

    #[serde(skip)]
    current_request: Option<usize>,
    #[serde(skip)]
    used_requests: HashMap<usize, HashSet<usize>>,
    #[serde(skip)]
    pub texture: Option<Handle<Image>>,
    #[serde(skip)]
    pub sprite: Option<Entity>,
}

impl Character {
    /// Sample the remaining requests. If at least on is available, it is returned
    /// along with its index.
    pub fn sample_requests(&self, day: usize, rng: &mut impl Rng) -> Option<(usize, &Request)> {
        self.requests.get(day).and_then(|requests| {
            if let Some(used) = self.used_requests.get(&day) {
                requests
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !used.contains(&i))
                    .choose(rng)
            } else {
                requests.iter().enumerate().choose(rng)
            }
        })
    }

    /// Set a request previously sampled with `sample_requests` as the current used request.
    pub fn set_used(&mut self, day: usize, request: usize) {
        self.current_request = Some(request);
        let used = self.used_requests.entry(day).or_default();
        used.insert(request);
    }

    /// Get the current request if any.
    pub fn request(&self, day: usize) -> Option<&Request> {
        self.current_request
            .and_then(|index| self.requests.get(day).map(|r| &r[index]))
    }

    /// Clear the current request selection. This should be called once at the start of every day.
    pub fn clear_request(&mut self) {
        self.current_request = None;
    }
}

#[derive(Debug, Deserialize, Asset, Component, Reflect)]
pub enum Class {
    Peasant,
    Royal,
}

#[derive(Debug, Deserialize, Asset, Component, Reflect, Clone)]
pub struct Request {
    pub text: String,
    pub yes: StateUpdate,
    pub no: StateUpdate,
    #[serde(default)]
    pub response_handlers: Vec<String>,
}
