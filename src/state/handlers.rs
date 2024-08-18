use super::KingdomState;
use crate::{
    character::{Character, Characters},
    ui::{Decision, DecisionType},
};
use bevy::{ecs::system::SystemId, prelude::*};
use foldhash::HashMap;

pub struct HandlerPlugin;

impl Plugin for HandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SmithyState::default())
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
    test2
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

fn test2(time: Res<Time>) {
    println!("Hello, world2! {}", time.delta_seconds());
}

handler_map! {
    /// Request filters.
    ///
    /// These can be used in requests to arbitrarily enable or disabled them.
    Filters,
    prince_one
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

pub fn prince_one(
    mut character_assets: ResMut<Assets<Character>>,
    character_data: Res<Characters>,
) {
    let Some(prince) = character_assets.get_mut(&character_data.table["prince"]) else {
        return;
    };

    let second_available = !prince.requests[0][0].availability.used;
    prince.requests[0][1].availability.filtered = second_available;
}
