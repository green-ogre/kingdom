use std::time::Duration;

use crate::pixel_perfect::PIXEL_PERFECT_LAYER;
use crate::ui::{set_world_to_black, ActiveMask};
use crate::{state::KingdomState, type_writer::TypeWriter, StateUpdate};
use crate::{type_writer, GameState};
use bevy::math::VectorSpace;
use bevy::{
    ecs::system::SystemId,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    utils::HashMap,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_common_assets::yaml::YamlAssetPlugin;
use bevy_tweening::lens::TransformPositionLens;
use bevy_tweening::{
    Animator, Delay, EaseFunction, EaseMethod, RepeatCount, RepeatStrategy, Tween, TweenCompleted,
};
use rand::{seq::IteratorRandom, thread_rng, Rng};
use serde::Deserialize;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(YamlAssetPlugin::<Character>::new(&["character.yaml"]))
            .insert_resource(TypeWriter::default())
            // .insert_resource(SelectedCharacter::default())
            .add_systems(
                OnEnter(GameState::Main),
                (
                    load_characters,
                    crate::state::initialize_filters,
                    choose_new_character,
                )
                    .chain(),
            )
            .add_systems(PreUpdate, (load_character_sprite, hide_characters))
            .add_systems(
                Update,
                (update_character_sprite, character_ui, handle_slide_intro)
                    .run_if(in_state(GameState::Main)),
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
    #[asset(path = "characters/princess.character.yaml")]
    princess: Handle<Character>,
    #[asset(path = "characters/blacksmith.character.yaml")]
    blacksmith: Handle<Character>,
    #[asset(path = "characters/tax-man.character.yaml")]
    tax_man: Handle<Character>,
    #[asset(path = "characters/village-leader.character.yaml")]
    village_leader: Handle<Character>,
    #[asset(path = "characters/baker.character.yaml")]
    baker: Handle<Character>,
    #[asset(path = "characters/west-duchess.character.yaml")]
    west_duchess: Handle<Character>,
    #[asset(path = "characters/nun.character.yaml")]
    nun: Handle<Character>,
}

#[derive(Debug, Resource)]
pub struct Characters {
    pub table: HashMap<&'static str, Handle<Character>>,
    pub current_key: &'static str,
    pub choose_new_character: SystemId,
}

fn manage_game_state(mut commands: Commands) {}

fn load_characters(mut commands: Commands, character_assets: Res<CharacterAssets>) {
    let mut characters = HashMap::default();

    characters.extend([
        // ("jeremy", character_assets.jeremy.clone()),
        // ("merideth", character_assets.merideth.clone()),
        // ("prince", character_assets.prince.clone()),
        // ("princess", character_assets.princess.clone()),
        ("blacksmith", character_assets.blacksmith.clone()),
        // ("tax-man", character_assets.tax_man.clone()),
        ("village-leader", character_assets.village_leader.clone()),
        ("baker", character_assets.baker.clone()),
        ("west-duchess", character_assets.west_duchess.clone()),
        ("nun", character_assets.nun.clone()),
    ]);

    let choose_new_character = commands.register_one_shot_system(choose_new_character);
    commands.insert_resource(Characters {
        table: characters,
        current_key: "jeremy",
        choose_new_character,
    });
}

#[derive(Component)]
struct SlidingIntro;

fn choose_new_character(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut characters: ResMut<Characters>,
    mut character_assets: ResMut<Assets<Character>>,
    mut type_writer: ResMut<TypeWriter>,
    prev_sel_sprite: Query<(Entity, &Transform), With<SelectedCharacterSprite>>,
    state: Res<KingdomState>,
    sprites: Query<&Transform, With<CharacterSprite>>,
    mut selected_character: Query<(Entity, &mut SelectedCharacter)>,
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

    for (entity, transform) in prev_sel_sprite.iter() {
        commands.entity(entity).remove::<SelectedCharacterSprite>();

        let slide = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs_f32(1.5),
            TransformPositionLens {
                start: transform.translation,
                end: Vec3::default()
                    .with_x(-300.)
                    .with_z(transform.translation.z),
            },
        );

        commands.entity(entity).insert(Animator::new(
            Delay::new(Duration::from_secs_f32(0.5)).then(slide),
        ));
    }

    let sfx = server.load("audio/interface/Wav/Cursor_tones/cursor_style_2.wav");
    *type_writer = TypeWriter::new(request.text.clone(), 0.025, sfx);

    let character = character_assets.get_mut(&new_handle).unwrap();
    character.set_used(state.day, request_index);

    if let Some(entities) = character.sprite {
        for entity in entities.iter() {
            if let Ok(sprite) = sprites.get(*entity) {
                let slide = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(1.5),
                    TransformPositionLens {
                        start: Vec3::default().with_x(300.).with_z(sprite.translation.z),
                        end: Vec3::ZERO.with_z(sprite.translation.z),
                    },
                );

                commands.entity(*entity).insert((
                    SelectedCharacterSprite,
                    Animator::new(
                        slide.then(
                            Delay::new(Duration::from_secs_f32(0.5))
                                .with_completed_event(FINISHED_SLIDE),
                        ),
                    ),
                ));
            }
        }
    }

    if let Ok((entity, mut selected_character)) = selected_character.get_single_mut() {
        selected_character.0 = new_handle;
        commands.entity(entity).insert(SlidingIntro);
    } else {
        commands.spawn((SelectedCharacter(new_handle), SlidingIntro));
    }
}

