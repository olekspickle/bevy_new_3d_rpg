use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect, Asset, Resource)]
#[reflect(Resource)]
pub struct Config {
    pub camera: Camera,
    pub sound: Sound,
    pub physics: Physics,
    pub player: PlayerConfig,
    pub settings: SettingsPreloaded,
    pub timers: Timers,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Sound {
    pub general: f32,
    pub music: f32,
    pub sfx: f32,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            general: 1.0,
            music: 0.5,
            sfx: 0.5,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct Physics {
    pub distance_fog: bool,
    pub fog_directional_light_exponent: f32,
    pub fog_visibility: f32,
    pub shadow_distance: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct PlayerConfig {
    pub movement: Movement,
    pub hitbox: Hitbox,
    pub zoom: (f32, f32),
    pub fov: f32,
    pub spawn_pos: (f32, f32, f32),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct Hitbox {
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct Movement {
    pub actions_in_air: u8,
    pub dash_distance: f32,
    pub speed: f32,
    pub sprint_factor: f32,
    pub crouch_factor: f32,
    pub idle_to_run_threshold: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct SettingsPreloaded {
    pub min_volume: f32,
    pub max_volume: f32,
    pub min_fov: f32,
    pub max_fov: f32,
    pub step: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct Timers {
    pub step: f32,
    pub jump: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct Camera {
    pub edge_margin: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub max_speed: f32,
    pub min_height: f32,
    pub max_height: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect, Asset, Resource)]
#[reflect(Resource)]
pub struct Credits {
    pub assets: Vec<(String, String)>,
    pub devs: Vec<(String, String)>,
}
