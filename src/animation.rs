use bevy::{ecs::system::SystemId, prelude::*};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DelayedSpawn::default())
            .add_systems(Startup, startup)
            .add_systems(PostUpdate, update_delayed_spawn)
            .add_systems(Update, (fade_to_black, fade_from_black));
    }
}

fn startup(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1000.0, 1000.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 900.0),
            ..default()
        },
        FadeToBlackSprite,
        SkipRemove,
    ));
}

fn update_delayed_spawn(mut commands: Commands, mut spawn: ResMut<DelayedSpawn>, time: Res<Time>) {
    spawn.update(time.delta_seconds(), &mut commands);
}

#[derive(Resource, Default)]
pub struct DelayedSpawn {
    bundles: Vec<(Box<dyn FnOnce(&mut Commands) + Send + Sync>, f32)>,
}

impl DelayedSpawn {
    pub fn spawn_after(
        &mut self,
        seconds: f32,
        f: impl FnOnce(&mut Commands) + Send + Sync + 'static,
    ) {
        self.bundles.push((Box::new(f), seconds));
    }

    pub fn update(&mut self, delta: f32, commands: &mut Commands) {
        let mut ready_indicies = Vec::new();
        for (i, (_, delay)) in self.bundles.iter_mut().enumerate() {
            *delay -= delta;
            if *delay <= 0.0 {
                ready_indicies.push(i);
            }
        }

        for i in ready_indicies.into_iter().rev() {
            let (spawn, _) = self.bundles.remove(i);
            spawn(commands);
        }
    }
}

pub struct AudioVolumeLens {
    pub start: f32,
    pub end: f32,
}

use bevy_tweening::{Lens, Targetable};

use crate::{ui::UiNode, GameState, SkipRemove};

impl Lens<AudioSink> for AudioVolumeLens {
    fn lerp(&mut self, target: &mut dyn Targetable<AudioSink>, ratio: f32) {
        let volume = self.start + (self.end - self.start) * ratio;
        target.set_volume(volume);
    }
}

#[derive(Component)]
pub struct FadeToBlackSprite;

#[derive(Resource)]
pub struct FadeToBlack {
    system_on_complete: SystemId,
    delay: f32,
    timer_per_step: Timer,
    total_steps: u32,
    steps: u32,
}

impl FadeToBlack {
    pub fn new(secs_per_step: f32, steps: u32, delay: f32, system_on_complete: SystemId) -> Self {
        Self {
            delay,
            system_on_complete,
            timer_per_step: Timer::from_seconds(secs_per_step, TimerMode::Repeating),
            total_steps: steps,
            steps,
        }
    }
}

pub fn set_world_to_black(
    mut ui_images: Query<&mut UiImage, With<UiNode>>,
    mut ui_text: Query<&mut Text, With<UiNode>>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
) {
    for mut image in ui_images.iter_mut() {
        image.color.set_alpha(0.);
    }

    for mut text in ui_text.iter_mut() {
        for section in text.sections.iter_mut() {
            let color = &mut section.style.color;
            color.set_alpha(0.);
        }
    }

    if let Ok(mut sprite) = sprite.get_single_mut() {
        sprite.color.set_alpha(1.);
    }
}

fn fade_to_black(
    mut commands: Commands,
    mut ui_images: Query<&mut UiImage, With<UiNode>>,
    mut ui_text: Query<&mut Text, With<UiNode>>,
    fade_to_black: Option<ResMut<FadeToBlack>>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
    time: Res<Time>,
) {
    if let Some(mut fade) = fade_to_black {
        fade.delay -= time.delta_seconds();
        if fade.delay > 0.0 {
            return;
        }

        fade.timer_per_step.tick(time.delta());
        if fade.timer_per_step.finished() {
            fade.steps = fade.steps.saturating_sub(1);

            for mut image in ui_images.iter_mut() {
                let a = image.color.alpha();
                image.color.set_alpha(a - 1.0 / fade.total_steps as f32);
            }

            for mut text in ui_text.iter_mut() {
                for section in text.sections.iter_mut() {
                    let color = &mut section.style.color;
                    let a = color.alpha();
                    color.set_alpha(a - 1.0 / fade.total_steps as f32);
                }
            }

            let mut sprite = sprite.single_mut();
            let alpha = fade.steps as f32 * (1.0 / fade.total_steps as f32);
            let alpha = 1. - alpha.powi(2);
            sprite.color.set_alpha(alpha);

            if fade.steps == 0 {
                commands.remove_resource::<FadeToBlack>();
                commands.run_system(fade.system_on_complete);
                return;
            }
        }
    }
}

#[derive(Resource)]
pub struct FadeFromBlack {
    system_on_complete: SystemId,
    delay: f32,
    timer_per_step: Timer,
    total_steps: u32,
    steps: u32,
}

impl FadeFromBlack {
    pub fn new(secs_per_step: f32, steps: u32, delay: f32, system_on_complete: SystemId) -> Self {
        Self {
            system_on_complete,
            delay,
            timer_per_step: Timer::from_seconds(secs_per_step, TimerMode::Repeating),
            total_steps: steps,
            steps,
        }
    }
}

fn fade_from_black(
    mut commands: Commands,
    mut ui_images: Query<&mut UiImage, With<UiNode>>,
    mut ui_text: Query<&mut Text, With<UiNode>>,
    fade_from_black: Option<ResMut<FadeFromBlack>>,
    mut sprite: Query<&mut Sprite, With<FadeToBlackSprite>>,
    time: Res<Time>,
) {
    if let Some(mut fade) = fade_from_black {
        fade.delay -= time.delta_seconds();
        if fade.delay > 0.0 {
            return;
        }

        fade.timer_per_step.tick(time.delta());
        if fade.timer_per_step.finished() {
            fade.steps = fade.steps.saturating_sub(1);

            for mut image in ui_images.iter_mut() {
                let a = image.color.alpha();
                image.color.set_alpha(a + 1.0 / fade.total_steps as f32);
            }

            for mut text in ui_text.iter_mut() {
                for section in text.sections.iter_mut() {
                    let color = &mut section.style.color;
                    let a = color.alpha();
                    color.set_alpha(a + 1.0 / fade.total_steps as f32);
                }
            }

            let mut sprite = sprite.single_mut();

            let alpha = fade.steps as f32 * (1.0 / fade.total_steps as f32);
            let alpha = 1. - (1. - alpha).powi(2);
            sprite.color.set_alpha(alpha);

            if fade.steps == 0 {
                commands.remove_resource::<FadeFromBlack>();
                commands.run_system(fade.system_on_complete);
                return;
            }
        }
    }
}
