use bevy::prelude::*;
use serde::Deserialize;
use sickle_ui::ui_commands::UpdateStatesExt;

use crate::{
    character::{Character, Characters, Request, SelectedCharacter},
    ui::Decision,
    GameState,
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KingdomState {
            heart_size: 50.,
            money: 100.,
            ..Default::default()
        })
        .add_event::<Decision>()
        .add_event::<NewHeartSize>()
        .add_systems(
            PostUpdate,
            (state_ui, update_state, check_end_conditions).run_if(in_state(GameState::Main)),
        );
    }
}

#[derive(Debug, Deserialize, Default, Asset, Resource, Reflect, Clone, Copy)]
#[serde(default)]
pub struct KingdomState {
    pub heart_size: f32,
    pub money: f32,
    pub peasant_happiness: f32,
    pub tax_income: f32,
}

impl KingdomState {
    pub fn apply_request_decision(&mut self, request: &Request, decision: Decision) {
        let result = match decision {
            Decision::Yes => &request.yes,
            Decision::No => &request.no,
        };

        self.heart_size += result.heart_size;
        self.peasant_happiness += result.peasant_happiness;
        self.tax_income += result.tax_income;
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
) {
    if reader.is_empty() {
        return;
    }

    if let Some(character) = characters.get_mut(&selected_character.0) {
        for decision in reader.read() {
            info!(
                "applying decision [{decision:?}] for character [{}]",
                character.name
            );

            state.apply_request_decision(character.request(), *decision);
            writer.send(NewHeartSize(state.heart_size));
            commands.run_system(system.choose_new_character);
        }
    } else {
        if !reader.is_empty() {
            error!("did not handle decision due to unloaded character");
        }
    }
}

fn state_ui(state: Res<KingdomState>, mut state_ui: Query<&mut Text, With<KingdomStateUi>>) {
    if let Ok(mut text) = state_ui.get_single_mut() {
        text.sections[0].value = format!("{:?}", *state);
    }
}

fn check_end_conditions(state: Res<KingdomState>, mut commands: Commands) {
    if state.heart_size < 10. || state.heart_size > 90. {
        commands.next_state(GameState::Loose);
    } else if state.money > 10000. {
        commands.next_state(GameState::Win);
    }
}
