//! Asset collections loaded by `bevy_asset_loader` in `screens::loading::preload_assets`.
use crate::shared::Config;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_seedling::sample::AudioSample;
use bevy_shuffle_bag::ShuffleBag;
use bevy_sprinkles::prelude::ParticlesAsset;

pub mod ron;

pub fn plugin(app: &mut App) {
    app.add_plugins(ron::RonLoadPlugin::<Config>::default())
        // Picks up hot-reloads; the initial insert is done by the loading state.
        .add_systems(
            Update,
            sync_config.run_if(
                resource_exists::<ConfigHandle>.and_then(resource_changed::<Assets<Config>>),
            ),
        );
}

/// Keeps the `config.ron` handle alive and gates the boot loading state on it.
#[derive(Resource)]
pub(crate) struct ConfigHandle {
    config: Handle<Config>,
}

// Manual impl: the derive tracks via `load_untyped`, which routes `.ron` to the sprinkles loader.
impl AssetCollection for ConfigHandle {
    fn create(world: &mut World) -> Self {
        let config = world.resource::<AssetServer>().load(Self::PATH);
        Self { config }
    }

    fn load(world: &mut World) -> Vec<UntypedHandle> {
        let config: Handle<Config> = world.resource::<AssetServer>().load(Self::PATH);
        vec![config.untyped()]
    }
}

impl ConfigHandle {
    const PATH: &'static str = "config.ron";
}

/// Clones the loaded `config.ron` into the world as a [`Config`] resource.
pub(crate) fn sync_config(
    handle: Res<ConfigHandle>,
    assets: Res<Assets<Config>>,
    mut commands: Commands,
) {
    if let Some(cfg) = assets.get(&handle.config) {
        commands.insert_resource(cfg.clone());
    }
}

/// UI icons: pause/mute for the gameplay HUD, github for the showcase modal.
#[derive(AssetCollection, Resource, Clone)]
pub struct Textures {
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "textures/pause.png")]
    pub pause: Handle<Image>,
    #[asset(path = "textures/mute.png")]
    pub mute: Handle<Image>,
}

/// Player model and the entry scene; needed when the level spawns in `LoadingScreen::Level`.
#[derive(AssetCollection, Resource, Clone)]
pub struct Models {
    #[asset(path = "models/player.glb")]
    pub player: Handle<Gltf>,
    #[asset(path = "models/scene.gltf")]
    pub entry_scene: Handle<Gltf>,
}

/// Raw audio handles; converted into [`AudioSources`] once loaded.
#[derive(AssetCollection, Resource, Clone)]
pub(crate) struct AudioHandles {
    #[asset(path = "audio/sfx/btn-hover.ogg")]
    hover: Handle<AudioSample>,
    #[asset(path = "audio/sfx/btn-press.ogg")]
    press: Handle<AudioSample>,
    #[asset(
        paths(
            "audio/sfx/step.ogg",
            "audio/sfx/step1.ogg",
            "audio/sfx/step2.ogg",
            "audio/sfx/step3.ogg",
            "audio/sfx/step4.ogg"
        ),
        collection(typed)
    )]
    steps: Vec<Handle<AudioSample>>,
    #[asset(paths("audio/music/smnbl-green-embrace.ogg"), collection(typed))]
    menu: Vec<Handle<AudioSample>>,
    #[asset(
        paths("audio/music/smnbl-rush-through-the-field.ogg"),
        collection(typed)
    )]
    explore: Vec<Handle<AudioSample>>,
    #[asset(paths("audio/music/smnbl-trouble.ogg"), collection(typed))]
    combat: Vec<Handle<AudioSample>>,
}

/// Playback-ready audio with never-repeating selection; built from [`AudioHandles`] after load.
#[derive(Resource)]
pub struct AudioSources {
    // SFX
    pub hover: Handle<AudioSample>,
    pub press: Handle<AudioSample>,
    pub steps: ShuffleBag<Handle<AudioSample>>,
    // music
    pub menu: ShuffleBag<Handle<AudioSample>>,
    pub explore: ShuffleBag<Handle<AudioSample>>,
    pub combat: ShuffleBag<Handle<AudioSample>>,
}

impl FromWorld for AudioSources {
    fn from_world(world: &mut World) -> Self {
        let handles = world.resource::<AudioHandles>();
        let mut rng = rand::rng();
        // Empty only if a `paths(...)` list above is emptied; fall back to a silent default handle.
        let mut bag = |tracks: &Vec<Handle<AudioSample>>| {
            let tracks = if tracks.is_empty() {
                error!("empty audio collection in AudioHandles, using default handle");
                vec![Handle::default()]
            } else {
                tracks.clone()
            };
            ShuffleBag::try_new(tracks, &mut rng).expect("audio bag is non-empty")
        };
        Self {
            hover: handles.hover.clone(),
            press: handles.press.clone(),
            steps: bag(&handles.steps),
            menu: bag(&handles.menu),
            explore: bag(&handles.explore),
            combat: bag(&handles.combat),
        }
    }
}

/// Particles attached during level spawn and by the player controller.
#[derive(AssetCollection, Resource, Clone)]
pub struct Particles {
    #[asset(path = "particles/sun-floor.ron")]
    pub sun_floor: Handle<ParticlesAsset>,
    #[asset(path = "particles/healing-zone.ron")]
    pub healing_zone: Handle<ParticlesAsset>,
    #[asset(path = "particles/wind-spin.ron")]
    pub wind_spin: Handle<ParticlesAsset>,
}

/// Preloaded so the shader-compilation screen has something to compile.
#[allow(dead_code)]
#[derive(AssetCollection, Resource, Clone)]
pub(crate) struct ShaderAssets {
    #[asset(path = "shaders/alpha_pattern.wgsl")]
    alpha_pattern: Handle<Shader>,
    #[asset(path = "shaders/cosmic_v2.wgsl")]
    cosmic_sphere: Handle<Shader>,
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
