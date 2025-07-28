use super::*;
use serde::{Deserialize, Serialize};

/// Keyboard and mouse settings.
///
/// Most games assign bindings for different input sources (keyboard + mouse, gamepads, etc.) separately or
/// even only allow rebinding for keyboard and mouse.
/// For example, gamepads use sticks for movement, which are bidirectional, so it doesn't make sense to assign
/// actions like "forward" to [`GamepadAxis::LeftStickX`].
///
/// If you want to assign a specific part of the axis, such as the positive part of [`GamepadAxis::LeftStickX`],
/// you need to create your own input enum. However, this approach is mostly used in emulators rather than games.
#[derive(Resource, Reflect, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Keybind {
    pub left: Vec<Input>,
    pub right: Vec<Input>,
    pub forward: Vec<Input>,
    pub backward: Vec<Input>,
    pub jump: Vec<Input>,
    pub dash: Vec<Input>,
    pub crouch: Vec<Input>,
    pub sprint: Vec<Input>,
    pub attack: Vec<Input>,
}
impl Keybind {
    pub fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
        self.forward.clear();
        self.backward.clear();
        self.jump.clear();
        self.dash.clear();
        self.crouch.clear();
        self.sprint.clear();
        self.attack.clear();
    }
}

impl Default for Keybind {
    fn default() -> Self {
        Self {
            forward: vec![KeyCode::KeyW.into(), KeyCode::ArrowUp.into()],
            left: vec![KeyCode::KeyA.into(), KeyCode::ArrowLeft.into()],
            backward: vec![KeyCode::KeyS.into(), KeyCode::ArrowDown.into()],
            right: vec![KeyCode::KeyD.into(), KeyCode::ArrowRight.into()],
            jump: vec![KeyCode::Space.into()],
            dash: vec![KeyCode::AltLeft.into()],
            crouch: vec![KeyCode::ControlLeft.into()],
            sprint: vec![KeyCode::ShiftLeft.into()],
            attack: vec![MouseButton::Left.into()],
        }
    }
}
