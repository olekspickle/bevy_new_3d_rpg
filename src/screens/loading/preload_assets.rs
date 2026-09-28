//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on Wasm.

use super::*;
use crate::asset_loading::{
    AudioHandles, AudioSources, ConfigHandle, Models, Particles, ShaderAssets, Textures,
    sync_config,
};
use bevy_asset_loader::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(LoadingScreen::Assets)
            .continue_to_state(LoadingScreen::Shaders)
            .load_collection::<ConfigHandle>()
            .load_collection::<Textures>()
            .load_collection::<Models>()
            .load_collection::<Particles>()
            .load_collection::<ShaderAssets>()
            .load_collection::<AudioHandles>()
            .finally_init_resource::<AudioSources>(),
    );
    app.add_systems(OnEnter(LoadingScreen::Assets), spawn_asset_loading_screen);
    // Config derives Default, so `finally_init_resource` would skip the file; insert it on exit instead.
    app.add_systems(OnExit(LoadingScreen::Assets), sync_config);
}

fn spawn_asset_loading_screen(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(LoadingScreen::Assets),
        widget::ui_root("Loading Screen"),
        BackgroundColor(colors::TRANSLUCENT),
        children![widget::label("Loading assets...")],
    ));
}
