use super::*;
use avian3d::prelude::RigidBody;
use bevy_third_person_camera::{ThirdPersonCamera, Zoom};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), add_tpv_cam)
        .add_systems(OnExit(Screen::Gameplay), rm_tpv_cam);
}

fn add_tpv_cam(
    cfg: Res<Config>,
    mut commands: Commands,
    mut camera: Query<Entity, With<SceneCamera>>,
    mut tpv_cam: Query<Entity, With<ThirdPersonCamera>>,
) -> Result {
    let Ok(cam) = camera.single_mut() else {
        return Ok(());
    };
    if tpv_cam.single_mut().is_ok() {
        debug!("Tried to add ThirdPersonCamera to an entiry that already has it");
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
            // bounds: vec![Bound::NO_FLIP, Bound::ABOVE_FLOOR],
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: cfg.player.fov.to_radians(),
            ..Default::default()
        }),
    ));

    Ok(())
}

fn rm_tpv_cam(mut commands: Commands, mut camera: Query<Entity, With<ThirdPersonCamera>>) {
    if let Ok(camera) = camera.single_mut() {
        commands
            .entity(camera)
            .remove::<RigidBody>()
            .remove::<ThirdPersonCamera>();
    }
}
