use super::*;
use serde::Deserialize;
use std::{error::Error, fs};

pub fn plugin(app: &mut App) {
    app.init_resource::<Settings>().init_resource::<ActiveTab>();
    app.add_systems(
        OnEnter(Screen::Title),
        inject_settings_from_cfg.run_if(resource_exists::<Config>.and(run_once)),
    );
}

pub const SETTINGS_PATH: &str = "assets/settings.ron";

#[derive(Resource, Reflect, Deserialize, Serialize, Debug, Clone)]
#[reflect(Resource)]
pub struct Settings {
    // audio
    pub sound: Sound,
    // video
    pub fov: f32,
    pub sun_cycle: SunCycle,
    // keybindings
    pub input_map: InputSettings,
}

impl Settings {
    pub fn music(&self) -> Volume {
        Volume::Linear(self.sound.general * self.sound.music)
    }

    pub fn sfx(&self) -> Volume {
        Volume::Linear(self.sound.general * self.sound.sfx)
    }

    pub fn read() -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(SETTINGS_PATH)?;
        let settings = ron::from_str(&content).unwrap_or_default();
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let content = ron::ser::to_string_pretty(self, Default::default())?;
        fs::write(SETTINGS_PATH, content)?;
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sun_cycle: SunCycle::DayNight,
            sound: Sound::default(),
            fov: 45.0, // bevy default
            input_map: InputSettings::default(),
        }
    }
}

fn inject_settings_from_cfg(mut commands: Commands) {
    let settings = match Settings::read() {
        Ok(settings) => {
            info!("loaded settings from '{SETTINGS_PATH}'");
            settings
        }
        Err(e) => {
            info!("unable to load settings from '{SETTINGS_PATH}', switching to defaults: {e}");
            Default::default()
        }
    };

    commands.insert_resource(settings);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, Component)]
#[reflect(Component)]
pub enum UiTab {
    #[default]
    Audio,
    Video,
    Keybindings,
}

#[derive(Resource, Default)]
pub struct ActiveTab(pub UiTab);
