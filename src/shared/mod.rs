//! Types shared across the domain-level plugins (player, camera, scene, screens, game, ui).
//! A primitive belongs here only if several plugins depend on it — otherwise it
//! lives in the plugin that owns it.
use crate::screens::Screen;
use bevy::prelude::*;

mod config;
mod ext_traits;
mod keybinding;
mod settings;
mod states;

pub use config::*;
pub use ext_traits::*;
pub use keybinding::*;
pub use settings::*;
pub use states::*;

pub fn plugin(app: &mut App) {
    app.configure_sets(
        Update,
        (
            AppSystems::UserInput,
            AppSystems::TickTimers,
            AppSystems::ChangeUi,
            AppSystems::PlaySounds,
            AppSystems::PlayAnimations,
            AppSystems::Update,
        )
            .chain(),
    );

    app.add_plugins((settings::plugin, states::plugin));
}

/// Macro to hide the derive trait boilerplate for marker components
#[macro_export]
macro_rules! markers {
  ( $( $name:ident ),* ) => {
        $(
            #[derive(Component, Reflect, Clone, Default)]
            #[reflect(Component)]
            pub struct $name;
        )*
    };
}

/// Same as [`markers!`] but for timer newtype components
#[macro_export]
macro_rules! timers {
  ( $( $name:ident ),* ) => {
        $(
            #[derive(Component, Reflect, Deref, DerefMut, Debug)]
            #[reflect(Component)]
            pub struct $name(pub Timer);
        )*
    };
}

/// High-level groupings of systems for the app in the [`Update`] schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
/// courtesy of janhohenheim
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AppSystems {
    /// User Input
    UserInput,
    /// Tick timers.
    TickTimers,
    /// Change UI.
    ChangeUi,
    /// Play sounds.
    PlaySounds,
    /// Play animations.
    PlayAnimations,
    /// Do everything else (consider splitting this into further variants).
    Update,
}
