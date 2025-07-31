// Disable console on Windows for non-dev builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    app::App, asset::AssetMetaCheck, log, prelude::*, window::PrimaryWindow, winit::WinitWindows,
};
use bevy_fix_cursor_unlock_web::prelude::*;
use std::io::Cursor;
use winit::window::Icon;

pub mod asset_loading;
pub mod audio;
pub mod game;
pub mod models;
pub mod player;
pub mod scene;
pub mod screens;
pub mod ui;

use asset_loading::{AudioSources, Models, ResourceHandles, Textures};
use audio::*;
use models::*;
use scene::*;
use ui::*;

fn main() {
    let mut app = App::new();

    app.configure_sets(
        Update,
        (
            AppSystems::TickTimers,
            AppSystems::RecordInput,
            AppSystems::Update,
        )
            .chain(),
    );

    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy Game".to_string(),
            // Bind to canvas included in `index.html` for custom wasm js logic
            // canvas: Some("#bevy".to_owned()),
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5 and Ctrl+R
            prevent_default_event_handling: false,
            ..default()
        }),
        ..default()
    };
    let assets = AssetPlugin {
        meta_check: AssetMetaCheck::Never,
        ..default()
    };
    let log_level = log::LogPlugin {
        level: log::Level::TRACE,
        filter: "info,symphonia=off,naga=off,wgpu=warn".to_string(),
        ..Default::default()
    };

    app.add_plugins(DefaultPlugins.set(window).set(assets).set(log_level));

    // custom plugins. the order is important
    // be sure you use resources/types AFTER you add plugins that insert them
    app.add_plugins((
        FixPointerUnlockPlugin,
        audio::plugin,
        asset_loading::plugin,
        ui::plugin,
        game::plugin,
    ))
    .add_systems(Startup, set_window_icon);

    app.run();
}

/// Sets the icon on windows and X11
/// TODO: fix when bevy gets a normal way of setting window image
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) -> Result {
    let primary_entity = primary_window.single()?;
    let Some(primary) = windows.get_window(primary_entity) else {
        return Ok(());
    };
    let icon_buf = Cursor::new(include_bytes!("../assets/textures/icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };

    Ok(())
}
