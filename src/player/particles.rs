use super::*;
use crate::asset_loading::Particles;
use bevy_sprinkles::prelude::Particles3d;

pub fn plugin(_app: &mut App) {
    // app.add_observer(land_particles);
}

#[allow(dead_code)]
fn land_particles(
    on: On<PlayerLanded>,
    particles: Res<Particles>,
    player_actions: Query<&Actions<PlayerInput>>,
    action_q: Query<&Action<Movement>>,
    transforms: Query<&Transform, Without<SceneCamera>>,
    camera: Single<&Transform, With<SceneCamera>>,
    mut commands: Commands,
) {
    let e = on.event_target();
    debug!("land_particles: {e}");

    let Ok(transform) = transforms.get(e) else {
        return;
    };
    let Ok(action_entities) = player_actions.get(e) else {
        return;
    };
    let Some(movement) = action_entities
        .iter()
        .find_map(|action_entity| action_q.get(action_entity).ok())
    else {
        return;
    };

    let movement: Vec2 = *(*movement);
    let input_dir = camera.movement_direction(movement);

    let mut pos = *transform;
    pos.translation.y -= 0.5;
    pos.scale = Vec3::new(50.0, 300.0, 50.0);
    pos.rotation =
        Quat::from_rotation_y(input_dir.x.atan2(input_dir.z) + std::f32::consts::FRAC_PI_2);

    commands.spawn((pos, Particles3d(particles.wind_spin.clone())));
}
