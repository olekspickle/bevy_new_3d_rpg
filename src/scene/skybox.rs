use super::*;
use bevy::{
    light::{
        Atmosphere, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder,
        atmosphere::ScatteringMedium, light_consts::lux,
    },
    pbr::{AtmosphereMode, AtmosphereSettings, DistanceFog, FogFalloff},
};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, sun_cycle.run_if(in_state(Screen::Gameplay)))
        .add_systems(OnEnter(Screen::Title), add_skybox_to_camera);
}

markers!(Sun, Moon);

/// Shared BSN scene for the Sun/Moon directional lights: patched per-caller for
/// color/illuminance (see `add_skybox_to_camera`). Everything both lights need
/// but don't differ on (currently just `shadow_maps_enabled`) lives here once.
fn directional_light() -> impl Scene {
    bsn! {
        DirectionalLight {
            shadow_maps_enabled: true,
        }
    }
}

/// Mainly this example:
/// <https://bevyengine.org/examples/3d-rendering/atmosphere/>
pub fn add_skybox_to_camera(
    cfg: Res<Config>,
    camera: Single<Entity, With<SceneCamera>>,
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) -> Result {
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: cfg.physics.shadow_distance,
        ..default()
    }
    .build();

    // `Transform`/`CascadeShadowConfig`/`DespawnOnExit` are precomputed runtime values with
    // no shared defaults to patch, so they're inserted as plain components on top of the
    // scene rather than forced through `bsn!`.
    commands
        .spawn_scene(bsn! {
            directional_light()
            Sun
            DirectionalLight {
                color: {colors::SUN},
                illuminance: {lux::AMBIENT_DAYLIGHT},
            }
        })
        .insert((
            Transform::from_xyz(0.0, 0.0, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
            cascade_shadow_config.clone(),
            DespawnOnExit(Screen::Gameplay),
        ));

    commands
        .spawn_scene(bsn! {
            directional_light()
            Moon
            DirectionalLight {
                color: {colors::MOON},
                illuminance: 24.0,
            }
        })
        .insert((
            Transform::from_xyz(0.0, 0.0, -200.0).looking_at(Vec3::ZERO, Vec3::Y),
            cascade_shadow_config,
            DespawnOnExit(Screen::Gameplay),
        ));

    // `Atmosphere` marks a planet entity: its `GlobalTransform` is the planet center in
    // world space, and each camera picks the *nearest* one to render with. It must live on
    // its own entity, not the camera - otherwise the camera's distance to it is zero, which
    // is read as the camera sitting `inner_radius` (millions of meters) underground and forces
    // a much more expensive render path.
    commands.spawn((
        Atmosphere::earth(scattering_mediums.add(ScatteringMedium::default())),
        DespawnOnExit(Screen::Gameplay),
    ));

    commands.entity(*camera).insert((
        // The scene is in units of 10km, so we need to scale up the
        // aerial view lut distance and set the scene scale accordingly.
        // Most usages of this feature will not need to adjust this.
        AtmosphereSettings {
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
            rendering_method: AtmosphereMode::LookupTexture,
            ..Default::default()
        },
        AtmosphereEnvironmentMapLight::default(),
    ));

    if cfg.physics.distance_fog {
        commands.entity(*camera).insert(distance_fog(cfg));
    }

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
    time: Res<Time>,
    cfg: Res<Config>,
    settings: Res<Settings>,
    mut sky_lights: Query<&mut Transform, Or<(With<Moon>, With<Sun>)>>,
) {
    let dt = time.delta_secs();
    match settings.sun_cycle {
        SunCycle::DayNight => sky_lights
            .iter_mut()
            .for_each(|mut tf| tf.rotate_x(-dt * PI / cfg.physics.day_length)),
        SunCycle::Nimbus => sky_lights
            .iter_mut()
            .for_each(|mut tf| tf.rotate_y(-dt * PI / cfg.physics.day_length)),
    }
}

#[derive(Reflect, Debug, Clone, Serialize, Deserialize)]
pub enum SunCycle {
    DayNight,
    Nimbus,
}

impl SunCycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            SunCycle::DayNight => "DayNight",
            SunCycle::Nimbus => "Nimbus",
        }
    }
}
