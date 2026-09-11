use super::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), spawn_settings_screen);
}

fn spawn_settings_screen(mut commands: Commands, settings: Res<Settings>, cfg: Res<Config>) {
    commands.spawn((
        DespawnOnExit(Screen::Settings),
        settings_ui(&settings, &cfg),
    ));
}
