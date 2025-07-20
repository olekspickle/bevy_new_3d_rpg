use asset_loading::*;
use audio::*;
use bevy::{prelude::*, ui::Val::*};
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;
use models::*;

mod interaction;
mod opts;
mod perf;
mod prefabs;
mod widget;

pub use interaction::*;
pub use opts::*;
pub use prefabs::*;
pub use widget::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((perf::plugin, interaction::plugin, prefabs::plugin));
}
