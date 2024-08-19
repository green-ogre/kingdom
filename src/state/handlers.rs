use std::time::Duration;

use super::KingdomState;
use crate::{
    character::{Character, Characters, SelectedCharacter, SelectedCharacterSprite},
    ui::DecisionType,
    TimeState,
};
use bevy::{ecs::system::SystemId, prelude::*};
use bevy_tweening::*;
use foldhash::HashMap;
use lens::TransformPositionLens;
use sickle_ui::ui_commands::UpdateStatesExt;

pub struct HandlerPlugin;

impl Plugin for HandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SmithyState::default())
            .insert_resource(NunState::default())
            .insert_resource(PrinceState::default())
            .add_systems(Startup, (ResponseHandlers::insert, Filters::insert));
    }
}

/// Generate a handler map.
macro_rules! handler_map {
    (
        $(#[$($attrss:tt)*])*
        $name:ident, $($funcs:ident),*
    ) => {
        #[derive(Debug, Resource)]
        pub struct $name(pub(super) HashMap<&'static str, SystemId>);

        impl $name {
            pub fn insert(mut commands: Commands) {

                let handlers = [
                    $(
                        (stringify!($funcs), commands.register_one_shot_system($funcs))
                    ),*
                ];

                let mut map = HashMap::default();
                map.extend(handlers);

                commands.insert_resource($name(map));
            }
        }
    };
}

// TODO: validate that all assets use existing handler names
// at startup in debug mode.
handler_map! {
    /// Response handlers.
    ///
    /// These can be used within requests to produce arbitrary side effects after a response.
    ResponseHandlers,
    smithy_strikers,
    nun_paganism,
    prince_festival_handler,
    prince_disabled_handler,
    dream_transition_to_day
}

#[derive(Debug, Default, Resource)]
pub struct SmithyState {
    granted_strikers: bool,
}

fn smithy_strikers(state: Res<KingdomState>, mut smithy: ResMut<SmithyState>) {
    if matches!(state.last_decision, Some(DecisionType::Yes)) {
        smithy.granted_strikers = true;
    }
}

#[derive(Debug, Default, Resource)]
pub struct NunState {
    made_paganism_illegal: Option<bool>,
}

fn nun_paganism(state: Res<KingdomState>, mut nun: ResMut<NunState>) {
    match state.last_decision {
        Some(DecisionType::Yes) => {
            nun.made_paganism_illegal = Some(true);
        }
        Some(DecisionType::No) => {
            nun.made_paganism_illegal = Some(false);
        }
        _ => {}
    }
}

#[derive(Debug, Default, Resource)]
pub struct PrinceState {
    approved_festival: Option<bool>,
    housed_disabled: Option<bool>,
}

fn prince_festival_handler(state: Res<KingdomState>, mut prince: ResMut<PrinceState>) {
    match state.last_decision {
        Some(DecisionType::Yes) => {
            prince.approved_festival = Some(true);
        }
        Some(DecisionType::No) => {
            prince.approved_festival = Some(false);
        }
        _ => {}
    }
}

fn prince_disabled_handler(state: Res<KingdomState>, mut prince: ResMut<PrinceState>) {
    match state.last_decision {
        Some(DecisionType::Yes) => {
            prince.housed_disabled = Some(true);
        }
        Some(DecisionType::No) => {
            prince.housed_disabled = Some(false);
        }
        _ => {}
    }
}

fn dream_transition_to_day(
    mut commands: Commands,
    prev_sel_sprite: Query<(Entity, &Transform), With<SelectedCharacterSprite>>,
    selected_character: Query<Entity, With<SelectedCharacter>>,
) {
    commands.next_state(TimeState::Morning);
    commands.entity(selected_character.single()).despawn();

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
}

handler_map! {
    /// Request filters.
    ///
    /// These can be used in requests to arbitrarily enable or disabled them.
    Filters,
    prince_festival,
    princess_disabled_filter
}

impl Filters {
    pub fn run<'a>(
        &self,
        day: usize,
        characters: impl Iterator<Item = &'a Character>,
        commands: &mut Commands,
    ) {
        for character in characters {
            for request in character.requests.get(day).iter().flat_map(|d| d.iter()) {
                if request.availability.used {
                    continue;
                }

                if let Some(filter) = request.filter.as_ref() {
                    match self.0.get(&filter.as_str()) {
                        Some(filter) => commands.run_system(*filter),
                        None => {
                            warn!("Attempted to run non-existent filter '{filter}'");
                        }
                    }
                }
            }
        }
    }
}

pub fn initialize_filters(
    mut commands: Commands,
    state: ResMut<KingdomState>,
    character_assets: ResMut<Assets<Character>>,
    character_data: Res<Characters>,
    filters: Res<Filters>,
) {
    filters.run(
        state.day,
        character_data
            .table
            .values()
            .filter_map(|v| character_assets.get(v)),
        &mut commands,
    );
}

pub fn prince_festival(
    mut character_assets: ResMut<Assets<Character>>,
    character_data: Res<Characters>,
    nun: Res<NunState>,
) {
    let Some(prince) = character_assets.get_mut(&character_data.table["prince"]) else {
        return;
    };

    prince.requests[0][0].availability.filtered = none_or_true(nun.made_paganism_illegal);
}

fn none_or_true(value: Option<bool>) -> bool {
    matches!(value, Some(true) | None)
}

fn none_or_false(value: Option<bool>) -> bool {
    matches!(value, Some(false) | None)
}

pub fn princess_disabled_filter(
    mut character_assets: ResMut<Assets<Character>>,
    character_data: Res<Characters>,
    prince: Res<PrinceState>,
) {
    let Some(princess) = character_assets.get_mut(&character_data.table["princess"]) else {
        return;
    };

    princess.requests[0][1].availability.filtered = none_or_false(prince.housed_disabled);
}
