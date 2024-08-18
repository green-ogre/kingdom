use crate::{
    character::{Character, Characters, Request, SelectedCharacter},
    ui::Decision,
    GameState,
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
                heart_size: 50.,
                wealth: 100.,
                ..Default::default()
            })
            .add_event::<Decision>()
            .add_event::<NewHeartSize>()
            .add_systems(
                PostUpdate,
                // TODO: check_end_conditions or its equivalent should be moved to a schedule _after_
                // this one so one-shot systems have a chance to actually be applied.
                (state_ui, update_state, check_heart_end_conditions)
                    .run_if(in_state(GameState::Main)),
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
    pub last_decision: Option<Decision>,
    pub day: usize,
}

#[derive(Debug, Deserialize, Default, Asset, Resource, Reflect, Clone)]
#[serde(default)]
pub struct StateUpdate {
    pub heart_size: f32,
    pub wealth: f32,
    pub happiness: f32,
    pub can_use_insight: Option<bool>,
}

impl KingdomState {
    pub fn apply_request_decision<'a>(
        &mut self,
        request: &'a Request,
        decision: Decision,
    ) -> &'a StateUpdate {
        let result = match decision {
            Decision::Yes => &request.yes,
            Decision::No => &request.no,
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
        self.happiness + self.wealth
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
    selected_character: Res<SelectedCharacter>,
    mut characters: ResMut<Assets<Character>>,
    system: Res<Characters>,
    response_handlers: Res<handlers::ResponseHandlers>,
    filters: Res<handlers::Filters>,
) {
    if reader.is_empty() {
        return;
    }

    if let Some(character) = characters.get_mut(&selected_character.0) {
        if let Some(decision) = reader.read().last() {
            info!(
                "applying decision [{decision:?}] for character [{}]",
                character.name
            );

            let request = character
                .request(state.day)
                .expect("Character presented with valid request");
            state.apply_request_decision(request, *decision);

            for handler in request.response_handlers.iter() {
                match response_handlers.0.get(handler.as_str()) {
                    Some(id) => commands.run_system(*id),
                    None => {
                        warn!("Attempted to run non-existent handler '{handler}'");
                    }
                }
            }

            writer.send(NewHeartSize(state.heart_size));
        }
    } else {
        if !reader.is_empty() {
            error!("did not handle decision due to unloaded character");
        }
        return;
    }

    // run filters
    filters.run(
        state.day,
        system.table.values().filter_map(|v| characters.get(v)),
        &mut commands,
    );
    commands.run_system(system.choose_new_character);
}

fn state_ui(state: Res<KingdomState>, mut state_ui: Query<&mut Text, With<KingdomStateUi>>) {
    if let Ok(mut text) = state_ui.get_single_mut() {
        text.sections[0].value = format!("{:?}", *state);
    }
}

fn check_heart_end_conditions(state: Res<KingdomState>, mut commands: Commands) {
    if state.heart_size < 10. || state.heart_size > 90. {
        commands.next_state(GameState::Loose);
    } else if state.wealth > 10000. {
        commands.next_state(GameState::Win);
    }
}
