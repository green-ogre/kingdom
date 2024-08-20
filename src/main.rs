use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::{PrimaryWindow, WindowResized, WindowResolution},
};
use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState, LoadingState, LoadingStateAppExt,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::prelude::*;
use character::{CharacterAssets, CharacterPlugin};
use menu::MainMenuPlugin;
use pixel_perfect::PixelPerfectPlugin;
use state::{StatePlugin, StateUpdate};
use ui::UiPlugin;

mod animated_sprites;
mod animation;
mod character;
mod end;
mod menu;
mod music;
mod pixel_perfect;
mod state;
mod time_state;
mod type_writer;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Concoeur".into(),
                        resolution: WindowResolution::new(1920., 1080.)
                            .with_scale_factor_override(1.0),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            CharacterPlugin,
            StatePlugin,
            UiPlugin,
            PixelPerfectPlugin,
            // WorldInspectorPlugin::new(),
            MainMenuPlugin,
            AudioPlugin,
            music::MusicPlugin,
            animation::AnimationPlugin,
            end::EndPlugin,
            time_state::TimeStatePlugin,
        ))
        .configure_sets(Update, CharacterSet.run_if(in_state(GameState::Main)))
        .configure_sets(PreUpdate, CharacterSet.run_if(in_state(GameState::Main)))
        .configure_sets(PostUpdate, CharacterSet.run_if(in_state(GameState::Main)))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::MainMenu)
                .load_collection::<CharacterAssets>(),
        )
        .add_systems(Startup, menu::setup_cursor)
        .add_systems(PreUpdate, update_window_scale_factor.before(CharacterSet))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Update, (close_on_escape, animated_sprites::animate_sprites))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    WinScreen,
    Main,
    MainMenu,
    Loose,
    Revolution,
    Win,
}

#[derive(Component)]
pub struct SkipRemove;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
struct CharacterSet;

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

fn update_window_scale_factor(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut resize: EventReader<WindowResized>,
) {
    // for resize in resize.read() {
    //     info!("{resize:?}");
    //     let mut window = windows.single_mut();
    //     window.resolution.set(1920., 1080.);
    // }
}
