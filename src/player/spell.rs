//! Casts a simple magic bolt: `CastSpell` plays the Enter→Shoot→Exit animation sequence, and
//! the instant the Shoot node starts (see [`SpellShootFrame`]) a `CosmicSphere`-shaded
//! projectile launches from the player's hand bone along the camera's forward direction.
use super::*;
use crate::scene::{
    CosmicSphere, CosmicSphereExtension, CosmicSphereMaterial, CosmicSphereUniforms,
};
use bevy::material::OpaqueRendererMethod;

mod knobs {
    pub const PROJECTILE_SPEED: f32 = 40.0;
    pub const PROJECTILE_START_RADIUS: f32 = 0.3;
    pub const PROJECTILE_LIFETIME_SECS: f32 = 3.0;
    pub const FACING_TURN_RATE: f32 = 0.2;
}

pub fn plugin(app: &mut App) {
    app.add_observer(handle_cast_spell)
        .add_observer(spawn_spell_projectile)
        .add_systems(
            Update,
            (face_shot_direction, fly_and_expire_projectiles).run_if(in_state(Screen::Gameplay)),
        );
}

/// Set on cast; [`turn_toward_facing_target`] slerps the player toward this each frame.
#[derive(Component)]
struct FacingTarget(Quat);

fn handle_cast_spell(
    on: On<Start<CastSpell>>,
    camera: Single<&Transform, With<SceneCamera>>,
    mut player_q: Query<&mut Player>,
    mut commands: Commands,
) -> Result {
    let mut player = player_q.get_mut(on.context)?;
    player.animation.cast_spell();

    let forward = camera.forward();
    let direction = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    if direction != Vec3::ZERO {
        let target = Quat::from_rotation_y(direction.x.atan2(direction.z));
        commands.entity(on.context).insert(FacingTarget(target));
    }

    Ok(())
}

fn face_shot_direction(
    mut commands: Commands,
    mut targets: Query<(Entity, &mut Transform, &FacingTarget)>,
) {
    for (entity, mut transform, facing) in &mut targets {
        transform.rotation = transform.rotation.slerp(facing.0, knobs::FACING_TURN_RATE);
        if transform.rotation.angle_between(facing.0) < 0.01 {
            commands.entity(entity).remove::<FacingTarget>();
        }
    }
}

timers!(ProjectileLifetime);

#[derive(Component)]
struct SpellProjectile {
    velocity: Vec3,
}

fn spawn_spell_projectile(
    on: On<SpellShootFrame>,
    players: Query<&Player>,
    transforms: Query<&GlobalTransform>,
    camera: Single<&Transform, With<SceneCamera>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CosmicSphereMaterial>>,
) -> Result {
    let player = players.get(on.event_target())?;
    let origin = transforms.get(player.hand)?.translation();
    let direction = *camera.forward();

    let material = materials.add(CosmicSphereMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            cull_mode: None,
            unlit: true,
            opaque_render_method: OpaqueRendererMethod::Auto,
            ..default()
        },
        extension: CosmicSphereExtension {
            uniforms: CosmicSphereUniforms::default(),
        },
    });

    commands.spawn((
        Name::new("Spell Projectile"),
        CosmicSphere,
        Mesh3d(meshes.add(Sphere::new(knobs::PROJECTILE_START_RADIUS))),
        MeshMaterial3d(material),
        Transform::from_translation(origin),
        SpellProjectile {
            velocity: direction * knobs::PROJECTILE_SPEED,
        },
        ProjectileLifetime(Timer::from_seconds(
            knobs::PROJECTILE_LIFETIME_SECS,
            TimerMode::Once,
        )),
        DespawnOnExit(Screen::Gameplay),
    ));

    Ok(())
}

fn fly_and_expire_projectiles(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &SpellProjectile,
        &mut ProjectileLifetime,
    )>,
) {
    for (entity, mut transform, projectile, mut lifetime) in &mut projectiles {
        transform.translation += projectile.velocity * time.delta_secs();
        lifetime.tick(time.delta());
        // Derived from the lifetime fraction so growth is frame-rate independent.
        transform.scale = Vec3::splat(1.0 + lifetime.fraction() + 2.0);

        if lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
