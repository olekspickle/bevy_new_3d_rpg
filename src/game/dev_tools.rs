//! Development tools for the game. This plugin is only enabled in dev builds.
use super::*;
use bevy::{dev_tools::states::log_transitions, input::common_conditions::input_toggle_active};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default())
        .add_plugins((
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)),
        ))
        .add_systems(Update, log_transitions::<Screen>)
        .add_observer(toggle_debug_ui);
}

fn toggle_debug_ui(_: On<ToggleDebugUi>, mut options: ResMut<GlobalUiDebugOptions>) {
    options.toggle();
}
