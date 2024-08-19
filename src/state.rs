use crate::{
    character::{Character, Characters, Request, SelectedCharacter},
    ui::{Decision, DecisionType},
    CharacterSet, GameState, TimeState,
};
use bevy::prelude::*;
use serde::Deserialize;
use sickle_ui::ui_commands::UpdateStatesExt;

mod handlers;

pub use handlers::initialize_filters;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(handlers::HandlerPlugin)
            .insert_resource(KingdomState {
                heart_size: 3.,
                wealth: 100.,
                ..Default::default()
            })
            .add_event::<Decision>()
            .add_event::<NewHeartSize>()
            .add_event::<EndDay>()
            .add_systems(
                PostUpdate,
                // TODO: check_end_conditions or its equivalent should be moved to a schedule _after_
                // this one so one-shot systems have a chance to actually be applied.
                (state_ui, update_state, check_heart_end_conditions).in_set(CharacterSet),
            );
    }
}

pub const PROSPERITY_THRESHOLDS: [f32; 4] = [10., 20., 30., 40.];

#[derive(Debug, Default, Asset, Resource, Reflect, Clone)]
pub struct KingdomState {
    pub heart_size: f32,
    pub wealth: f32,
    pub happiness: f32,
    pub can_use_insight: bool,
    pub last_decision: Option<DecisionType>,
    pub day: usize,
}

#[derive(Event)]
pub struct EndDay;

#[derive(Debug, Deserialize, Default, Asset, Resource, Reflect, Clone)]
#[serde(default)]
pub struct StateUpdate {
    pub heart_size: f32,
    pub wealth: f32,
    pub happiness: f32,
    pub can_use_insight: Option<bool>,
    pub last_word: Option<String>,
}

impl KingdomState {
    pub fn apply_request_decision<'a>(
        &mut self,
        request: &'a Request,
        decision: DecisionType,
    ) -> &'a StateUpdate {
        let result = match decision {
            DecisionType::Yes => &request.yes,
            DecisionType::No => &request.no,
        };

        self.last_decision = Some(decision);
        self.heart_size += result.heart_size;
        self.happiness += result.happiness;

        if let Some(insight) = result.can_use_insight {
            self.can_use_insight = insight;
        }

        result
    }

    /// Calculate the overall prosperity based on wealth and happiness.
    pub fn prosperity(&self) -> f32 {
        Self::calculate_prosperity(self.happiness, self.wealth)
    }

    pub fn calculate_prosperity(happiness: f32, wealth: f32) -> f32 {
        happiness + wealth
    }

    pub fn day_name(&self) -> &'static str {
        match self.day {
            0 => "Spring",
            1 => "Fall",
            2 => "Winter",
            _ => "Spring",
        }
    }
}

#[derive(Component)]
pub struct KingdomStateUi;

#[derive(Event)]
pub struct NewHeartSize(pub f32);

fn update_state(
    mut commands: Commands,
    mut state: ResMut<KingdomState>,
    mut reader: EventReader<Decision>,
    mut writer: EventWriter<NewHeartSize>,
    mut characters: ResMut<Assets<Character>>,
    system: Res<Characters>,
    response_handlers: Res<handlers::ResponseHandlers>,
    filters: Res<handlers::Filters>,
    // time_state: Res<State<TimeState>>,
) {
    if reader.is_empty() {
        return;
    }

    if let Some(decision) = reader.read().last() {
        let character = match decision {
            Decision::Yes(c) => c,
            Decision::No(c) => c,
        };

        if let Some(character) = characters.get_mut(character) {
            info!(
                "applying decision [{decision:?}] for character [{}]",
                character.name
            );

            let request = character
                .request(state.day)
                .expect("Character presented with valid request");
            state.apply_request_decision(request, decision.into());

            for handler in request.response_handlers.iter() {
                match response_handlers.0.get(handler.as_str()) {
                    Some(id) => {
                        info!("running handler: {}", handler.as_str());
                        commands.run_system(*id);
                    }
                    None => {
                        warn!("Attempted to run non-existent handler '{handler}'");
                    }
                }
            }

            writer.send(NewHeartSize(state.heart_size));
        }
    }

    // run filters
    filters.run(
        state.day,
        system.table.values().filter_map(|v| characters.get(v)),
        &mut commands,
    );

    commands.run_system(system.choose_new_character);
    // match time_state.get() {
    //     TimeState::Day => {
    //         commands.run_system(system.choose_new_character);
    //     }
    //     _ => {}
    // }
}

fn state_ui(state: Res<KingdomState>, mut state_ui: Query<&mut Text, With<KingdomStateUi>>) {
    if let Ok(mut text) = state_ui.get_single_mut() {
        text.sections[0].value = format!("{:?}", *state);
    }
}

fn check_heart_end_conditions(state: Res<KingdomState>, mut commands: Commands) {
    if state.heart_size <= 0. || state.heart_size >= 6. {
        commands.next_state(GameState::Loose);
    } else if state.wealth > 10000. {
        commands.next_state(GameState::Win);
    } else if state.day == 3 {
        commands.next_state(GameState::Win);
    }
}
