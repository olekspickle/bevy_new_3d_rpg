use crate::asset_loading::AudioSources;
use crate::camera::ToggleCamCursor;
use crate::game::{ToggleDebugUi, TogglePause};
use crate::markers;
use crate::scene::SunCycle;
use crate::screens::{Escape, GoTo, Screen};
use crate::shared::{Config, EntityExt, GameState, SETTINGS_PATH, Settings};
use bevy::prelude::*;
use bevy::{ecs::spawn::SpawnRelated, ui::Val::*, ui_widgets::Button};
use bevy_seedling::prelude::*;

mod constants;
mod interaction;
pub mod modal;

#[cfg(feature = "dev")]
mod perf;
mod prefabs;
mod props;
pub mod widget;

pub use constants::*;
pub use modal::*;
pub use prefabs::*;
pub use props::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((prefabs::plugin, interaction::plugin));

    #[cfg(feature = "dev")]
    app.add_plugins(perf::plugin);
}
