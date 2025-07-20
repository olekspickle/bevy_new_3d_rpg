use super::*;
use bevy::core_pipeline::{
    bloom::Bloom,
    tonemapping::{DebandDither, Tonemapping},
};
#[cfg(feature = "top_down_camera")]
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy_third_person_camera::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(Screen::Title), add_skybox_to_camera)
        .add_systems(OnEnter(Screen::Gameplay), add_tpv_cam)
        .add_systems(OnExit(Screen::Gameplay), rm_tpv_cam)
        .add_observer(toggle_cam_cursor);

    #[cfg(feature = "top_down_camera")]
    app.add_systems(Update, camera_mouse_pan);
}

pub fn spawn_camera(mut commands: Commands, #[cfg(feature = "top_down_camera")] cfg: Res<Config>) {
    commands.spawn((
        SceneCamera,
        Camera3d::default(),
        Msaa::Sample4,
        IsDefaultUiCamera,
        #[cfg(feature = "top_down_camera")]
        Transform::from_xyz(0.0, cfg.player.spawn_pos.1 + 50.0, 0.0)
            .looking_at(cfg.player.spawn_pos.into(), Vec3::X), // X is 'up' in this case
        #[cfg(not(feature = "top_down_camera"))]
        Transform::from_xyz(100., 50., 100.).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            hdr: true,
            ..Default::default()
        },
        (
            Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            Bloom::default(),           // 3. Enable bloom for the camera
            DebandDither::Enabled,      // Optional: bloom causes gradients which cause banding
        ),
    ));
}

// TODO: add top down camera as well
fn add_tpv_cam(
    cfg: Res<Config>,
    mut commands: Commands,
    mut camera: Query<Entity, With<SceneCamera>>,
    mut scene_cam: Query<Entity, With<ThirdPersonCamera>>,
) -> Result {
    let Ok(cam) = camera.single_mut() else {
        return Ok(());
    };
    if scene_cam.single_mut().is_ok() {
        debug!("ThirdPersonCamera already exist");
        return Ok(());
    }

    commands.entity(cam).insert((
        ThirdPersonCamera {
            // aim_speed: 3.0,
            // aim_zoom: 0.7,
            // aim_enabled: true,
            zoom_enabled: true,
            zoom: Zoom::new(cfg.player.zoom.0, cfg.player.zoom.1),
            offset_enabled: true,
            offset_toggle_enabled: true,
            cursor_lock_key: KeyCode::KeyL,
            gamepad_settings: CustomGamepadSettings::default(),
            // bounds: vec![Bound::NO_FLIP, Bound::ABOVE_FLOOR],
            ..default()
        },
        RigidBody::Kinematic,
        Collider::sphere(1.0),
        Projection::from(PerspectiveProjection {
            fov: cfg.player.fov.to_radians(),
            ..Default::default()
        }),
    ));

    Ok(())
}

fn rm_tpv_cam(mut commands: Commands, mut camera: Query<Entity, With<SceneCamera>>) {
    if let Ok(camera) = camera.single_mut() {
        commands
            .entity(camera)
            .remove::<RigidBody>()
            .remove::<ThirdPersonCamera>();
    }
}

fn toggle_cam_cursor(_: Trigger<CamCursorToggle>, mut cam: Query<&mut ThirdPersonCamera>) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };
    cam.cursor_lock_active = !cam.cursor_lock_active;
}

/// Pans the camera using right mouse drag
#[cfg(feature = "top_down_camera")]
fn camera_mouse_pan(
    on: Trigger<Fired<Pan>>,
    config: Res<Config>,
    mut query: Query<&mut Transform, With<SceneCamera>>,
) {
    // let mut delta = Vec2::ZERO;
    //
    // delta += on.delta;
    //
    // if delta == Vec2::ZERO {
    //     return;
    // }
    //
    // let mut transform = query.single_mut();
    //
    // // Move in XZ plane, based on mouse drag direction
    // let right = Vec3::Z * delta.x * config.camera.speed;
    // let forward = Vec3::X * -delta.y * config.camera.speed;
    //
    // transform.translation += right + forward;
}
