use super::*;
use crate::scene::SunCycle;
use bevy::settings::{ReflectSettingsGroup, SettingsGroup};
use bevy_seedling::prelude::Volume;

pub fn plugin(app: &mut App) {
    app.register_type::<Settings>();
    app.init_resource::<Settings>();
}

#[derive(Resource, SettingsGroup, Reflect, Debug, Clone)]
#[reflect(Resource, SettingsGroup, Default)]
pub struct Settings {
    // audio
    pub sound: SoundPreset,
    // video
    pub fov: f32,
    pub sun_cycle: SunCycle,
    // keybindings
    pub input_map: InputSettings,
}

impl Settings {
    pub fn general(&self) -> Volume {
        Volume::Linear(self.sound.general)
    }
    pub fn music(&self) -> Volume {
        Volume::Linear(self.sound.general * self.sound.music)
    }

    pub fn sfx(&self) -> Volume {
        Volume::Linear(self.sound.general * self.sound.sfx)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sun_cycle: SunCycle::DayNight,
            sound: SoundPreset::default(),
            fov: 45.0, // bevy default
            input_map: InputSettings::default(),
        }
    }
}

/// Fired when [`Settings`] change so settings UI can refresh its content.
#[derive(Event)]
pub struct SettingsChanged;
