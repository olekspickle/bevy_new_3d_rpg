use super::*;
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    pbr::{Atmosphere, AtmosphereSettings, CascadeShadowConfigBuilder, light_consts::lux},
    render::camera::Exposure,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, sun_cycle.run_if(in_state(Screen::Gameplay)));
}

/// Mainly this example:
/// <https://bevyengine.org/examples/3d-rendering/atmosphere/>
pub fn add_skybox_to_camera(
    cfg: Res<Config>,
    mut commands: Commands,
    mut camera: Query<Entity, With<SceneCamera>>,
) -> Result {
    let camera = camera.single_mut()?;

    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: cfg.physics.shadow_distance,
        ..default()
    }
    .build();

    // Sun
    commands.spawn((
        StateScoped(Screen::Gameplay),
        DirectionalLight {
            color: SUN,
            shadows_enabled: true,
            illuminance: lux::FULL_DAYLIGHT,
            ..Default::default()
        },
        Sun,
        Transform::from_translation(Vec3::new(0.0, 0.0, 200.0)),
        cascade_shadow_config.clone(),
    ));

    // Moon
    commands.spawn((
        StateScoped(Screen::Gameplay),
        DirectionalLight {
            color: MOON,
            shadows_enabled: true,
            illuminance: lux::FULL_MOON_NIGHT,
            ..Default::default()
        },
        Moon,
        Transform::from_translation(Vec3::new(0.0, 0.0, -200.0)),
        cascade_shadow_config,
    ));

    // Lighting
    commands.entity(camera).insert((
        // This is the component that enables atmospheric scattering for a camera
        // TODO: manipulate ground_albedo depending on the angle of the sun
        Atmosphere::EARTH,
        // The scene is in units of 10km, so we need to scale up the
        // aerial view lut distance and set the scene scale accordingly.
        // Most usages of this feature will not need to adjust this.
        AtmosphereSettings {
            scene_units_to_m: 1.0,
            aerial_view_lut_max_distance: 40_000.0, //  40 km for a vast scene

            // Higher resolution LUTs for smoother gradients and details
            transmittance_lut_size: UVec2::new(512, 256), // Double resolution for smoother light transmission
            sky_view_lut_size: UVec2::new(800, 400),      // Higher resolution for sky appearance
            aerial_view_lut_size: UVec3::new(64, 64, 64), // More detailed aerial perspective

            // Increased sample counts for better accuracy and less artifacts
            transmittance_lut_samples: 60, // More samples for light transmission accuracy
            multiscattering_lut_dirs: 128, // Double directions for multiscattering
            multiscattering_lut_samples: 30, // More samples for multiscattering accuracy
            sky_view_lut_samples: 24,      // More samples for sky appearance
            aerial_view_lut_samples: 15,   // More samples for aerial view depth
            ..Default::default()
        },
        Tonemapping::BlenderFilmic,
        Exposure::OVERCAST,
        Bloom::NATURAL,
    ));

    if cfg.physics.distance_fog {
        commands.entity(camera).insert(distance_fog(cfg));
    }

    Ok(())
}

pub fn rm_skybox_from_camera(
    mut commands: Commands,
    mut camera: Query<Entity, With<SceneCamera>>,
) -> Result {
    let camera = camera.single_mut()?;
    commands
        .entity(camera)
        .remove::<Atmosphere>()
        .remove::<AtmosphereSettings>()
        .remove::<Exposure>()
        .remove::<Bloom>()
        .remove::<DistanceFog>()
        .remove::<Tonemapping>();

    Ok(())
}

pub fn distance_fog(cfg: Res<Config>) -> impl Bundle {
    DistanceFog {
        color: Color::srgba(0.35, 0.48, 0.66, 1.0),
        directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
        directional_light_exponent: cfg.physics.fog_directional_light_exponent,
        falloff: FogFalloff::ExponentialSquared { density: 0.002 },
        // falloff: FogFalloff::from_visibility_colors(
        //     cfg.physics.fog_visibility, // distance in world units up to which objects retain visibility (>= 5% contrast)
        //     Color::srgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
        //     Color::srgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
        // ),
    }
}

fn sun_cycle(
    settings: Res<Settings>,
    mut sky_lights: Query<&mut Transform, Or<(With<Moon>, With<Sun>)>>,
    time: Res<Time>,
) {
    match settings.sun_cycle {
        SunCycle::DayNight => sky_lights
            .iter_mut()
            .for_each(|mut tf| tf.rotate_x(-time.delta_secs() * std::f32::consts::PI / 50.0)),
        SunCycle::Nimbus => sky_lights
            .iter_mut()
            .for_each(|mut tf| tf.rotate_y(-time.delta_secs() * std::f32::consts::PI / 50.0)),
    }
}
