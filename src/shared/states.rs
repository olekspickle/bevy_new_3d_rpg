use super::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<GameState>();
}

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct GameState {
    pub last_screen: Screen,

    pub diagnostics: bool,
    pub debug_ui: bool,
    pub paused: bool,
    pub muted: bool,
    pub inspector_enabled: bool,
    pub inspector_own_window: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            last_screen: Screen::Title,
            diagnostics: true,
            debug_ui: true,
            paused: false,
            muted: false,
            inspector_enabled: false,
            inspector_own_window: true,
        }
    }
}

impl GameState {
    pub fn reset(&mut self) {
        self.paused = false;
        self.muted = false;
    }
}
