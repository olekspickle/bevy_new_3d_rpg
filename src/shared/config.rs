use bevy::{asset::Asset, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Asset, Clone, Debug, Default, Serialize, Deserialize, Reflect, Resource)]
#[reflect(Resource)]
pub struct Config {
    pub camera: CameraPreset,
    pub sound: SoundPreset,
    pub physics: PhysicsPreset,
    pub player: PlayerPreset,
    pub settings: SettingsPreset,
    pub timers: TimersPreset,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SoundPreset {
    pub general: f32,
    pub music: f32,
    pub sfx: f32,
}

impl Default for SoundPreset {
    fn default() -> Self {
        Self {
            general: 1.0,
            music: 0.5,
            sfx: 0.5,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct PhysicsPreset {
    pub day_length: f32,
    pub distance_fog: bool,
    pub fog_directional_light_exponent: f32,
    pub fog_visibility: f32,
    pub shadow_distance: f32,
    pub gravity: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct PlayerPreset {
    pub movement: MovementPreset,
    pub hitbox: HitboxPreset,
    pub zoom: (f32, f32),
    pub fov: f32,
    pub spawn_pos: (f32, f32, f32),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct HitboxPreset {
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct MovementPreset {
    pub max_speed: f32,
    pub jump_height: f32,
    pub sprint_factor: f32,
    pub crouch_factor: f32,
    pub dash_distance: f32,
    pub idle_to_run_threshold: f32,
}

impl MovementPreset {
    pub fn speed(&self) -> f32 {
        self.max_speed / 2.0
    }
    pub fn sprint_speed(&self) -> f32 {
        self.speed() * self.sprint_factor
    }
    pub fn crouch_speed(&self) -> f32 {
        self.speed() * self.crouch_factor
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct SettingsPreset {
    pub min_volume: f32,
    pub max_volume: f32,
    pub min_fov: f32,
    pub max_fov: f32,
    pub step: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct TimersPreset {
    pub step: f32,
    pub jump: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct CameraPreset {
    pub edge_margin: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub max_speed: f32,
    pub min_height: f32,
    pub max_height: f32,
}
