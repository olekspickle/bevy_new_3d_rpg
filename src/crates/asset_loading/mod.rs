use super::*;
use bevy::{asset::Asset, prelude::*};
use bevy_seedling::sample::Sample;

mod ron;
mod tracking;

pub use ron::*;
pub use tracking::*;

pub fn plugin(app: &mut App) {
    // start asset loading
    app.add_plugins(tracking::plugin)
        .add_plugins(RonAssetPlugin::<Config>::default())
        .load_resource_from_path::<Config>("config.ron")
        .add_plugins(RonAssetPlugin::<Credits>::default())
        .load_resource_from_path::<Credits>("credits.ron")
        .load_resource::<AudioSources>()
        .load_resource::<Textures>()
        // .load_resource::<Fonts>()
        .load_resource::<Models>();
}

// #[derive(Asset, Clone, Reflect, Resource)]
// #[reflect(Resource)]
// pub struct Fonts {
//     #[dependency]
//     pub custom: Handle<Font>,
// }
//
// impl FromWorld for Fonts {
//     fn from_world(world: &mut World) -> Self {
//         let assets = world.resource::<AssetServer>();
//         Self {
//             custom: assets.load("fonts/custom.ttf"),
//         }
//     }
// }

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct Textures {
    #[dependency]
    pub github: Handle<Image>,
    #[dependency]
    pub pause: Handle<Image>,
    #[dependency]
    pub mute: Handle<Image>,
}

impl FromWorld for Textures {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            github: assets.load("textures/github.png"),
            pause: assets.load("textures/pause.png"),
            mute: assets.load("textures/mute.png"),
        }
    }
}

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct Models {
    #[dependency]
    pub player: Handle<Gltf>,
    #[dependency]
    pub scene: Handle<Gltf>,
    // #[dependency]
    // pub rock: Handle<Gltf>,
    // #[dependency]
    // pub comb_sphere: Handle<Gltf>,
}

impl FromWorld for Models {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player: assets.load("models/player.glb"),
            scene: assets.load("models/scene.glb"),
            // rock: assets.load("models/rock.glb"),
            // comb_sphere: assets.load("models/comb_sphere.glb"),
        }
    }
}

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct AudioSources {
    // SFX
    #[dependency]
    pub btn_hover: Handle<Sample>,
    #[dependency]
    pub btn_press: Handle<Sample>,
    #[dependency]
    pub steps: Vec<Handle<Sample>>,

    // music
    #[dependency]
    pub explore: Vec<Handle<Sample>>,
    #[dependency]
    pub combat: Vec<Handle<Sample>>,
}

impl AudioSources {
    pub const BTN_HOVER: &'static str = "audio/sfx/btn-hover.ogg";
    pub const BTN_PRESS: &'static str = "audio/sfx/btn-press.ogg";

    pub const STEPS: &[&'static str] = &[
        "audio/sfx/step.ogg",
        "audio/sfx/step1.ogg",
        "audio/sfx/step2.ogg",
        "audio/sfx/step3.ogg",
        "audio/sfx/step4.ogg",
    ];
    pub const EXPLORE: &[&'static str] = &["audio/music/smnbl-time-for-fun.ogg"];
    pub const COMBAT: &[&'static str] = &["audio/music/smnbl-trouble.ogg"];
}

impl FromWorld for AudioSources {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let steps = Self::STEPS.iter().map(|p| assets.load(*p)).collect();
        let explore = Self::EXPLORE.iter().map(|p| assets.load(*p)).collect();
        let combat = Self::COMBAT.iter().map(|p| assets.load(*p)).collect();
        Self {
            steps,
            combat,
            explore,
            btn_hover: assets.load(Self::BTN_HOVER),
            btn_press: assets.load(Self::BTN_PRESS),
        }
    }
}
