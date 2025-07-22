use asset_loading::*;
use audio::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use models::*;
use scene::*;

mod camera;
#[cfg(feature = "dev_native")]
mod dev_tools;
mod mood;

pub use camera::*;

pub fn plugin(app: &mut App) {
    app.insert_resource(Score(0));
    app.add_plugins((
        models::plugin,
        camera::plugin,
        scene::plugin,
        player::plugin,
        mood::plugin,
        #[cfg(feature = "dev_native")]
        dev_tools::plugin,
    ));
}

#[derive(Default, Resource)]
pub struct Score(pub i32);
