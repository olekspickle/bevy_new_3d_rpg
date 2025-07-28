//! Simple setup for a game: main bus, music and sfx channels
//!
//! [Music sampler pool](Music)
//! [Sfx sampler pool](Sfx)
//!
//! ```text
//! ┌─────┐┌───┐┌───────────┐
//! │Music││Sfx││DefaultPool│
//! └┬────┘└┬──┘└┬──────────┘
//! ┌▽──────▽────▽┐
//! │MainBus      │
//! └─────────────┘
//! ```
//!
//! The `Music` pool, `Sfx` pool, and `DefaultPool` are all routed to the `MainBus` node.
//! Since each pool has a `VolumeNode`, we can control them all individually. And,
//! since they're all routed to the `MainBus`, we can also set the volume of all three
//! at once.
//!
//! You can see this in action in the knob observers: to set the master volume,
//! we adjust the `MainBus` node, and to set the individual volumes, we adjust the
//! pool nodes.
//!
//! # Example
//! ```rust,no_run
//! #[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
//! pub struct Sound {
//!     pub general: f32,
//!     pub music: f32,
//!     pub sfx: f32,
//! }
//! fn lower_general(
//!     mut sound: ResMut<Sound>,
//!     mut general: Single<&mut VolumeNode, With<MainBus>>,
//! ) {
//!     let new_volume = (sound.general - 0.1).max(3.0);
//!     sound.general = new_volume;
//!     general.volume = Volume::Linear(new_volume);
//! }
//! ```
//!
use bevy::prelude::*;
use bevy_seedling::{pool::SamplerPool, prelude::*};

pub fn plugin(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        bevy_seedling::SeedlingPlugin::<firewheel_web_audio::WebAudioBackend> {
            config: Default::default(),
            stream_config: Default::default(),
            spawn_default_pool: true,
            pool_size: 4..=32,
        },
    );

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(bevy_seedling::SeedlingPlugin::default());

    app.add_systems(Startup, spawn_pools);
}

fn spawn_pools(mut master: Single<&mut VolumeNode, With<MainBus>>, mut cmds: Commands) {
    // Since the main bus already exists, we can just set the desired volume.
    master.volume = Volume::UNITY_GAIN;

    cmds.spawn((
        SamplerPool(Music),
        VolumeNode {
            volume: Volume::Linear(0.5),
        },
    ));
    cmds.spawn((
        SamplerPool(Sfx),
        VolumeNode {
            volume: Volume::Linear(0.5),
        },
    ));
}

/// An organizational marker component that indicates that [`SamplePlayer`] should be routed to the music sampler pool.
///
/// Suitable for anything in "music" category (e.g. global background music, soundtrack)
///
/// This can then be used to query for and operate on sounds in that category
/// ```rust,no_run
/// commands.spawn(
///        Music,
///        SamplePlayer::new(handle).with_volume(Volume::Linear(vol)),
///    );
///
/// // or looping
///
/// commands.spawn(
///        Music,
///        SamplePlayer::new(handle).with_volume(Volume::Linear(vol)).looping(),
///    );
/// ```
#[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub struct Music;

/// An organizational marker component that indicates that [`SamplePlayer`] should be routed to the SFX sampler pool.
///
/// Suitable for anything in "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening)
///
/// This can then be used to query for and operate on sounds in that category
/// ```rust,no_run
/// commands.spawn(
///        Sfx,
///        SamplePlayer::new(handle).with_volume(Volume::Linear(vol)),
///    );
///
/// // or looping
///
/// commands.spawn(
///        Sfx,
///        SamplePlayer::new(handle).with_volume(Volume::Linear(vol)).looping(),
///    );
/// ```
#[derive(PoolLabel, Debug, Clone, PartialEq, Eq, Hash, Default, Reflect)]
#[reflect(Component)]
pub struct Sfx;
