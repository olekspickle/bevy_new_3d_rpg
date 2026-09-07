use avian3d::prelude::PhysicsPlugins;
use bevy::prelude::*;
use bevy_ahoy::prelude::*;
// use bevy_fix_cursor_unlock_web::prelude::*;
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_seedling::SeedlingPlugins;
use bevy_skein::SkeinPlugin;
use bevy_sprinkles::SprinklesPlugin;
#[cfg(feature = "third_person")]
pub use bevy_third_person_camera::{
    ThirdPersonCamera, ThirdPersonCameraPlugin, ThirdPersonCameraTarget,
};
#[cfg(feature = "top_down")]
pub use bevy_top_down_camera::{TopDownCameraPlugin, TopDownCameraTarget};

pub fn plugin(app: &mut App) {
    let seed: u64 = 256;

    app.add_plugins((
        // FixPointerUnlockPlugin,
        SeedlingPlugins,
        EnhancedInputPlugin,
        SkeinPlugin::default(),
        PhysicsPlugins::default(),
        SprinklesPlugin,
        AhoyPlugins::default(),
        EntropyPlugin::<WyRand>::with_seed(seed.to_ne_bytes()),
    ));

    #[cfg(feature = "third_person")]
    app.add_plugins(ThirdPersonCameraPlugin).configure_sets(
        PostUpdate,
        bevy_third_person_camera::CameraSyncSet.before(TransformSystems::Propagate),
    );
    #[cfg(feature = "top_down")]
    app.add_plugins(TopDownCameraPlugin).configure_sets(
        PostUpdate,
        bevy_top_down_camera::CameraSyncSet.before(TransformSystems::Propagate),
    );
}
