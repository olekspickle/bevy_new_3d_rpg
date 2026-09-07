use crate::shared::Config;
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy_seedling::sample::AudioSample;
use bevy_shuffle_bag::ShuffleBag;
use bevy_sprinkles::prelude::ParticleSystemAsset;

pub mod ron;
mod tracking;
pub(crate) use tracking::*;

pub fn plugin(app: &mut App) {
    // start asset loading
    app.add_plugins(tracking::plugin)
        .add_plugins(ron::RonLoadPlugin::<Config>::default())
        .load_resource_from_path::<Config>("config.ron")
        .load_resource::<AudioSources>()
        .load_resource::<Particles>()
        .load_resource::<Textures>()
        .load_resource::<Models>();
    // .load_resource::<ShaderAssets>()
    // .load_resource::<Fonts>();
}

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
    pub entry_scene: Handle<Gltf>,
}

impl FromWorld for Models {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player: assets.load("models/player.glb"),
            entry_scene: assets.load("models/scene.gltf"),
        }
    }
}

#[derive(Asset, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct AudioSources {
    // SFX
    #[dependency]
    pub hover: Handle<AudioSample>,
    #[dependency]
    pub press: Handle<AudioSample>,
    #[dependency]
    pub steps: ShuffleBag<Handle<AudioSample>>,

    // music
    #[dependency]
    pub menu: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    pub explore: ShuffleBag<Handle<AudioSample>>,
    #[dependency]
    pub combat: ShuffleBag<Handle<AudioSample>>,
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
    pub const MENU: &[&'static str] = &["audio/music/smnbl-green-embrace.ogg"];
    pub const EXPLORE: &[&'static str] = &["audio/music/smnbl-rush-through-the-field.ogg"];
    pub const COMBAT: &[&'static str] = &["audio/music/smnbl-trouble.ogg"];
}

impl FromWorld for AudioSources {
    fn from_world(world: &mut World) -> Self {
        let mut rng = rand::rng();
        let a = world.resource::<AssetServer>();

        let steps = Self::STEPS.iter().map(|p| a.load(*p)).collect::<Vec<_>>();
        let explore = Self::EXPLORE.iter().map(|p| a.load(*p)).collect::<Vec<_>>();
        let combat = Self::COMBAT.iter().map(|p| a.load(*p)).collect::<Vec<_>>();
        let menu = Self::MENU.iter().map(|p| a.load(*p)).collect::<Vec<_>>();

        Self {
            menu: ShuffleBag::try_new(menu, &mut rng).unwrap(),
            steps: ShuffleBag::try_new(steps, &mut rng).unwrap(),
            combat: ShuffleBag::try_new(combat, &mut rng).unwrap(),
            explore: ShuffleBag::try_new(explore, &mut rng).unwrap(),
            hover: a.load(Self::BTN_HOVER),
            press: a.load(Self::BTN_PRESS),
        }
    }
}

#[derive(Resource, Asset, Clone, TypePath)]
pub struct Particles {
    #[dependency]
    pub sun_floor: Handle<ParticleSystemAsset>,
    #[dependency]
    pub healing_zone: Handle<ParticleSystemAsset>,
    #[dependency]
    pub wind_spin: Handle<ParticleSystemAsset>,
}

impl FromWorld for Particles {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            sun_floor: assets.load("particles/sun-floor.ron"),
            healing_zone: assets.load("particles/healing-zone.ron"),
            wind_spin: assets.load("particles/wind-spin.ron"),
        }
    }
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[allow(dead_code)]
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct ShaderAssets {
    #[dependency]
    alpha_pattern: Handle<Shader>,
    #[dependency]
    cosmic_sphere: Handle<Shader>,
}

impl FromWorld for ShaderAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            alpha_pattern: assets.load("shaders/alpha_pattern.wgsl"),
            cosmic_sphere: assets.load("shaders/cosmic_sphere.wgsl"),
        }
    }
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
