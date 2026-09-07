use crate::asset_loading::AudioSources;
use crate::audio::{FadeIn, FadeOut};
use crate::camera::ToggleCamCursor;
use crate::scene::FadeCam;
use crate::screens::Screen;
use crate::shared::Settings;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_seedling::{prelude::*, sample_effects};

#[cfg(feature = "dev_native")]
mod dev_tools;
mod dialogue;
mod mood;

pub use mood::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        mood::plugin,
        #[cfg(feature = "dev_native")]
        dev_tools::plugin,
        dialogue::plugin,
    ))
    .add_observer(pause)
    .add_observer(mute);
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct Pause;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Mute;

#[derive(EntityEvent)]
pub struct TogglePause(pub Entity);
#[derive(EntityEvent)]
pub struct ToggleMute(pub Entity);
#[derive(Event)]
pub struct ToggleDebugUi;

// ================== trigger events on input ========================
fn pause(on: On<Start<Pause>>, mut commands: Commands) {
    commands.entity(on.event_target()).trigger(TogglePause);
}
fn mute(on: On<Start<Mute>>, mut commands: Commands) {
    commands.entity(on.event_target()).trigger(ToggleMute);
}
