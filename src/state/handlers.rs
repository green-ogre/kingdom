use super::KingdomState;
use crate::{
    character::{Character, Characters},
    music::play_special_stinger,
    ui::decision::DecisionType,
    GameState,
};
use bevy::{ecs::system::SystemId, prelude::*};
use foldhash::HashMap;

pub struct HandlerPlugin;

impl Plugin for HandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SmithyState::default())
            .insert_resource(NunState::default())
            .insert_resource(PrinceState::default())
            .insert_resource(PrincessState::default())
            .insert_resource(DreamState::default())
            .insert_resource(DuchyState::default())
            .add_systems(
                OnEnter(GameState::Main),
                (ResponseHandlers::insert, Filters::insert),
            );
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
    // dream_transition_to_day,
    dream_summon,
    present_hand,
    conditional_succ,
    succ,
    set_cardiac_dream,
    set_no_choice,
    set_this_gift,
    fine_duchy_handler,
    grasp_handler,
    entertain_handler,
    prosper_handler,
    princess_conscription_handler,
    princess_alliance_handler,
    accord_handler,
    kill_handler
}

macro_rules! set_flag {
    ($name:ident, $ty:ident, $flag:ident) => {
        fn $name(state: Res<KingdomState>, mut item: ResMut<$ty>) {
            match state.last_decision {
                Some(DecisionType::Yes) => {
                    item.$flag = Some(true);
                }
                Some(DecisionType::No) => {
                    item.$flag = Some(false);
                }
                _ => {}
            }
        }
    };
}

#[derive(Debug, Default, Resource)]
pub struct SmithyState {
    granted_strikers: Option<bool>,
}

set_flag!(smithy_strikers, SmithyState, granted_strikers);

#[derive(Debug, Default, Resource)]
pub struct NunState {
    made_paganism_illegal: Option<bool>,
}

set_flag!(nun_paganism, NunState, made_paganism_illegal);

#[derive(Debug, Default, Resource)]
pub struct PrinceState {
    approved_festival: Option<bool>,
    housed_disabled: Option<bool>,
}

set_flag!(prince_festival_handler, PrinceState, approved_festival);
set_flag!(prince_disabled_handler, PrinceState, housed_disabled);

#[derive(Debug, Default, Resource)]
pub struct DuchyState {
    fined_duchy: Option<bool>,
}

set_flag!(fine_duchy_handler, DuchyState, fined_duchy);

#[derive(Debug, Default, Resource)]
pub struct PrincessState {
    lowered_conscription: Option<bool>,
    made_alliance: Option<bool>,
}

set_flag!(
    princess_conscription_handler,
    PrincessState,
    lowered_conscription
);
set_flag!(princess_alliance_handler, PrincessState, made_alliance);

// fn dream_transition_to_day(
//     mut commands: Commands,
//     prev_sel_sprite: Query<(Entity, &Transform), With<SelectedCharacterSprite>>,
//     selected_character: Query<Entity, With<SelectedCharacter>>,
// ) {
//     commands.next_state(TimeState::Morning);
//     commands.entity(selected_character.single()).despawn();
//
//     for (entity, transform) in prev_sel_sprite.iter() {
//         commands.entity(entity).remove::<SelectedCharacterSprite>();
//
//         let slide = Tween::new(
//             EaseFunction::QuadraticInOut,
//             Duration::from_secs_f32(1.5),
//             TransformPositionLens {
//                 start: transform.translation,
//                 end: Vec3::default()
//                     .with_x(-300.)
//                     .with_z(transform.translation.z),
//             },
//         );
//
//         commands.entity(entity).insert(Animator::new(
//             Delay::new(Duration::from_secs_f32(0.5)).then(slide),
//         ));
//     }
// }

/////////////////////////////
// DREAM
/////////////////////////////

#[derive(Debug, Default, Resource)]
pub struct DreamState {
    said_summoned: Option<bool>,
    presented_hand: Option<bool>,
    cardiac_dream: bool,
    no_choice: bool,
    this_gift: bool,
    entertain: bool,
    prosper: bool,
    only: bool,
    more: bool,
    kill_prince: bool,
    kill_princess: bool,
    sanction: bool,
    done: bool,
}

set_flag!(dream_summon, DreamState, said_summoned);
// set_flag!(present_hand, DreamState, presented_hand);
// set_flag!(present_hand, DreamState, presented_hand);
// set_flag!(present_hand, DreamState, presented_hand);

fn present_hand(state: Res<KingdomState>, mut dream: ResMut<DreamState>) {
    match state.last_decision {
        Some(DecisionType::Yes) => {
            dream.this_gift = true;
        }
        Some(DecisionType::No) => {
            dream.no_choice = true;
        }
        _ => {}
    }
}

fn conditional_succ(
    state: Res<KingdomState>,
    mut dream: ResMut<DreamState>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    if matches!(state.last_decision, Some(DecisionType::Yes)) {
        play_special_stinger(&mut commands, &server);
    }
    warn!("Do succing");
}

fn succ(
    state: Res<KingdomState>,
    mut dream: ResMut<DreamState>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    dream.this_gift = true;
    play_special_stinger(&mut commands, &server);
    warn!("Do succing");
}

