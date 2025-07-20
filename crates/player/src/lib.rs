use asset_loading::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_enhanced_input::prelude::*;
use bevy_third_person_camera::*;
use bevy_tnua::prelude::*;
use bevy_tnua::{TnuaAnimatingState, control_helpers::TnuaSimpleAirActionsCounter};
use bevy_tnua_avian3d::*;
use models::*;
use std::{f32::consts::PI, time::Duration};

mod animation;
mod control;
mod sound;

pub use animation::*;

pub const IDLE_TO_RUN_TRESHOLD: f32 = 0.01;

/// This plugin handles player related stuff like movement, shooting
/// Player logic is only active during the State `Screen::Playing`
pub fn plugin(app: &mut App) {
    app.add_plugins((
        ThirdPersonCameraPlugin,
        TnuaControllerPlugin::new(FixedUpdate),
        TnuaAvian3dPlugin::new(FixedUpdate),
        control::plugin,
        sound::plugin,
    ));

    app.configure_sets(PostUpdate, CameraSyncSet.after(PhysicsSet::Sync))
        .add_systems(OnEnter(Screen::Gameplay), spawn_player)
        .add_systems(
            Update,
            animating
                .in_set(TnuaUserControlsSystemSet)
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_observer(player_post_spawn);
}

pub fn spawn_player(
    cfg: Res<Config>,
    models: Res<Models>,
    gltf_assets: Res<Assets<Gltf>>,
    mut commands: Commands,
    // DEBUG
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    let Some(gltf) = gltf_assets.get(&models.player) else {
        return Ok(());
    };

    let player_rot = Quat::from_rotation_y(PI);
    let mesh = SceneRoot(gltf.scenes[0].clone());
    let pos = Vec3::from(cfg.player.spawn_pos);
    let pos = Transform::from_translation(pos).with_rotation(player_rot);
    let player = Player {
        id: Entity::PLACEHOLDER,
        speed: cfg.player.movement.speed,
        animation_state: AnimationState::StandIdle,
        ..default()
    };
    let collider = Collider::capsule(cfg.player.hitbox.radius, cfg.player.hitbox.height);

    commands
        .spawn((
            StateScoped(Screen::Gameplay),
            pos,
            player,
            ThirdPersonCameraTarget,
            // input context
            (
                GameplayCtx,
                CurrentCtx(Context::Gameplay),
                Actions::<GameplayCtx>::default(),
            ),
            // tnua stuff
            (
                TnuaController::default(),
                // Tnua can fix the rotation, but the character will still get rotated before it can do so.
                // By locking the rotation we can prevent this.
                LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
                TnuaAnimatingState::<AnimationState>::default(),
                TnuaSimpleAirActionsCounter::default(),
                // A sensor shape is not strictly necessary, but without it we'll get weird results.
                TnuaAvian3dSensorShape(collider.clone()),
            ),
            // physics
            (
                collider,
                RigidBody::Dynamic,
                Friction::ZERO.with_combine_rule(CoefficientCombine::Multiply),
            ),
            // other player related components
            (
                JumpTimer(Timer::from_seconds(cfg.timers.jump, TimerMode::Repeating)),
                StepTimer(Timer::from_seconds(cfg.timers.step, TimerMode::Repeating)),
                InheritedVisibility::default(), // silence the warning because of adding SceneRoot as a child
            ),
        ))
        // spawn character mesh as child to adjust mesh position relative to the player origin
        .with_children(|parent| {
            let mut e = parent.spawn((Transform::from_xyz(0.0, -1.5, 0.0), mesh));
            e.observe(prepare_animations);

            // DEBUG
            // let collider_mesh = Mesh::from(Capsule3d::new(
            //     cfg.player.hitbox.radius,
            //     cfg.player.hitbox.height,
            // ));
            // let debug_collider_mesh = Mesh3d(meshes.add(collider_mesh.clone()));
            // let debug_collider_color: MeshMaterial3d<StandardMaterial> =
            //     MeshMaterial3d(materials.add(Color::srgba(0.9, 0.9, 0.9, 0.1)));
            // parent.spawn((
            //     debug_collider_mesh,
            //     debug_collider_color,
            //     Transform::from_xyz(0.0, -0.1, 0.0),
            // ));
            // DEBUG
        })
        .observe(player_post_spawn);

    Ok(())
}

fn player_post_spawn(
    on: Trigger<OnAdd, Player>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    let player = on.target();
    if let Ok(mut p) = players.get_mut(player) {
        p.id = player; // update player id with spawned entity
    }
    commands.trigger(SwitchInputCtx::new(player, Context::Gameplay));
    // We need global context to handle input on main menu, when no players spawned yet
    // but we have to reset it manually when one is spawned
    commands.trigger(SwitchInputCtx::from_context(Context::Gameplay));
}
