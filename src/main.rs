use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::WindowResolution,
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_inspector_egui::{quick::WorldInspectorPlugin, DefaultInspectorConfigPlugin};
use character::{CharacterAssets, CharacterPlugin};
use menu::MainMenuPlugin;
use pixel_perfect::PixelPerfectPlugin;
use state::{KingdomState, StatePlugin, StateUpdate};
use ui::{set_world_to_black, UiPlugin};

mod animated_sprites;
mod character;
mod menu;
mod pixel_perfect;
mod state;
mod type_writer;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Concoeur".into(),
                        resolution: WindowResolution::new(1920., 1080.),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            CharacterPlugin,
            StatePlugin,
            UiPlugin,
            PixelPerfectPlugin,
            WorldInspectorPlugin::new(),
            MainMenuPlugin,
        ))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Main)
                .load_collection::<CharacterAssets>(),
        )
        .add_systems(Startup, menu::setup_cursor)
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(GameState::Main), setup)
        .add_systems(Update, (close_on_escape, animated_sprites::animate_sprites))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    Main,
    MainMenu,
    Loose,
    Win,
}

fn close_on_escape(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.read() {
        if matches!(e, KeyboardInput {
            key_code,
            state,
            ..
        }
            if *key_code == KeyCode::Escape && *state == ButtonState::Pressed
        ) {
            writer.send(AppExit::Success);
        }
    }
}

fn setup(mut commands: Commands) {
    // commands.spawn(Camera2dBundle {
    //     camera: Camera {
    //         clear_color: ClearColorConfig::Custom(Color::linear_rgba(0., 0., 0., 0.)),
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // });
}
