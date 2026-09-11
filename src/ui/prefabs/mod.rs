use super::*;

mod keybind_editor;
mod modals;
mod settings;
mod showcase;

pub use keybind_editor::*;
pub use modals::*;
pub use settings::*;
pub use showcase::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((keybind_editor::plugin, settings::plugin))
        .add_systems(Update, toggle_ui_showcase);
}
