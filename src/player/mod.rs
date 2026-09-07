use crate::asset_loading::{AudioSources, Models};
use crate::camera::SceneCamera;
use crate::screens::{Escape, Screen};
use crate::shared::{AppSystems, Config, EntityExt, GameState, Settings, TransformExt};
#[cfg(feature = "third_person")]
use crate::third_party::ThirdPersonCameraTarget;
#[cfg(feature = "top_down")]
use crate::third_party::TopDownCameraTarget;
use crate::timers;
use crate::ui::modal::{Modal, ModalInput};
use avian3d::prelude::*;
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy_ahoy::{CharacterLook, prelude::*};
use bevy_enhanced_input::prelude::*;
use bevy_seedling::prelude::*;

mod animation;
mod control;
mod input;
mod particles;
mod sound;

pub use animation::*;
pub use control::*;
pub use input::*;

/// Fired on the player entity when it hits the ground after a jump/fall.
#[derive(EntityEvent)]
pub struct PlayerLanded(pub Entity);

timers!(JumpTimer, StepTimer);

/// This plugin handles player related stuff like movement, shooting
/// Player logic is only active during the State `Screen::Playing`
pub fn plugin(app: &mut App) {
    app.add_plugins((
        control::plugin,
        sound::plugin,
        animation::plugin,
        input::plugin,
        particles::plugin,
    ))
    .add_systems(OnEnter(Screen::Gameplay), spawn_player)
    .add_observer(player_post_spawn);
}

pub fn spawn_player(
    cfg: Res<Config>,
    models: Res<Models>,
    gltf_assets: Res<Assets<Gltf>>,
    #[cfg(feature = "fpv")] camera: Single<Entity, With<SceneCamera>>,
    mut commands: Commands,
    // DEBUG
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(gltf) = gltf_assets.get(&models.player) else {
        return;
    };

    let mesh = SceneRoot(gltf.scenes[0].clone());
    let pos = Vec3::from(cfg.player.spawn_pos);
    let pos = Transform::from_translation(pos);
    let hitbox = Capsule3d::new(cfg.player.hitbox.radius, cfg.player.hitbox.height);
    let collider = Collider::from(hitbox);

    let _player = commands
        .spawn((
            DespawnOnExit(Screen::Gameplay),
            pos,
            Player::default(),
            PreviousPosition(pos.translation),
            CharacterController {
                crouch_height: 2.0,
                gravity: cfg.physics.gravity,
                speed: cfg.player.movement.speed(),
                max_speed: cfg.player.movement.max_speed,
                crouch_speed_scale: cfg.player.movement.crouch_factor,
                jump_height: cfg.player.movement.jump_height,
                ..default()
            },
            collider,
            // other player related components
            StepTimer(Timer::from_seconds(cfg.timers.step, TimerMode::Repeating)),
            {
                // started by jump input, stopped on PlayerLanded
                let mut timer = Timer::from_seconds(cfg.timers.jump, TimerMode::Once);
                timer.pause();
                JumpTimer(timer)
            },
        ))
        // spawn character mesh as child to adjust mesh position relative to the player origin
        .with_children(|parent| {
            let mut e = parent.spawn((mesh, Transform::from_xyz(0.0, -1.0, 0.0)));
            e.observe(prepare_animations);

            // DEBUG
            // let collider_mesh = Mesh::from(hitbox);
            // let debug_collider_mesh = Mesh3d(meshes.add(collider_mesh.clone()));
            // let debug_collider_color =
            //     MeshMaterial3d(materials.add(Color::srgba(0.9, 0.1, 0.9, 0.1)));
            // parent.spawn((
            //     debug_collider_mesh,
            //     debug_collider_color,
            //     Transform::from_xyz(0.0, -0.1, 0.0),
            // ));
            // DEBUG
        })
        .id();

    #[cfg(feature = "fpv")]
    commands
        .entity(*camera)
        .insert(CharacterControllerCameraOf::new(_player));
}

fn player_post_spawn(
    on: On<Add, Player>,
    mut players: Query<&mut Player>,
    // mut commands: &mut Commands,
) {
    if let Ok(mut p) = players.get_mut(on.entity) {
        p.id = on.entity; // update player id with spawned entity
        info!("NEW PLAYER ID: {}", p.id);
    }

    // #[cfg(not(feature = "third_person"))]
    // for player in players.iter() {
    //     debug!("triggering cam cursor on player: {}", player.id);
    //     commands.entity(player.id).trigger(ToggleCamCursor);
    // }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
#[require(
    PlayerInput,
    RigidBody::Kinematic,
    Friction::default(),
    Mass(10.0),
    Collider::cylinder(0.7, 1.8),
    CharacterLook::default(),
    Visibility::default() // because we add mesh as child
)]
#[cfg_attr(feature = "third_person", require(ThirdPersonCameraTarget))]
#[cfg_attr(feature = "top_down", require(TopDownCameraTarget))]
pub struct Player {
    /// Will be used in the UI and split screen
    pub id: Entity,
    /// Used for time based effects, like slide dust or magic attacks
    pub last_input_change: Instant,
    pub animation: Animations,
}

/// FIXME: hack because we spawn player entity with complex child hierarchy
/// u32::MAX is Entity::PLACEHOLDER and using placeholder leads to issues and using option
/// here while being idiomatic will unnecessary complicate handling it in systems
/// We replace it with real id when the player entity is spawned anyway
/// It's fine until we have 10 mil entities on spawn I guess
const PLACEHOLDER_ENTITY: Entity = Entity::from_raw_u32(10_000_000).unwrap();

impl Default for Player {
    fn default() -> Self {
        Self {
            id: PLACEHOLDER_ENTITY,
            animation: Animations::default(),
            last_input_change: Instant::now(),
        }
    }
}

#[derive(Component, Default, Deref)]
pub struct PreviousPosition(pub Vec3);