fn set_cardiac_dream(mut dream: ResMut<DreamState>) {
    dream.cardiac_dream = true;
}

fn set_no_choice(mut dream: ResMut<DreamState>) {
    dream.no_choice = true;
}

fn set_this_gift(mut dream: ResMut<DreamState>) {
    dream.this_gift = true;
}

fn entertain_handler(state: Res<KingdomState>, mut dream: ResMut<DreamState>) {
    match state.last_decision {
        Some(DecisionType::Yes) => {
            dream.prosper = true;
        }
        Some(DecisionType::No) => {
            dream.only = true;
        }
        _ => {}
    }
}

fn grasp_handler(mut dream: ResMut<DreamState>) {
    dream.entertain = true;
}

fn prosper_handler(mut dream: ResMut<DreamState>) {
    dream.more = true;
}

fn accord_handler(
    mut dream: ResMut<DreamState>,
    prince: Res<PrinceState>,
    princess: Res<PrincessState>,
) {
    let total_prince = prince.approved_festival.unwrap_or_default() as u32
        + prince.housed_disabled.unwrap_or_default() as u32;

    let total_princess = princess.lowered_conscription.unwrap_or_default() as u32
        + princess.made_alliance.unwrap_or_default() as u32;

    if total_prince >= total_princess {
        dream.kill_prince = true;
    } else {
        dream.kill_princess = true;
    }
}

fn kill_handler(state: Res<KingdomState>, mut dream: ResMut<DreamState>) {
    match state.last_decision {
        Some(DecisionType::Yes) => {
            dream.done = true;
        }
        Some(DecisionType::No) => {
            dream.sanction = true;
        }
        _ => {}
    }
}

handler_map! {
    /// Request filters.
    ///
    /// These can be used in requests to arbitrarily enable or disabled them.
    Filters,
    prince_festival,
    princess_disabled_filter,
    summon_no,
    summon_yes,
    presented,
    cardiac_dream,
    no_choice,
    this_gift,
    didnt_fine_duchy,
    entertain_filter,
    prosper_filter,
    only_filter,
    more_filter,
    dream_princess_filter,
    dream_prince_filter,
    dream_sanction_filter,
    dream_done_filter
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

macro_rules! filter_by {
    ($name:ident, $key:literal, $ty:ident, |$chara:ident, $state:ident| $cond:expr) => {
        fn $name(
            mut character_assets: ResMut<Assets<Character>>,
            character_data: Res<Characters>,
            $state: Res<$ty>,
        ) {
            let Some($chara) = character_assets.get_mut(&character_data.table[$key]) else {
                return;
            };

            $cond
        }
    };
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

filter_by!(summon_no, "dream-man", DreamState, |ch, state| {
    ch.requests[0][1].availability.filtered = none_or_true(state.said_summoned)
});

filter_by!(summon_yes, "dream-man", DreamState, |ch, state| {
    ch.requests[0][2].availability.filtered = none_or_false(state.said_summoned)
});

filter_by!(presented, "dream-man", DreamState, |ch, state| {
    // TODO: this isn't the right index
    ch.requests[0][2].availability.filtered = none_or_true(state.presented_hand)
});

filter_by!(cardiac_dream, "dream-man", DreamState, |ch, state| {
    ch.requests[0][3].availability.filtered = !state.cardiac_dream;
});

filter_by!(no_choice, "dream-man", DreamState, |ch, state| {
    ch.requests[0][4].availability.filtered = !state.no_choice;
});

filter_by!(this_gift, "dream-man", DreamState, |ch, state| {
    ch.requests[0][5].availability.filtered = !state.this_gift;
});

filter_by!(entertain_filter, "dream-man", DreamState, |ch, state| {
    ch.requests[1][1].availability.filtered = !state.entertain;
});

filter_by!(prosper_filter, "dream-man", DreamState, |ch, state| {
    ch.requests[1][2].availability.filtered = !state.prosper;
});

filter_by!(only_filter, "dream-man", DreamState, |ch, state| {
    ch.requests[1][3].availability.filtered = !state.only;
});

filter_by!(more_filter, "dream-man", DreamState, |ch, state| {
    ch.requests[1][4].availability.filtered = !state.more;
});

filter_by!(dream_prince_filter, "dream-man", DreamState, |ch, state| {
    ch.requests[2][1].availability.filtered = !state.kill_prince;
});

filter_by!(
    dream_princess_filter,
    "dream-man",
    DreamState,
    |ch, state| {
        ch.requests[2][2].availability.filtered = !state.kill_princess;
    }
);

filter_by!(
    dream_sanction_filter,
    "dream-man",
    DreamState,
    |ch, state| {
        ch.requests[2][3].availability.filtered = !state.sanction;
    }
);

filter_by!(dream_done_filter, "dream-man", DreamState, |ch, state| {
    ch.requests[2][4].availability.filtered = !state.done;
});

filter_by!(didnt_fine_duchy, "west-duchess", DuchyState, |ch, state| {
    ch.requests[1][0].availability.filtered = none_or_true(state.fined_duchy);
});
