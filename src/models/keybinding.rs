use super::*;
use serde::{Deserialize, Serialize};

/// Number of input columns.
pub const BINDINGS_COUNT: usize = 3;
const SETTINGS_PATH: &str = "target/settings.ron";
const GAP: Val = Val::Px(10.0);
const PADDING: UiRect = UiRect::all(Val::Px(15.0));
const PANEL_BACKGROUND: BackgroundColor = BackgroundColor(Color::srgb(0.8, 0.8, 0.8));

/// Keyboard and mouse settings.
///
/// Most games assign bindings for different input sources (keyboard + mouse, gamepads, etc.) separately or
/// even only allow rebinding for keyboard and mouse.
/// For example, gamepads use sticks for movement, which are bidirectional, so it doesn't make sense to assign
/// actions like "forward" to [`GamepadAxis::LeftStickX`].
///
/// If you want to assign a specific part of the axis, such as the positive part of [`GamepadAxis::LeftStickX`],
/// you need to create your own binding enum. However, this approach is mostly used in emulators rather than games.
///
/// So in this example we assign only keyboard and mouse bindings.
#[derive(Resource, Reflect, Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct InputSettings {
    pub forward: [Binding; BINDINGS_COUNT],
    pub left: [Binding; BINDINGS_COUNT],
    pub backward: [Binding; BINDINGS_COUNT],
    pub right: [Binding; BINDINGS_COUNT],
    pub jump: [Binding; BINDINGS_COUNT],
    pub dash: [Binding; BINDINGS_COUNT],
    pub sprint: [Binding; BINDINGS_COUNT],
    pub crouch: [Binding; BINDINGS_COUNT],
    pub attack: [Binding; BINDINGS_COUNT],
}

impl InputSettings {
    pub fn clear(&mut self) {
        self.forward.fill(Binding::None);
        self.left.fill(Binding::None);
        self.backward.fill(Binding::None);
        self.right.fill(Binding::None);
        self.jump.fill(Binding::None);
        self.dash.fill(Binding::None);
        self.sprint.fill(Binding::None);
        self.crouch.fill(Binding::None);
        self.attack.fill(Binding::None);
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            forward: [KeyCode::KeyW.into(), KeyCode::ArrowUp.into(), Binding::None],
            left: [
                KeyCode::KeyA.into(),
                KeyCode::ArrowLeft.into(),
                Binding::None,
            ],
            backward: [
                KeyCode::KeyS.into(),
                KeyCode::ArrowDown.into(),
                Binding::None,
            ],
            right: [
                KeyCode::KeyD.into(),
                KeyCode::ArrowRight.into(),
                Binding::None,
            ],
            jump: [KeyCode::Space.into(), Binding::None, Binding::None],
            dash: [KeyCode::AltLeft.into(), Binding::None, Binding::None],
            crouch: [KeyCode::ControlLeft.into(), Binding::None, Binding::None],
            sprint: [KeyCode::ShiftLeft.into(), Binding::None, Binding::None],
            attack: [MouseButton::Left.into(), Binding::None, Binding::None],
        }
    }
}
