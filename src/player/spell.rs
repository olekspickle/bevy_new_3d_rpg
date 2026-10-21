//! Casts a simple magic bolt: `CastSpell` plays the Enter→Shoot→Exit animation sequence (Shoot
//! repeats while held), and each time the Shoot node starts (see [`SpellShootFrame`]) a
//! `CosmicSphere`-shaded projectile launches from the player's hand bone along the camera's forward.
use super::*;
use crate::scene::{
    CosmicSphere, CosmicSphereExtension, CosmicSphereMaterial, CosmicSphereUniforms,
};
use bevy::material::OpaqueRendererMethod;

mod knobs {
    use bevy::prelude::Color;

    pub const PROJECTILE_SPEED: f32 = 20.0;
    pub const PROJECTILE_START_RADIUS: f32 = 0.3;
    pub const PROJECTILE_LIFETIME_SECS: f32 = 6.0;
    pub const LIGHT_COLOR: Color = Color::srgb(0.6, 0.4, 1.0);
    pub const LIGHT_INTENSITY: f32 = 200_000.0;
    pub const LIGHT_RANGE: f32 = 10.0;
    pub const FACING_TURN_RATE: f32 = 0.2;
}

pub fn plugin(app: &mut App) {
    app.add_observer(handle_cast_spell)
        .add_observer(handle_stop_casting)
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
    face_camera(on.context, &camera, &mut commands);
    Ok(())
}

fn handle_stop_casting(on: On<Complete<CastSpell>>, mut player_q: Query<&mut Player>) -> Result {
    player_q.get_mut(on.context)?.animation.stop_casting();
    Ok(())
}

fn face_camera(player: Entity, camera: &Transform, commands: &mut Commands) {
    let forward = camera.forward();
    let direction = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    if direction != Vec3::ZERO {
        let target = Quat::from_rotation_y(direction.x.atan2(direction.z));
        commands.entity(player).insert(FacingTarget(target));
    }
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
    // re-aim each shot of a held cast
    face_camera(on.event_target(), &camera, &mut commands);

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
        // no shadows: held casts spawn many of these
        PointLight {
            color: knobs::LIGHT_COLOR,
            intensity: knobs::LIGHT_INTENSITY,
            range: knobs::LIGHT_RANGE,
            shadow_maps_enabled: false,
            ..default()
        },
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
