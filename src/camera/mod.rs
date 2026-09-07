use crate::markers;
use crate::screens::Screen;
use crate::shared::Config;
#[cfg(feature = "third_person")]
use crate::third_party::ThirdPersonCamera;
use bevy::prelude::*;
#[cfg(not(feature = "third_person"))]
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
#[cfg(feature = "native")]
use bevy::{anti_alias::taa::TemporalAntiAliasing, pbr::ScreenSpaceAmbientOcclusion};
use bevy::{
    camera::Exposure, core_pipeline::tonemapping::Tonemapping, light::ShadowFilteringMethod,
    post_process::bloom::Bloom, render::view::Hdr,
};

// mod gamepad_cursor;
mod hdr;
#[cfg(feature = "third_person")]
mod third_person;
#[cfg(feature = "top_down")]
mod top_down;

pub fn plugin(app: &mut App) {
    app.add_plugins(hdr::plugin)
        // app.add_plugins((hdr::plugin, gamepad_cursor::plugin))
        .add_systems(Startup, spawn_camera)
        .add_observer(on_toggle_cam_cursor);

    #[cfg(feature = "third_person")]
    app.add_plugins(third_person::plugin);
    #[cfg(feature = "top_down")]
    app.add_plugins(top_down::plugin);
}

markers!(SceneCamera);

#[derive(EntityEvent)]
pub struct ToggleCamCursor(pub Entity);

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        SceneCamera,
        IsDefaultUiCamera,
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(100., 50., 100.).looking_at(Vec3::ZERO, Vec3::Y),
        (
            Exposure::BLENDER,
            Tonemapping::TonyMcMapface,
            Bloom::NATURAL,
            Hdr,
        ),
        // performance critical
        (
            Msaa::Off,
            #[cfg(feature = "native")] // breaks wasm
            TemporalAntiAliasing::default(),
            ShadowFilteringMethod::Temporal,
            #[cfg(feature = "native")] // See https://github.com/bPluginevyengine/bevy/issues/20459
            ScreenSpaceAmbientOcclusion::default(),
        ),
    ));
}

// TODO: make it work in a local split screen
fn on_toggle_cam_cursor(
    _: On<ToggleCamCursor>,
    #[cfg(feature = "third_person")] mut cam: Query<&mut ThirdPersonCamera>,
    #[cfg(not(feature = "third_person"))] mut window_q: Query<
        &mut CursorOptions,
        With<PrimaryWindow>,
    >,
) {
    #[cfg(feature = "third_person")]
    if let Ok(mut cam) = cam.single_mut() {
        cam.cursor_lock_active = !cam.cursor_lock_active;
        return;
    };

    #[cfg(not(feature = "third_person"))]
    if let Ok(mut cursor_options) = window_q.single_mut() {
        cursor_options.visible = !cursor_options.visible;
        debug!("cursor will be visible: {}", cursor_options.visible);
        if cursor_options.visible {
            cursor_options.grab_mode = CursorGrabMode::None;
        } else {
            cursor_options.grab_mode = CursorGrabMode::Locked;
        }
    }
}
