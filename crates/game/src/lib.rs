use asset_loading::*;
use audio::*;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use models::*;
use scene::*;

mod camera;
#[cfg(debug_assertions)]
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
        #[cfg(debug_assertions)]
        dev_tools::plugin,
    ));
}

#[derive(Default, Resource)]
pub struct Score(pub i32);
