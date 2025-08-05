use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;
use serde::{Deserialize, Serialize};

mod event_dispatch;
mod ext_traits;
mod input;
mod keybinding;
mod palette;
mod player;
mod pre_load;
mod primitives;
mod settings;
mod states;

pub use event_dispatch::*;
pub use ext_traits::*;
pub use input::*;
pub use keybinding::*;
pub use palette::*;
pub use player::*;
pub use pre_load::*;
pub use primitives::*;
pub use settings::*;
pub use states::*;

pub fn plugin(app: &mut App) {
    // register for skein
    app.add_plugins((
        primitives::plugin,
        settings::plugin,
        states::plugin,
        input::plugin,
        event_dispatch::plugin,
    ));
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AppSystems {
    TickTimers,
    RecordInput,
    Update,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Modal {
    Main,
    Settings,
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum SunCycle {
    DayNight,
    Nimbus,
}

impl SunCycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            SunCycle::DayNight => "DayNight",
            SunCycle::Nimbus => "Nimbus",
        }
    }
}
