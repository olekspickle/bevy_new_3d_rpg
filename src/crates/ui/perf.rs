use super::*;
use iyes_perf_ui::{
    PerfUiPlugin,
    entries::{PerfUiFramerateEntries, PerfUiWindowEntries},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_plugins((
        PerfUiPlugin,
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        // https://github.com/IyesGames/iyes_perf_ui/issues/30
        // bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        bevy::render::diagnostic::RenderDiagnosticsPlugin,
    ));

    app.add_systems(Startup, setup_perf_ui);
}

fn setup_perf_ui(mut commands: Commands) {
    commands.spawn((
        PerfUi,
        PerfUiRoot {
            position: PerfUiPosition::TopRight,
            ..default()
        },
        // Contains everything related to FPS and frame time
        PerfUiFramerateEntries::default(),
        // Contains everything related to the window and cursor
        PerfUiWindowEntries::default(),
        // Contains everything related to system diagnostics (CPU, RAM)
        // PerfUiSystemEntries::default(),
    ));
}
