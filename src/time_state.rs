use crate::animation::{AudioVolumeLens, FadeFromBlack, FadeToBlack};
use crate::menu::FONT_PATH;
use crate::music::{MusicEvent, MusicKind};
use crate::state::KingdomState;
use crate::ui::background::{
    setup_background_particles, setup_background_particles_for_dream, BackgroundTownNight,
    CricketAudio, Crowd, CrowdAudio, CRICKET_VOLUME, CROWD_VOLUME,
};
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_tweening::*;
use sickle_ui::ui_commands::UpdateStatesExt;
use std::time::Duration;

pub struct TimeStatePlugin;

impl Plugin for TimeStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TimeState>()
            .add_systems(Startup, startup)
            .add_systems(
                OnEnter(TimeState::Evening),
                (increment_day, enter_night).chain(),
            )
            .add_systems(OnEnter(TimeState::Morning), enter_morning)
            .insert_resource(DayNumberUi::default());
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    commands
        .spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: server.load(FONT_PATH),
                    font_size: 128.0,
                    ..default()
                },
            )
            .with_text_justify(JustifyText::Center)
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Percent(50.),
                left: Val::Percent(41.),
                ..default()
            }),
            NextDayUi,
        ))
        .insert(Visibility::Hidden);
}

fn increment_day(mut state: ResMut<KingdomState>) {
    state.day += 1;
}

#[derive(Component)]
pub struct NextDayUi;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum TimeState {
    Night,
    Evening,
    Morning,
    Day,
    #[default]
    None,
}

fn enter_night(
    mut commands: Commands,
    mut music: EventWriter<MusicEvent>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
    cricket_audio: Query<Entity, With<CricketAudio>>,
) {
    let system = commands.register_one_shot_system(show_night);
    commands.insert_resource(FadeToBlack::new(0.5, 10, 0., system));
    music.send(MusicEvent::FadeOutSecs(5.));
    commands
        .entity(crowd_audio.single())
        .insert(Animator::new(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: CROWD_VOLUME,
                end: 0.,
            },
        )));
    commands
        .entity(cricket_audio.single())
        .insert(Animator::new(Delay::new(Duration::from_secs_f32(3.)).then(
            Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(5.),
                AudioVolumeLens {
                    start: 0.,
                    end: CRICKET_VOLUME,
                },
            ),
        )));
    info!("entering night");
}

pub fn start_in_night(
    mut commands: Commands,
    mut nigth_village_sprite: Query<&mut Visibility, With<BackgroundTownNight>>,
    mut crowds: Query<&mut Visibility, (With<Crowd>, Without<BackgroundTownNight>)>,
    mut crowd_audio: Query<&mut PlaybackSettings, With<CrowdAudio>>,
    mut cricket_audio: Query<&mut PlaybackSettings, (With<CricketAudio>, Without<CrowdAudio>)>,
) {
    cricket_audio.single_mut().volume = Volume::new(CRICKET_VOLUME);
    crowd_audio.single_mut().volume = Volume::new(0.);
    let id = commands.register_one_shot_system(setup_background_particles_for_dream);
    commands.run_system(id);
    *nigth_village_sprite.single_mut() = Visibility::Visible;
    for mut vis in crowds.iter_mut() {
        *vis = Visibility::Hidden;
    }
    commands.next_state(TimeState::Night);
}

fn show_night(
    mut commands: Commands,
    mut nigth_village_sprite: Query<&mut Visibility, With<BackgroundTownNight>>,
    mut crowds: Query<&mut Visibility, (With<Crowd>, Without<BackgroundTownNight>)>,
    mut music: EventWriter<MusicEvent>,
) {
    *nigth_village_sprite.single_mut() = Visibility::Visible;
    for mut vis in crowds.iter_mut() {
        *vis = Visibility::Hidden;
    }
    let id = commands.register_one_shot_system(setup_background_particles_for_dream);
    commands.run_system(id);
    music.send(MusicEvent::FadeInSecs(crate::music::MusicKind::Dream, 3.));
    let system = commands.register_one_shot_system(handle_night);
    commands.insert_resource(FadeFromBlack::new(0.5, 10, 3., system));
}

fn handle_night(mut commands: Commands) {
    info!("entered night");
    commands.next_state(TimeState::Night);
}

#[derive(Resource, Default)]
pub struct DayNumberUi(Option<Timer>);

fn enter_morning(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut music: EventWriter<MusicEvent>,
) {
    info!("enter morning");
    music.send(MusicEvent::FadeOutSecs(5.));
    let system = commands.register_one_shot_system(handle_morning);
    commands.insert_resource(FadeToBlack::new(0.5, 10, 0., system));
    commands.spawn(AudioBundle {
        source: server.load("audio/church_bells.wav"),
        settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(0.5)),
    });
    // music.send(MusicEvent::FadeOutSecs(5.));
}

pub fn handle_morning(
    mut commands: Commands,
    mut next_day_ui: Query<(&mut Visibility, &mut Text), With<NextDayUi>>,
    mut day_number_ui: ResMut<DayNumberUi>,
    state: Res<KingdomState>,
    crowd_audio: Query<Entity, With<CrowdAudio>>,
    cricket_audio: Query<Entity, With<CricketAudio>>,
    mut nigth_village_sprite: Query<
        &mut Visibility,
        (With<BackgroundTownNight>, Without<NextDayUi>),
    >,
    mut crowds: Query<
        &mut Visibility,
        (
            With<Crowd>,
            Without<BackgroundTownNight>,
            Without<NextDayUi>,
        ),
    >,
    mut music: EventWriter<MusicEvent>,
) {
    info!("handle morning");

    if let Ok(cricket_audio) = cricket_audio.get_single() {
        commands
            .entity(cricket_audio)
            .insert(Animator::new(Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(5.),
                AudioVolumeLens {
                    start: CRICKET_VOLUME,
                    end: 0.,
                },
            )));
    }
    commands.entity(crowd_audio.single()).insert(Animator::new(
        Delay::new(Duration::from_secs_f32(3.)).then(Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(5.),
            AudioVolumeLens {
                start: 0.,
                end: CROWD_VOLUME,
            },
        )),
    ));

    *nigth_village_sprite.single_mut() = Visibility::Hidden;
    for mut vis in crowds.iter_mut() {
        *vis = Visibility::Visible;
    }

    let (mut vis, mut text) = next_day_ui.single_mut();
    *vis = Visibility::Visible;
    text.sections[0].value = state.day_name().to_string();

    let id = commands.register_one_shot_system(setup_background_particles);
    commands.run_system(id);

    music.send(MusicEvent::FadeInSecs(MusicKind::Day, 5.));
    let system = commands.register_one_shot_system(enter_day);
    commands.insert_resource(FadeFromBlack::new(0.5, 4, 3., system));
    day_number_ui.0 = None;
}

fn enter_day(
    mut commands: Commands,
    mut next_day_ui: Query<(&mut Visibility, &mut Text), With<NextDayUi>>,
) {
    info!("enter day");

    let (mut vis, _) = next_day_ui.single_mut();
    *vis = Visibility::Hidden;

    commands.next_state(TimeState::Day);
}
