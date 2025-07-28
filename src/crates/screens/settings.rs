//! The settings screen accessible from the title screen.
use super::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), spawn_settings_screen);
}

fn spawn_settings_screen(mut commands: Commands) {
    commands.spawn((StateScoped(Screen::Settings), settings_ui()));
}
