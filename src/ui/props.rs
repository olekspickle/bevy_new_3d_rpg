use super::*;
use std::borrow::Cow;

#[derive(Debug, Clone, Bundle)]
pub struct Props {
    pub content: WidgetContent,
    pub palette_set: PaletteSet,
    // layout
    pub border_color: BorderColor,
    pub bg_color: BackgroundColor,
    pub node: Node,
}

#[allow(dead_code)]
impl Props {
    pub fn new(c: impl Into<WidgetContent>) -> Self {
        Self {
            content: c.into(),
            palette_set: PaletteSet::default(),
            node: Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                justify_items: JustifyItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Px(2.0)),
                padding: UiRect::horizontal(Vw(3.0)),
                border_radius: BorderRadius::all(size::BORDER_RADIUS),
                ..Default::default()
            },
            bg_color: BackgroundColor(colors::TRANSPARENT),
            border_color: BorderColor::all(colors::WHITEISH),
        }
    }

    pub fn font(mut self, font: TextFont) -> Self {
        if let WidgetContent::Text(ref mut t) = self.content {
            t.font = font;
        }
        self
    }
    pub fn font_size(mut self, s: f32) -> Self {
        if let WidgetContent::Text(ref mut t) = self.content {
            t.font.font_size = FontSize::Px(s);
        }
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        if let WidgetContent::Text(ref mut t) = self.content {
            *t.color = c;
        }
        self
    }
    pub fn bg_color(mut self, color: Color) -> Self {
        self.bg_color = BackgroundColor(color);
        self
    }
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = BorderColor::all(color);
        self
    }
    pub fn border_radius(mut self, r: Val) -> Self {
        self.node.border_radius = BorderRadius::all(r);
        self
    }
    pub fn node(mut self, new: Node) -> Self {
        self.node = new;
        self
    }
    pub fn border(mut self, b: UiRect) -> Self {
        self.node.border = b;
        self
    }
    pub fn hidden(mut self) -> Self {
        self.node.display = Display::None;
        self
    }
    pub fn width(mut self, w: Val) -> Self {
        self.node.width = w;
        self
    }
    pub fn height(mut self, h: Val) -> Self {
        self.node.height = h;
        self
    }
    pub fn row_gap(mut self, g: Val) -> Self {
        self.node.row_gap = g;
        self
    }
    pub fn margin(mut self, m: UiRect) -> Self {
        self.node.margin = m;
        self
    }
    pub fn padding(mut self, p: UiRect) -> Self {
        self.node.padding = p;
        self
    }
    pub fn flex_direction(mut self, d: FlexDirection) -> Self {
        self.node.flex_direction = d;
        self
    }
    pub fn palette_set(mut self, p: PaletteSet) -> Self {
        self.palette_set = p;
        self
    }

    // Content related methods

    pub fn image(mut self, s: Handle<Image>) -> Self {
        self.content = WidgetContent::Image(ImageNode::new(s));
        self
    }
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        self.content = WidgetContent::Text(text.into().into());
        self
    }
    // TODO: do a mesh2d ui bundle, similar to svg
    pub fn into_image_bundle(self) -> impl Bundle {
        match self.content {
            WidgetContent::Image(c) => c,
            _ => unreachable!("Spawning image bundle on non image content"),
        }
    }
    pub fn into_text_bundle(self) -> impl Bundle {
        match self.content {
            WidgetContent::Text(c) => c,
            _ => unreachable!("Spawning text bundle on non text content"),
        }
    }
}

impl Default for Props {
    fn default() -> Self {
        Props::new("")
    }
}

#[derive(Debug, Clone, Component)]
pub enum WidgetContent {
    Image(ImageNode),
    Text(TextContent),
}

#[derive(Debug, Clone, Bundle)]
pub struct TextContent {
    pub text: Text,
    pub color: TextColor,
    pub layout: TextLayout,
    pub font: TextFont,
    pub border: BorderColor,
}

impl From<Cow<'static, str>> for TextContent {
    fn from(text: Cow<'static, str>) -> Self {
        Self {
            text: Text(text.into()),
            ..Default::default()
        }
    }
}
impl Default for TextContent {
    fn default() -> Self {
        Self {
            text: "".into(),
            color: colors::WHITEISH.into(),
            layout: TextLayout::justify(Justify::Center),
            font: TextFont::from_font_size(size::FONT_SIZE),
            border: BorderColor {
                bottom: colors::WHITEISH,
                ..BorderColor::DEFAULT
            },
        }
    }
}

// To be able to provide just "my-label" or Sprite{..} as an argument for UI widgets
impl<T: Into<WidgetContent>> From<T> for Props {
    fn from(value: T) -> Self {
        Props::new(value.into())
    }
}

impl From<Handle<Image>> for WidgetContent {
    fn from(value: Handle<Image>) -> Self {
        Self::Image(ImageNode::new(value))
    }
}
impl From<ImageNode> for WidgetContent {
    fn from(value: ImageNode) -> Self {
        Self::Image(value)
    }
}
impl From<&'static str> for WidgetContent {
    fn from(value: &'static str) -> Self {
        Self::Text(TextContent {
            text: value.into(),
            ..Default::default()
        })
    }
}
impl From<Cow<'static, str>> for WidgetContent {
    fn from(text: Cow<'static, str>) -> Self {
        Self::Text(TextContent {
            text: Text(text.to_string()),
            ..Default::default()
        })
    }
}
impl From<String> for WidgetContent {
    fn from(value: String) -> Self {
        Self::Text(TextContent {
            text: value.into(),
            ..Default::default()
        })
    }
}
