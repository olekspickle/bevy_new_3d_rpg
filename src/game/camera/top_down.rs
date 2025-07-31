use super::*;
use bevy_top_down_camera::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), add_td_cam)
        .add_systems(OnExit(Screen::Gameplay), rm_td_cam);
}

fn add_td_cam(
    cfg: Res<Config>,
    mut commands: Commands,
    mut camera: Query<Entity, With<SceneCamera>>,
    mut tpv_cam: Query<Entity, With<TopDownCamera>>,
) -> Result {
    let Ok(cam) = camera.single_mut() else {
        return Ok(());
    };
    if tpv_cam.single_mut().is_ok() {
        debug!("Tried to add TopDownCamera to an entiry that already has it");
        return Ok(());
    }

    commands.entity(cam).insert((
        TopDownCamera {
            zoom_enabled: true,
            zoom: cfg.player.zoom.into(),
            ..default()
        },
        Projection::from(PerspectiveProjection {
            fov: cfg.player.fov.to_radians(),
            ..Default::default()
        }),
    ));

    Ok(())
}

fn rm_td_cam(mut commands: Commands, mut camera: Query<Entity, With<TopDownCamera>>) {
    if let Ok(camera) = camera.single_mut() {
        commands.entity(camera).remove::<TopDownCamera>();
    }
}
