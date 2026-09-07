//! The loading screen that appears when the game is starting, but still spawning the level.

use super::*;
use crate::scene::spawn_level;
use bevy::world_serialization::WorldInstance;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(LoadingScreen::Level),
        (spawn_level, spawn_level_loading_screen),
    );
    app.add_systems(
        Update,
        advance_to_title.run_if(in_state(LoadingScreen::Level)),
    );
}

fn spawn_level_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        BackgroundColor(colors::TRANSLUCENT),
        DespawnOnExit(LoadingScreen::Level),
        children![widget::label("Spawning Level...")],
    ));
}

fn advance_to_title(
    mut next_screen: ResMut<NextState<Screen>>,
    world_instance_spawner: Res<WorldInstanceSpawner>,
    world_instances: Query<&WorldInstance>,
    just_added_roots: Query<(), (With<WorldAssetRoot>, Without<WorldInstance>)>,
    just_added_meshes: Query<(), Added<Mesh3d>>,
) {
    if !(just_added_meshes.is_empty() && just_added_roots.is_empty()) {
        return;
    }

    for world_instance in world_instances.iter() {
        if !world_instance_spawner.instance_is_ready(**world_instance) {
            return;
        }
    }
    next_screen.set(Screen::Title);
}
