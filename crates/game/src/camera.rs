use super::*;
#[cfg(feature = "third_person")]
use avian3d::prelude::*;
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
    app.add_systems(
        OnEnter(Screen::Gameplay),
        sync_camera_to_player.after(player::spawn_player),
    )
    .add_observer(camera_mouse_pan)
    .add_observer(camera_to_rotate)
    .add_observer(camera_to_move)
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

/// Can be used in Update to make camera follow player at all times
/// TODO: check for player id for each player for split screen
#[cfg(feature = "top_down")]
fn sync_camera_to_player(
    cfg: Res<Config>,
    player: Query<&mut Transform, (With<Player>, Without<SceneCamera>)>,
    mut camera: Query<&mut Transform, (With<SceneCamera>, Without<Player>)>,
) {
    let Ok(mut cam) = camera.single_mut() else {
        return;
    };
    for player in player.iter() {
        let mut new = player.looking_at(Vec3::new(0.0, 0.0, -f32::INFINITY), Vec3::Y);
        new.rotate_x(-0.65);
        let offset = cfg.camera.max_height / 2.0;

        cam.rotation = new.rotation;
        cam.translation = new.translation + Vec3::new(0.0, offset, offset);
    }
}

/// Moves the camera using mouse drag on edges
#[cfg(feature = "top_down")]
fn camera_mouse_pan(
    on: Trigger<Fired<Pan>>,
    time: Res<Time>,
    cfg: Res<Config>,
    state: Res<GameState>,
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

    match state.camera_mode {
        CameraMode::Move => {
            // Check screen borders and set movement direction
            let mut dir = *cam.left();
            dir.y = 0.0;
            let dir = dir.normalize();
            let mut edge_rel_speed = 1.0;
            if cursor_pos.x <= cfg.camera.edge_margin {
                // left edge → move camera left
                edge_rel_speed = cursor_pos.x / cfg.camera.edge_margin;
                movement += dir;
            } else if cursor_pos.x >= window.width() - cfg.camera.edge_margin {
                // right edge → move right
                edge_rel_speed = (window.width() - cursor_pos.x).abs() / cfg.camera.edge_margin;
                movement += -dir;
            }

            let mut dir = *cam.forward();
            dir.y = 0.0;
            let dir = dir.normalize();
            if cursor_pos.y <= cfg.camera.edge_margin {
                // top edge → move up (X+)
                edge_rel_speed = cursor_pos.y / cfg.camera.edge_margin;
                movement += dir;
            } else if cursor_pos.y >= window.height() - cfg.camera.edge_margin {
                // bottom edge → move down (X-)
                edge_rel_speed = (window.height() - cursor_pos.y).abs() / cfg.camera.edge_margin;
                movement += -dir;
            }

            if movement != Vec3::ZERO {
                // just for corner cases to avoid speed being infinite
                edge_rel_speed = edge_rel_speed.max(0.1);
                let speed = cfg.camera.max_speed / edge_rel_speed;
                cam.translation += movement.normalize_or_zero() * speed * time.delta_secs();
            }
        }
        CameraMode::Rotate => {
            let yaw_rot = Quat::from_rotation_y(on.value.x * cfg.camera.rotate_speed);
            cam.rotate(yaw_rot);
        }
    }
}

#[cfg(feature = "top_down")]
fn camera_zoom(
    on: Trigger<Fired<ScrollZoom>>,
    cfg: Res<Config>,
    mut cam: Query<&mut Transform, With<SceneCamera>>,
) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };

    let direction = cam.forward().normalize();
    cam.translation += direction * on.value.y * cfg.camera.zoom_speed;
    // cam.translation.y = cam.translation.y.min(cfg.camera.max_height);
}

#[cfg(feature = "top_down")]
fn camera_to_rotate(_: Trigger<Started<RotateToggle>>, mut state: ResMut<GameState>) {
    state.camera_mode = CameraMode::Rotate;
}
#[cfg(feature = "top_down")]
fn camera_to_move(_: Trigger<Completed<RotateToggle>>, mut state: ResMut<GameState>) {
    state.camera_mode = CameraMode::Move;
}