const FINISHED_SLIDE: u64 = 0xff3;

fn handle_slide_intro(
    mut commands: Commands,
    selected_character: Query<Entity, (With<SelectedCharacter>, With<SlidingIntro>)>,
    mut completed_tweens: EventReader<TweenCompleted>,
) {
    for completion in completed_tweens.read() {
        if completion.user_data == FINISHED_SLIDE {
            if let Ok(selected_character) = selected_character.get_single() {
                println!("finished slide");
                commands.entity(selected_character).remove::<SlidingIntro>();
            }
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
                let head_texture =
                    server.load(format!("{}_head.png", character.sprite_path.trim()));
                let body_texture =
                    server.load(format!("{}_body.png", character.sprite_path.trim()));

                character.sprite = Some([
                    commands
                        .spawn((
                            SpriteBundle {
                                // visibility: Visibility::Hidden,
                                transform: Transform::from_translation(
                                    Vec3::default().with_z(1.).with_x(300.),
                                ),
                                texture: head_texture,
                                ..Default::default()
                            },
                            CharacterSprite::Head,
                            PIXEL_PERFECT_LAYER,
                        ))
                        .id(),
                    commands
                        .spawn((
                            SpriteBundle {
                                // visibility: Visibility::Hidden,
                                transform: Transform::from_xyz(300., 0., 0.),
                                texture: body_texture,
                                ..Default::default()
                            },
                            CharacterSprite::Body,
                            PIXEL_PERFECT_LAYER,
                        ))
                        .id(),
                ]);
            }
            _ => {}
        }
    }
}

#[derive(Component)]
enum CharacterSprite {
    Head,
    Body,
}

#[derive(Component)]
struct SelectedCharacterSprite;

#[derive(Component)]
pub enum CharacterUi {
    Name,
    Request,
}

#[derive(Component)]
pub struct TalkingCharacter;

