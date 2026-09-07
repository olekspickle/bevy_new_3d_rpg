use super::*;
use bevy_ahoy::CharacterLook;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        movement
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::UserInput),
    )
    .add_systems(
        Update,
        tick_jump_timer
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::TickTimers),
    )
    .add_observer(handle_sprint_in)
    .add_observer(handle_sprint_out)
    .add_observer(handle_dash)
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
    on: On<Start<Sprint>>,
    cfg: Res<Config>,
    mut ahoy_q: Query<&mut CharacterController>,
) -> Result {
    let entity = on.context;
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

fn handle_dash(
    on: On<Start<Dash>>,
    navigate: Single<&Action<Movement>>,
    camera: Query<&Transform, With<SceneCamera>>,
    mut player_q: Query<&mut Player>,
    mut commands: Commands,
) -> Result {
    let navigate = **navigate.into_inner();
    for player in player_q.iter_mut() {
        debug!("Dashing player: on-{},id-{}", on.context, player.id);
        let cam_transform = camera.single()?;
        let direction = cam_transform.movement_direction(navigate);
        commands.entity(player.id).insert(Dashing::new(direction));
    }

    // TODO: dash

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

#[derive(Component)]
pub struct Dashing {
    pub start: Instant,
    pub direction: Vec3,
}

impl Dashing {
    fn new(direction: Vec3) -> Self {
        Self {
            start: Instant::now(),
            direction,
        }
    }
}
