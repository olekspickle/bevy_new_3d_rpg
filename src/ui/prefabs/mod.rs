use super::*;

mod keybind_editor;
mod settings;

pub use keybind_editor::*;
pub use settings::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((keybind_editor::plugin, settings::plugin));
}
