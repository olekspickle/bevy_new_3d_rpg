use super::*;
use bevy::{
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial},
    render::render_resource::*,
    shader::ShaderRef,
    world_serialization::WorldInstanceReady,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<CosmicSphereMaterial>::default())
        .add_systems(Update, update_cosmic_sphere_time);
}

#[derive(ShaderType, Clone, Reflect, Debug, Default)]
#[repr(C)]
pub struct CosmicSphereUniforms {
    pub time: f32,
}

markers!(CosmicSphere);

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct CosmicSphereExtension {
    #[uniform(100)]
    pub uniforms: CosmicSphereUniforms,
}

impl MaterialExtension for CosmicSphereExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/cosmic_v2.wgsl".into()
    }
}

pub type CosmicSphereMaterial = ExtendedMaterial<StandardMaterial, CosmicSphereExtension>;

fn update_cosmic_sphere_time(time: Res<Time>, mut materials: ResMut<Assets<CosmicSphereMaterial>>) {
    let current_time = time.elapsed_secs();
    for (_, mat) in materials.iter_mut() {
        mat.extension.uniforms.time = current_time;
    }
    materials.set_changed();
}

pub fn setup_cosmic_sphere(
    _: On<WorldInstanceReady>,
    cosmic_spheres: Query<Entity, With<CosmicSphere>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<CosmicSphereMaterial>>,
) {
    let material = materials.add(CosmicSphereMaterial {
        base: StandardMaterial {
            opaque_render_method: OpaqueRendererMethod::Auto,
            base_color: Color::WHITE,
            cull_mode: None,
            unlit: true,
            ..Default::default()
        },
        extension: CosmicSphereExtension {
            uniforms: CosmicSphereUniforms::default(),
        },
    });

    for entity in cosmic_spheres.iter() {
        commands
            .entity(entity)
            // glTF meshes come with an auto-inserted `MeshMaterial3d<StandardMaterial>`
            // (bevy_gltf's "/std" material bridge). Since it's a distinct component type
            // from `MeshMaterial3d<CosmicSphereMaterial>`, both would otherwise coexist
            // and get drawn, with the plain StandardMaterial pass hiding our shader.
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((CosmicSphere, MeshMaterial3d(material.clone())));
    }
}
