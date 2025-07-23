use super::*;
use std::collections::HashMap;

pub fn plugin(app: &mut App) {
    app.init_resource::<GameState>().init_resource::<Moods>();
}

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct GameState {
    /// Modal stack. kudo for the idea to @skyemakesgames
    /// Only relevant in [`Screen::Gameplay`]
    pub modals: Vec<Modal>,
    pub last_screen: Screen,
    pub current_mood: MoodType,

    /// Only relevant in top_down camera mode
    /// Track if we should move or rotate camera on mouse move
    pub camera_mode: CameraMode,

    pub diagnostics: bool,
    pub debug_ui: bool,
    pub paused: bool,
    pub muted: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            modals: vec![],
            last_screen: Screen::Title,
            camera_mode: CameraMode::Move,
            current_mood: MoodType::Exploration,
            diagnostics: true,
            debug_ui: false,
            paused: false,
            muted: false,
        }
    }
}

impl GameState {
    pub fn reset(&mut self) {
        self.modals.clear();
        self.paused = false;
        self.muted = false;
    }
}

/// The game's main screen states.
/// See <https://bevy-cheatbook.github.io/programming/states.html>
/// Or <https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs>
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum Screen {
    // Bevy tribute <3
    #[default]
    Splash,
    // During the loading State the LoadingPlugin will load our assets
    Loading,
    Tutorial,
    Credits,
    Settings,
    // Here the menu is drawn and waiting for player interaction
    Title,
    // During this State the actual game logic is executed
    Gameplay,
}

#[derive(Resource, Reflect, Debug, Clone, Default)]
#[reflect(Resource)]
pub struct Moods {
    pub inner: HashMap<MoodType, Entity>,
}

#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum MoodType {
    #[default]
    Exploration,
    Combat,
}

// #[cfg(feature = "top_down")]
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum CameraMode {
    #[default]
    Move,
    Rotate,
}
