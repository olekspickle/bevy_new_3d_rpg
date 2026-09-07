use super::*;
use avian3d::prelude::{Collider, RigidBody};
use bevy::gltf::GltfMesh;
use easy_ext::ext;

/// Helper trait to spawn mesh with minimum effort
///
/// # Example system of spawning 3D object
/// ```rust,no_run
///
/// pub fn spawn(
///     models: Res<Models>,
///     gltf_assets: Res<Assets<Gltf>>,
///     mut meshes: ResMut<Assets<Mesh>>,
///     mut commands: Commands,
/// ) {
///     let Some(obj) = gltf_assets.get(&models.scene) else {
///         return;
///     };
///
///     commands.spawn_colliding_mesh(
///         obj,
///         &meshes,
///         &gltf_meshes,
///         Transform::from_scale(Vec3::splat(3.0)),
///         );
///     }
/// ```
#[ext(CommandsExt)]
impl Commands<'_, '_> {
    pub fn spawn_colliding_mesh(
        &mut self,
        gltf: &Gltf,
        meshes: &ResMut<Assets<Mesh>>,
        gltf_meshes: &Res<Assets<GltfMesh>>,
        bundle: impl Bundle + Clone,
    ) {
        let mesh = gltf.meshes[0].clone();
        let material = gltf.materials[0].clone();
        if let Some(mesh) = gltf_meshes.get(&mesh) {
            for primitive in &mesh.primitives {
                let mesh = primitive.mesh.clone();
                let mut e = self.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    RigidBody::Static,
                    bundle.clone(),
                ));

                if let Some(mesh) = meshes.get(&mesh) {
                    e.insert(
                        Collider::trimesh_from_mesh(mesh)
                            .expect("failed to create collider from rock mesh"),
                    );
                }
            }
        }
    }
}

/// Helper trait to get direction of movement based on camera transform
///
/// # Example
///
/// ```rust,no_run
/// pub fn movement(
///     mut player: Query<&mut Player>,
///     mut camera: Query<&mut Camera3d>,
///     input: Res<Input<KeyCode>>,
/// ) {
///     if let Ok(mut player) = player.single_mut() &&
///         input.just_pressed(KeyCode::W) {
///             player.movement.direction += player.movement.direction.movement_direction(Vec2::new(0.0, 1.0));
///     }
/// }
/// ```
#[ext(TransformExt)]
impl Transform {
    /// Get movement direction as normalized verticality-agnostic vector
    pub fn movement_direction(&self, input: Vec2) -> Vec3 {
        let forward = self.forward();
        let forward_flat = Vec3::new(forward.x, 0.0, forward.z);
        let right = forward_flat.cross(Vec3::Y).normalize_or_zero();
        let direction = (right * input.x) + (forward_flat * input.y);
        direction.normalize_or_zero()
    }
}

#[ext(EntityExt)]
impl Entity {
    pub fn replace_recursive(
        &mut self,
        children_q: Query<&Children>,
        mut commands: Commands,
        r: impl Bundle,
    ) {
        if let Ok(c) = children_q.get(*self) {
            for child in c.iter() {
                commands.entity(child).despawn();
            }

            let text = commands.spawn(r).id();
            commands.entity(*self).add_children(&[text]);
        }
    }

    pub fn get_recursive<T: Component>(
        &self,
        children_q: Query<&Children>,
        component_q: Query<Entity, With<T>>,
    ) -> Option<Entity> {
        if component_q.get(*self).is_ok() {
            return Some(*self);
        }

        if let Ok(children) = children_q.get(*self) {
            for child in children.iter() {
                if let Some(e) = child.get_recursive(children_q, component_q) {
                    return Some(e);
                }
            }
        }

        None
    }
}

#[ext(AnimationPlayerExt)]
impl AnimationPlayer {
    pub fn zero_all_animations(&mut self) {
        self.playing_animations_mut().for_each(|(_, a)| {
            a.set_weight(0.0);
        });
    }
}