fn character_ui(
    mut commands: Commands,
    selected_character: Query<&SelectedCharacter, Without<SlidingIntro>>,
    mut sprites: Query<
        (
            Entity,
            &CharacterSprite,
            &mut Transform,
            Has<TalkingCharacter>,
        ),
        With<SelectedCharacterSprite>,
    >,
    characters: Res<Assets<Character>>,
    mut character_ui: Query<(&mut Text, &CharacterUi)>,
    mut type_writer: ResMut<TypeWriter>,
    mut reader: EventReader<KeyboardInput>,
    time: Res<Time>,
) {
    let Ok(selected_character) = selected_character.get_single() else {
        commands.remove_resource::<ActiveMask>();
        for (mut text, _) in character_ui.iter_mut() {
            text.sections[0].style.color.set_alpha(0.);
        }

        return;
    };

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
                    // if selected_character.is_changed() {
                    //     text.sections[0].value = format!("Character: {:?}", character.name);
                    // }
                }
                CharacterUi::Request => {
                    text.sections[0].style.color.set_alpha(1.);
                    text.sections[0].value = type_writer.slice_with_line_wrap().into();

                    if !type_writer.is_finished {
                        for (sprite, ty, transform, is_talking) in sprites.iter() {
                            if is_talking {
                                continue;
                            }

                            match ty {
                                CharacterSprite::Head => {
                                    let talking_tween = Tween::new(
                                        EaseMethod::Linear,
                                        Duration::from_secs_f32(0.3),
                                        TransformPositionLens {
                                            start: Vec3::ZERO.with_z(transform.translation.z),
                                            end: Vec3::ZERO
                                                .with_z(transform.translation.z)
                                                .with_y(2.),
                                        },
                                    )
                                    .with_repeat_count(RepeatCount::Infinite)
                                    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

                                    info!("inserting talking animation");

                                    commands
                                        .entity(sprite)
                                        .insert((TalkingCharacter, Animator::new(talking_tween)));
                                }
                                _ => {}
                            }
                        }
                    } else {
                        for (sprite, ty, mut transform, is_talking) in sprites.iter_mut() {
                            if !is_talking {
                                continue;
                            }

                            match ty {
                                CharacterSprite::Head => {
                                    transform.translation.y = 0.;

                                    info!("removing talking animation");

                                    commands
                                        .entity(sprite)
                                        .remove::<(TalkingCharacter, Animator<Transform>)>();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn hide_characters(mut character_sprites: Query<&mut Visibility, With<CharacterSprite>>) {
    for mut vis in character_sprites.iter_mut() {
        // *vis = Visibility::Hidden;
    }
}

fn update_character_sprite(
    windows: Query<&Window>,
    mut character_sprite: Query<&mut Visibility, With<SelectedCharacterSprite>>,
    mut talking_sprite: Query<&mut Transform, With<TalkingCharacter>>,
) {
    let window = windows.single();

    // for mut vis in character_sprite.iter_mut() {
    //     *vis = Visibility::Visible;
    // }

    // for mut transform in talking_sprite.iter_mut() {
    //     transform.translation.y += 0.01;
    // }

    // if let Some(world_position) = window.cursor_position() {
    //     if world_position.y > 100. && world_position.y < 400. {
    //         sprite.scale = Vec3::splat(1.2);
    //     } else {
    //         sprite.scale = Vec3::splat(1.0);
    //     }
    // }
}

#[derive(Debug, Default, Component)]
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
    pub sprite: Option<[Entity; 2]>,
}

impl Character {
    /// Sample the remaining requests. If at least on is available, it is returned
    /// along with its index.
    pub fn sample_requests(&self, day: usize, rng: &mut impl Rng) -> Option<(usize, &Request)> {
        self.requests.get(day).and_then(|requests| {
            requests
                .iter()
                .enumerate()
                .filter(|(_, r)| r.availability.is_available())
                .choose(rng)
        })
    }

    /// Set a request previously sampled with `sample_requests` as the current used request.
    pub fn set_used(&mut self, day: usize, request: usize) {
        self.current_request = Some(request);
        self.requests[day][request].availability.used = true;
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
    Craftsman,
    Artist,
    Merchant,
    Priest,
    Lord,
    Royal,
}

#[derive(Debug, Default, Deserialize, Component, Reflect, Clone)]
pub struct RequestAvailability {
    pub filtered: bool,
    pub used: bool,
}

impl RequestAvailability {
    pub fn is_available(&self) -> bool {
        !(self.filtered || self.used)
    }
}

#[derive(Debug, Deserialize, Asset, Component, Reflect, Clone)]
pub struct Request {
    pub text: String,
    pub yes: StateUpdate,
    pub no: StateUpdate,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub response_handlers: Vec<String>,
    #[serde(default)]
    pub availability: RequestAvailability,
}
