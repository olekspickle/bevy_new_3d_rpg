//! Various non-themable constants
use super::*;

/// Font asset paths
pub mod fonts {
    /// Default regular font path
    pub const REGULAR: &str = "embedded://bevy_new_3d_rpg/assets/fonts/FiraCode-Regular.ttf";
    /// Regular italic font path
    pub const ITALIC: &str = "embedded://bevy_new_3d_rpg/assets/fonts/FiraCode-Italic.ttf";
    /// Bold font path
    pub const BOLD: &str = "embedded://bevy_new_3d_rpg/assets/fonts/FiraCode-Bold.ttf";
    /// Monospace font path
    pub const MONO: &str = "embedded://bevy_new_3d_rpg/assets/fonts/Not-Jam-Mono-Clean-16.ttf";
    ///  Pixel font path
    pub const PIXEL: &str = "embedded://bevy_new_3d_rpg/assets/fonts/pixelSquare.ttf";
}

/// Size constants
pub mod size {
    use super::*;

    /// Default font size
    pub const FONT_SIZE: f32 = 24.0;

    /// Default border radius for buttons, sliders, spinners, etc.
    pub const BORDER_RADIUS: Val = Px(15.0);

    /// Common row size for buttons, sliders, spinners, etc.
    pub const ROW_HEIGHT: Val = Px(24.0);

    /// Width and height of a checkbox
    pub const CHECKBOX_SIZE: Val = Px(18.0);

    /// Width and height of a radio button
    pub const RADIO_SIZE: Val = Px(18.0);

    /// Width of a toggle switch
    pub const TOGGLE_WIDTH: Val = Px(32.0);

    /// Height of a toggle switch
    pub const TOGGLE_HEIGHT: Val = Px(18.0);
}

pub mod colors {
    pub use bevy::color::palettes::{css::*, tailwind::*};
    use bevy::prelude::Color;

    /// #00000000
    pub const TRANSPARENT: Color = Color::srgba(0.0, 0.0, 0.0, 0.0);
    /// #332666b3
    pub const TRANSLUCENT: Color = Color::srgba(0.2, 0.15, 0.4, 0.7);
    /// #000000
    pub const BLACK: Color = Color::oklcha(0.0, 0.0, 0.0, 1.0);
    /// #1F1F24
    pub const GRAY: Color = Color::oklcha(0.2414, 0.0095, 285.67, 1.0);
    /// #282828
    pub const GRAY_0: Color = Color::srgb(0.157, 0.157, 0.157);
    /// #2A2A2E
    pub const GRAY_1: Color = Color::oklcha(0.2866, 0.0072, 285.93, 1.0);
    /// #36373B
    pub const GRAY_2: Color = Color::oklcha(0.3373, 0.0071, 274.77, 1.0);
    /// #46474D
    pub const GRAY_3: Color = Color::oklcha(0.3992, 0.0101, 278.38, 1.0);
    /// #414142
    pub const WARM_GRAY_1: Color = Color::oklcha(0.3757, 0.0017, 286.32, 1.0);
    /// #838385
    pub const LIGHT_GRAY_1: Color = Color::oklcha(0.6106, 0.003, 286.31, 1.0);
    /// #B1B1B2
    pub const LIGHT_GRAY_2: Color = Color::oklcha(0.7607, 0.0014, 286.37, 1.0);
    /// #ececec
    pub const WHITEISH: Color = Color::srgb(0.925, 0.925, 0.925);
    /// #FFFFFF
    pub const WHITE: Color = Color::oklcha(1.0, 0.000000059604645, 90.0, 1.0);
    /// #cdaa6d
    pub const SAND_YELLOW: Color = Color::srgb(205. / 255., 170. / 255., 109. / 255.);
    /// #49e05f
    pub const ACID_GREEN: Color = Color::srgb(0.286, 0.878, 0.373);
    /// #5D8D0A
    pub const GRASS_GREEN: Color = Color::oklcha(0.5866, 0.1543, 129.84, 1.0);
    /// #2f5392
    pub const DIM_BLUE: Color = Color::srgb(0.186, 0.328, 0.573);
    /// #2160A3
    pub const BLUE: Color = Color::oklcha(0.4847, 0.1249, 253.08, 1.0);
    /// #206EC9
    pub const BRIGHT_BLUE: Color = Color::oklcha(0.4847, 0.1249, 253.08, 1.0);
    /// #4979c5
    pub const CHROME_BLUE: Color = Color::srgb(0.286, 0.478, 0.773);
    /// #fac896
    pub const SUN: Color = Color::srgb(250.0 / 255.0, 200.0 / 255.0, 150.0 / 255.0);
    /// #4d5ad5
    pub const MOON: Color = Color::srgb(77.0 / 255.0, 90.0 / 255.0, 213.0 / 255.0);
    /// #AB4051
    pub const RED: Color = Color::oklcha(0.5232, 0.1404, 13.84, 1.0);
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct Palette {
    pub text: Color,
    pub bg: Color,
    pub border: BorderColor,
}

impl Palette {
    pub fn new(text: Color, bg: Color, border: BorderColor) -> Self {
        Self { text, bg, border }
    }
}

/// Palette for widget interactions
/// Add this to an entity you want changing color properties
#[derive(Component, Clone, Debug, Reflect)]
pub struct PaletteSet {
    pub none: Palette,
    pub hovered: Palette,
    pub pressed: Palette,
    pub disabled: Palette,
}
impl Default for PaletteSet {
    fn default() -> Self {
        Self {
            none: Palette::new(
                colors::WHITE,
                colors::TRANSPARENT,
                BorderColor::all(colors::WHITEISH),
            ),
            pressed: Palette::new(
                colors::TRANSPARENT,
                colors::BRIGHT_BLUE,
                BorderColor::DEFAULT,
            ),
            hovered: Palette::new(colors::WHITE, colors::BLUE, BorderColor::DEFAULT),
            disabled: Palette::new(
                colors::TRANSPARENT,
                colors::WARM_GRAY_1,
                BorderColor::DEFAULT,
            ),
        }
    }
}
