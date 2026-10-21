use super::*;
use bevy_perf_ui::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        PerfUiPlugin,
        #[cfg(feature = "dev")]
        (
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
            // no-op on wasm and with bevy/dynamic_linking, use `just sysinfo` to see CPU/RAM
            #[cfg(not(target_arch = "wasm32"))]
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
        ),
    ));

    app.add_systems(Startup, setup_perf_ui);
}

fn setup_perf_ui(mut commands: Commands) {
    commands.spawn((
        PerfUiAllEntries::default(),
        PerfUiRoot {
            position: PerfUiPosition::TopRight,
            ..default()
        },
        Pickable::IGNORE,
    ));
}
