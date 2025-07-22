use super::*;
use bevy::core_pipeline::{
    bloom::Bloom,
    tonemapping::{DebandDither, Tonemapping},
};
#[cfg(feature = "top_down")]
use bevy_enhanced_input::prelude::*;
#[cfg(feature = "third_person")]
use bevy_third_person_camera::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(Screen::Title), add_skybox_to_camera);

    #[cfg(feature = "third_person")]
    app.add_systems(OnEnter(Screen::Gameplay), add_tpv_cam)
        .add_systems(OnExit(Screen::Gameplay), rm_tpv_cam)
        .add_observer(toggle_cam_cursor);

    #[cfg(feature = "top_down")]
    app.add_systems(OnEnter(Screen::Gameplay), move_camera_to_top_down)
        .add_observer(camera_mouse_pan)
        .add_observer(camera_zoom);
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        SceneCamera,
        Camera3d::default(),
        Msaa::Sample4,
        IsDefaultUiCamera,
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
#[cfg(feature = "third_person")]
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

#[cfg(feature = "third_person")]
fn rm_tpv_cam(mut commands: Commands, mut camera: Query<Entity, With<SceneCamera>>) {
    if let Ok(camera) = camera.single_mut() {
        commands
            .entity(camera)
            .remove::<RigidBody>()
            .remove::<ThirdPersonCamera>();
    }
}

#[cfg(feature = "third_person")]
fn toggle_cam_cursor(_: Trigger<CamCursorToggle>, mut cam: Query<&mut ThirdPersonCamera>) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };
    cam.cursor_lock_active = !cam.cursor_lock_active;
}

#[cfg(feature = "top_down")]
fn move_camera_to_top_down(cfg: Res<Config>, mut camera: Query<&mut Transform, With<SceneCamera>>) {
    let Ok(mut cam) = camera.single_mut() else {
        return;
    };

    // *cam = Transform::from_xyz(0.0, cfg.player.spawn_pos.1 + 10.0, 0.0)
    //     .looking_at(cfg.player.spawn_pos.into(), Vec3::Y);
    cam.rotate_y(-0.6);
}

/// Pans the camera using mouse drag
#[cfg(feature = "top_down")]
fn camera_mouse_pan(
    on: Trigger<Fired<Pan>>,
    time: Res<Time>,
    cfg: Res<Config>,
    windows: Query<&Window>,
    mut cam: Query<&mut Transform, With<SceneCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };

    let mut movement = Vec3::ZERO;

    // Check screen borders and set movement direction
    if cursor_pos.x <= cfg.camera.edge_margin {
        movement += Vec3::Z; // left edge → move camera left (Z+)
    } else if cursor_pos.x >= window.width() - cfg.camera.edge_margin {
        movement += -Vec3::Z; // right edge → move right (Z-)
    }

    if cursor_pos.y <= cfg.camera.edge_margin {
        movement += -Vec3::X; // bottom edge → move down (X-)
    } else if cursor_pos.y >= window.height() - cfg.camera.edge_margin {
        movement += Vec3::X; // top edge → move up (X+)
    }

    if movement != Vec3::ZERO {
        cam.translation += movement.normalize() * cfg.camera.speed * time.delta_secs();
    }
}

#[cfg(feature = "top_down")]
fn camera_zoom(
    on: Trigger<Fired<ScrollZoom>>,
    cfg: Res<Config>,
    player: Query<&mut Transform, (With<Player>, Without<SceneCamera>)>,
    mut cam: Query<&mut Transform, With<SceneCamera>>,
) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };
    let Ok(player) = player.single() else {
        return;
    };

    info!("scroll: {}", on.value);

    let new_height = (cam.translation.y - on.value.y * cfg.camera.zoom_speed)
        .clamp(cfg.camera.min_height, cfg.camera.max_height);

    cam.translation.y = new_height;
    let (x, z) = (cam.translation.x, cam.translation.z);

    // cam.look_at(Vec3::new(x, player.translation.y, z), Vec3::X);
}
