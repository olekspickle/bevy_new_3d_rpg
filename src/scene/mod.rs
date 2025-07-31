use crate::*;
use avian3d::prelude::*;
use bevy_skein::SkeinPlugin;

mod skybox;

pub use skybox::*;

/// This plugin handles loading and saving scenes
/// Scene logic is only active during the State `Screen::Playing`
pub fn plugin(app: &mut App) {
    app.add_plugins((
        PhysicsPlugins::default(),
        SkeinPlugin::default(),
        bevy_fix_gltf_coordinate_system::FixGltfCoordinateSystemPlugin,
        skybox::plugin,
    ))
    .add_systems(OnEnter(Screen::Title), setup);
}

pub fn setup(models: Res<Models>, gltf_assets: Res<Assets<Gltf>>, mut commands: Commands) {
    let Some(scene) = gltf_assets.get(&models.scene) else {
        return;
    };
    commands.spawn((
        SceneRoot(scene.scenes[0].clone()),
        Transform::from_scale(Vec3::splat(1.0)),
    ));

    // to see something when suns go away
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..Default::default()
    });
}
