//! Development tools for the game. This plugin is only enabled in dev builds.

use super::*;
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_toggle_active,
    ui::UiDebugOptions,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((EguiPlugin {
        enable_multipass_for_primary_context: true,
    },))
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Backquote)),
        )
        .add_systems(Update, log_transitions::<Screen>)
        .add_observer(toggle_debug_ui);
}

fn toggle_debug_ui(_: Trigger<ToggleDebugUi>, mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
