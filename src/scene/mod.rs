//! This plugin handles loading and saving scenes
//!
//! We use blender->bevy workflow with the the help of skein plugin
//! Which allows you to add components to meshes inside blender
//!
//! Sometimes libraries you depend on and use their components in blender add some changes
//! like rename components or add new enum variants, for example it appened once when
//! avian3d added voxel stuff and all TrimeshFromMesh collider constructors were replaced by
//! VoxelisedTrimesh because blender storing enums not as a string but as a enum discriminant.
//!
//! In such cases the mass rename exists:
//! ```bash
//! blender --background -b art/tunic.blend -c change_component_path --old_path tunic_bush::BushSensor --new_path api::BushSensor
//!
//! ```
//! more on that here: <https://bevyskein.dev/docs/migration-tools>
//! Scene logic is only active during the State `Screen::Playing`
use crate::asset_loading::{Models, Particles};
use crate::camera::SceneCamera;
use crate::game::Mood;
use crate::markers;
use crate::screens::Screen;
use crate::shared::{Config, Settings};
use crate::ui::colors;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use bevy_sprinkles::prelude::*;

mod cosmic_sphere;
mod screen_fade;
mod skybox;
pub use cosmic_sphere::*;
pub use screen_fade::*;
pub use skybox::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((skybox::plugin, screen_fade::plugin, cosmic_sphere::plugin));
}

pub fn spawn_level(models: Res<Models>, gltf_assets: Res<Assets<Gltf>>, mut commands: Commands) {
    let Some(scene) = gltf_assets.get(&models.entry_scene) else {
        return;
    };
    commands
        .spawn((
            SceneRoot(scene.scenes[0].clone()),
            Transform::from_scale(Vec3::splat(1.0)),
        ))
        .observe(attach_particles)
        .observe(setup_cosmic_sphere);

    // to see something when suns go away
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..Default::default()
    });
}

fn attach_particles(
    _: On<SceneInstanceReady>,
    moods: Query<(Entity, &Mood)>,
    transforms: Query<&Transform>,
    particles: Res<Particles>,
    mut commands: Commands,
) {
    for (e, mood) in moods.iter() {
        if let Ok(transform) = transforms.get(e) {
            let mut pos = *transform;
            pos.scale = Vec3::new(0.2, 2.0, 0.2);

            let handle = match mood {
                Mood::Exploration => particles.healing_zone.clone(),
                Mood::Combat => particles.sun_floor.clone(),
            };
            commands.entity(e).with_children(|parent| {
                parent.spawn((pos, ParticleSystem3D { handle }));
            });
        }
    }
}
