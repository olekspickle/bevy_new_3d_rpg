use super::*;
use std::borrow::Cow;

#[derive(Debug, Clone, Bundle)]
pub struct Opts {
    pub inner: WidgetContent,
    // layout
    pub border_radius: BorderRadius,
    pub border_color: BorderColor,
    pub bg_color: BackgroundColor,
    pub node: Node,
    pub ui_palette: UiInteraction,
}

#[allow(dead_code)]
impl Opts {
    pub fn new(c: impl Into<WidgetContent>) -> Self {
        Self {
            inner: c.into(),
            node: Node {
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                justify_items: JustifyItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Px(2.0)),
                padding: UiRect::horizontal(Vw(3.0)),
                ..Default::default()
            },
            ui_palette: UiInteraction::DEFAULT,
            bg_color: BackgroundColor(TRANSPARENT),
            border_color: BorderColor(WHITEISH),
            border_radius: BorderRadius::all(Px(BORDER_RADIUS)),
        }
    }

    pub fn image(mut self, s: Handle<Image>) -> Self {
        self.inner = WidgetContent::Image(ImageNode::new(s));
        self
    }
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        match self.inner {
            WidgetContent::Text(ref mut t) => {
                t.text = Text(text.into().to_string());
            }
            _ => self.inner = WidgetContent::Text(text.into().into()),
        }
        self
    }
    pub fn font(mut self, font: TextFont) -> Self {
        if let WidgetContent::Text(ref mut t) = self.inner {
            t.font = font;
        }
        self
    }
    pub fn font_size(mut self, s: f32) -> Self {
        if let WidgetContent::Text(ref mut t) = self.inner {
            t.font.font_size = s;
        }
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        if let WidgetContent::Text(ref mut t) = self.inner {
            *t.color = c;
        }
        self
    }
    pub fn bg_color(mut self, color: Color) -> Self {
        self.bg_color = BackgroundColor(color);
        self
    }
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = BorderColor(color);
        self
    }
    pub fn border_radius(mut self, r: Val) -> Self {
        self.border_radius = BorderRadius::all(r);
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
    pub fn ui_palette(mut self, p: UiInteraction) -> Self {
        self.ui_palette = p;
        self
    }
    // TODO: do a mesh2d ui bundle
    pub fn into_image_bundle(self) -> impl Bundle {
        match &self.inner {
            WidgetContent::Image(c) => ImageWidgetBundle {
                image: c.clone(),
                node: Node {
                    position_type: PositionType::Absolute,
                    width: Percent(100.0),
                    height: Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
            },
            _ => unreachable!("Spawning image bundle on non image content"),
        }
    }
    pub fn into_text_bundle(self) -> impl Bundle {
        match &self.inner {
            WidgetContent::Text(c) => (c.clone(), self.bg_color),
            _ => unreachable!("Spawning text bundle on non text content"),
        }
    }
}

impl Default for Opts {
    fn default() -> Self {
        Opts::new("")
    }
}

#[derive(Bundle)]
pub struct ImageWidgetBundle {
    node: Node,
    image: ImageNode,
}

#[derive(Debug, Clone, Bundle)]
pub struct TextContent {
    pub text: Text,
    pub color: TextColor,
    pub layout: TextLayout,
    pub font: TextFont,
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
            color: WHITEISH.into(),
            layout: TextLayout::new_with_justify(JustifyText::Center),
            font: TextFont::from_font_size(FONT_SIZE),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub enum WidgetContent {
    Image(ImageNode),
    Text(TextContent),
}

// To be able to provide just "my-label" or Sprite{..} as an argument for UI widgets
impl<T: Into<WidgetContent>> From<T> for Opts {
    fn from(value: T) -> Self {
        Opts::new(value.into())
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
