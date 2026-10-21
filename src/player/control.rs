use super::*;
use bevy_ahoy::CharacterLook;
use std::f32::consts::PI;

/// Falling past this height (e.g. through a hole in the level) teleports the player back
/// to the configured spawn point instead of leaving them to fall forever.
const FALL_RESPAWN_Y: f32 = -100.0;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        movement
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::UserInput),
    )
    .add_systems(
        Update,
        (
            tick_jump_timer.in_set(AppSystems::TickTimers),
            respawn_if_fallen.in_set(AppSystems::Update),
        )
            .run_if(in_state(Screen::Gameplay)),
    )
    .add_observer(handle_sprint_in)
    .add_observer(handle_sprint_out)
    .add_observer(face_dash_direction)
    .add_observer(handle_jump)
    .add_observer(handle_land)
    //  .add_observer(handle_attack)
    .add_observer(crouch_in)
    .add_observer(crouch_out);
}

fn movement(
    cfg: Res<Config>,
    movement: Query<&Action<Movement>>,
    camera: Single<&Transform, With<SceneCamera>>,
    mut player_q: Query<(&mut Transform, &mut CharacterLook), Without<SceneCamera>>,
) {
    for movement in movement.iter() {
        let movement = *movement;

        for (mut pos, mut look) in player_q.iter_mut() {
            let input_dir = camera.movement_direction(*movement);

            if input_dir.length_squared() > cfg.player.movement.idle_to_run_threshold {
                // set ahoy KCC direction
                let (yaw, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
                *look = CharacterLook { yaw, pitch };

                // rotate model
                let rotation = Quat::from_rotation_y(input_dir.x.atan2(input_dir.z));
                pos.rotation = pos.rotation.slerp(rotation, 0.2);
            }
        }
    }
}

fn handle_sprint_in(
    on: On<Fire<Sprint>>,
    cfg: Res<Config>,
    navigate: Single<&Action<Movement>>,
    mut ahoy_q: Query<&mut CharacterController>,
) -> Result {
    let entity = on.context;
    // fires every frame while held; reset only happens on movement Complete, so skip idle
    if ***navigate == Vec2::ZERO {
        return Ok(());
    }
    if let Ok(mut ahoy) = ahoy_q.get_mut(entity)
        && ahoy.speed == cfg.player.movement.speed()
    {
        ahoy.speed *= cfg.player.movement.sprint_factor;
        debug!("Sprint started for entity: {entity}");
    }

    Ok(())
}

fn handle_sprint_out(
    on: On<Complete<Movement>>,
    cfg: Res<Config>,
    mut ahoy_q: Query<&mut CharacterController>,
) {
    if let Ok(mut ahoy) = ahoy_q.get_mut(on.context)
        && ahoy.speed > cfg.player.movement.speed()
    {
        ahoy.speed = cfg.player.movement.speed();
    }
}

fn face_dash_direction(
    on: On<Start<Dash>>,
    navigate: Single<&Action<Movement>>,
    camera: Single<&Transform, With<SceneCamera>>,
    mut player_q: Query<(&mut Transform, &mut CharacterLook), Without<SceneCamera>>,
) -> Result {
    let (mut pos, mut look) = player_q.get_mut(on.context)?;
    let input_dir = camera.movement_direction(***navigate);
    if input_dir != Vec3::ZERO {
        // snap so the roll faces where we dash
        pos.rotation = Quat::from_rotation_y(input_dir.x.atan2(input_dir.z));
    } else {
        // without input ahoy dashes along the look, so aim it where the model (+Z) faces
        let (yaw, _, _) = pos.rotation.to_euler(EulerRot::YXZ);
        look.yaw = yaw + PI;
    }
    Ok(())
}

pub fn crouch_in(_: On<Start<Crouch>>, mut player: Query<&mut Collider>) -> Result {
    for mut collider in player.iter_mut() {
        collider.set_scale(Vec3::new(1.0, 0.5, 1.0), 4);
    }

    // TODO: Handle slide

    Ok(())
}

pub fn crouch_out(_: On<Complete<Crouch>>, mut player: Query<&mut Collider>) -> Result {
    for mut collider in player.iter_mut() {
        collider.set_scale(Vec3::ONE, 4);
    }

    // TODO: Handle slide

    Ok(())
}

// fn handle_attack(on: On<Start<Attack>>, mut commands: Commands) {
//     let entity = on.target();
//     // TODO: Hit
// }

/// Start the [`JumpTimer`], it runs until [`PlayerLanded`].
fn handle_jump(on: On<Start<Jump>>, mut player_q: Query<&mut JumpTimer, With<Player>>) -> Result {
    let mut timer = player_q.get_mut(on.context)?;
    timer.reset();
    timer.unpause();

    Ok(())
}

fn handle_land(on: On<PlayerLanded>, mut player_q: Query<&mut JumpTimer, With<Player>>) -> Result {
    let mut timer = player_q.get_mut(on.event_target())?;
    timer.reset();
    timer.pause();

    Ok(())
}

fn tick_jump_timer(time: Res<Time>, mut timers: Query<&mut JumpTimer, With<Player>>) {
    for mut timer in timers.iter_mut() {
        timer.tick(time.delta());
    }
}

fn respawn_if_fallen(
    cfg: Res<Config>,
    mut player_q: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut CharacterControllerState,
        ),
        With<Player>,
    >,
) {
    for (mut transform, mut velocity, mut state) in &mut player_q {
        if transform.translation.y < FALL_RESPAWN_Y {
            transform.translation = Vec3::from(cfg.player.spawn_pos);
            *velocity = LinearVelocity(Vec3::ZERO);
            *state = CharacterControllerState::default();
        }
    }
}
