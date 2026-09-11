//! A single-line text input widget, styled to match this project's [`Props`]/[`colors`]
//! conventions.
//!
//! Built on bevy's own first-party [`EditableText`] widget (`bevy_text`/`bevy_ui_widgets`)
//! rather than a third-party crate: at the time this was written, no external text-input
//! crate had been updated for this bevy version, and bevy itself now ships an editable
//! text field with keyboard/IME/selection handling built in. This module only adds what
//! that widget doesn't provide natively: a placeholder, numeric parsing/clamping/drag-to-adjust,
//! and prefix/suffix decoration.
use super::*;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::Hovered;
use bevy::text::{EditableText, EditableTextFilter, FontSource, TextCursorStyle};
use bevy::ui::widget::TextScroll;
use bevy::ui_widgets::SelectAllOnFocus;
use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};

const INPUT_HEIGHT: f32 = 56.0;
const TEXT_SIZE: f32 = size::FONT_SIZE;
const TEXT_SIZE_SM: f32 = size::FONT_SIZE * 0.6;
const AFFIX_SIZE: f32 = 16.0;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, setup_text_edit_input)
        .add_systems(
            Update,
            (
                handle_focus_style,
                handle_numeric_increment,
                handle_unfocus,
                handle_clamp_on_unfocus,
                handle_drag_value,
                handle_suffix,
                update_placeholder,
                apply_cursor_icon,
                sync_text_edit_display,
                blink_text_cursor,
                position_text_cursor,
            ),
        )
        .add_systems(PostUpdate, apply_default_value);
}

/// `EditableText`'s own glyph layout (parley `PlainEditor`, resolved by family name through
/// `fontique`) never produces any glyphs for this field on this bevy version, confirmed live
/// via BRP: `TextLayoutInfo.size` stays `[0, 0]` even while forcing a layout recompute every
/// frame. So `EditableText` is kept only for its input/focus/selection/IME logic, and the
/// text actually shown is this plain sibling `Text` node kept in sync with its buffer —
/// the same rendering path already used successfully by every other widget in this file.
fn sync_text_edit_display(
    text_edits: Query<&EditableText, With<EditorTextEdit>>,
    mut displays: Query<(&TextEditDisplay, &mut Text)>,
) {
    for (display, mut text) in &mut displays {
        let Ok(editable) = text_edits.get(display.0) else {
            continue;
        };
        let raw = editable.editor().raw_text();
        if text.0 != raw {
            text.0 = raw.to_string();
        }
    }
}

/// Keeps the blinking focus-indicator bar (see [`blink_text_cursor`]) positioned right after
/// the currently displayed text, since it substitutes for `EditableText`'s own (non-functional)
/// cursor rendering.
fn position_text_cursor(
    displays: Query<(&TextEditDisplay, &ComputedNode)>,
    mut cursors: Query<(&TextEditCursor, &mut Node)>,
) {
    for (cursor, mut node) in &mut cursors {
        if let Some((_, computed)) = displays.iter().find(|(d, _)| d.0 == cursor.0) {
            node.left = Px(computed.size().x);
        }
    }
}

/// A plain colored bar (not a text glyph, so it can't be affected by whatever is currently
/// keeping `EditableText` from rendering its own glyphs) that blinks roughly once a second
/// while its associated text-input entity has focus, as a minimal signal that focus/typing
/// is registering even when no text appears.
fn blink_text_cursor(
    // Real time, not virtual: menus that pause `Time<Virtual>` (e.g. the Esc menu) would
    // otherwise freeze this timer's delta at 0 and the bar would stick on its first state.
    time: Res<Time<Real>>,
    mut timer: Local<Option<Timer>>,
    mut on: Local<bool>,
    focus: Res<InputFocus>,
    mut cursors: Query<(&TextEditCursor, &mut Node)>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.53, TimerMode::Repeating));
    if timer.tick(time.delta()).just_finished() {
        *on = !*on;
    }
    for (cursor, mut node) in &mut cursors {
        let focused = focus.get() == Some(cursor.0);
        node.display = if focused && *on {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[derive(Component)]
pub struct EditorTextEdit;

#[derive(Component)]
struct TextEditWrapper(Entity);

#[derive(Component)]
struct TextEditPlaceholder(Entity);

/// Links a blinking focus-indicator bar (see [`blink_text_cursor`]) to the text-input entity
/// it belongs to.
#[derive(Component)]
struct TextEditCursor(Entity);

/// Links the plain `Text` node that actually renders the typed content (see
/// [`sync_text_edit_display`]) to its text-input entity.
#[derive(Component)]
struct TextEditDisplay(Entity);

#[derive(Component, Default, Clone, Copy, PartialEq)]
pub enum TextEditVariant {
    #[default]
    Default,
    NumericF32,
    NumericI32,
}

impl TextEditVariant {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::NumericF32 | Self::NumericI32)
    }
}

#[derive(Clone)]
pub struct TextEditPrefix {
    pub label: String,
    pub size: f32,
}

#[derive(Component)]
struct TextEditSuffix(String);

#[derive(Component)]
struct TextEditSuffixNode(Entity);

#[derive(Component)]
struct TextEditDefaultValue(String);

#[derive(Component, Default)]
struct DragHitbox {
    dragging: bool,
    start_x: f32,
    start_value: f64,
}

#[derive(Component, Clone, Copy)]
struct NumericRange {
    min: f64,
    max: f64,
}

#[derive(Component)]
struct AllowEmpty;

/// Hover/drag cursor icon markers, applied to the primary window by [`apply_cursor_icon`].
#[derive(Component)]
struct HoverCursor(SystemCursorIcon);
#[derive(Component)]
struct ActiveCursor(SystemCursorIcon);

#[derive(Clone)]
pub enum FilterType {
    Decimal,
    Integer,
}

impl FilterType {
    fn allows(&self, c: char) -> bool {
        match self {
            Self::Decimal => c.is_ascii_digit() || c == '.' || c == '-',
            Self::Integer => c.is_ascii_digit() || c == '-',
        }
    }
}

#[derive(Component)]
struct TextEditConfig {
    label: Option<String>,
    variant: TextEditVariant,
    filter: Option<FilterType>,
    prefix: Option<TextEditPrefix>,
    suffix: Option<String>,
    placeholder: String,
    default_value: Option<String>,
    min: f64,
    max: f64,
    allow_empty: bool,
    drag_bottom: bool,
    initialized: bool,
}

pub struct TextEditProps {
    pub label: Option<String>,
    pub placeholder: String,
    pub default_value: Option<String>,
    pub variant: TextEditVariant,
    pub filter: Option<FilterType>,
    pub prefix: Option<TextEditPrefix>,
    pub suffix: Option<String>,
    pub min: f64,
    pub max: f64,
    pub allow_empty: bool,
    pub drag_bottom: bool,
}

impl Default for TextEditProps {
    fn default() -> Self {
        Self {
            label: None,
            placeholder: String::new(),
            default_value: None,
            variant: TextEditVariant::Default,
            filter: None,
            prefix: None,
            suffix: None,
            min: f64::MIN,
            max: f64::MAX,
            allow_empty: false,
            drag_bottom: false,
        }
    }
}

impl TextEditProps {
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    pub fn with_prefix(mut self, prefix: TextEditPrefix) -> Self {
        self.prefix = Some(prefix);
        self
    }
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
    pub fn with_default_value(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
    pub fn with_min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }
    pub fn with_max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }
    pub fn drag_bottom(mut self) -> Self {
        self.drag_bottom = true;
        self
    }
    pub fn numeric_f32(mut self) -> Self {
        self.variant = TextEditVariant::NumericF32;
        self.filter = Some(FilterType::Decimal);
        self.min = f32::MIN as f64;
        self.max = f32::MAX as f64;
        self
    }
    pub fn numeric_i32(mut self) -> Self {
        self.variant = TextEditVariant::NumericI32;
        self.filter = Some(FilterType::Integer);
        self.min = i32::MIN as f64;
        self.max = i32::MAX as f64;
        self
    }
}

pub fn text_edit(props: TextEditProps) -> impl Bundle {
    let TextEditProps {
        label,
        placeholder,
        default_value,
        variant,
        filter,
        prefix,
        suffix,
        min,
        max,
        allow_empty,
        drag_bottom,
    } = props;

    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Px(3.0),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            flex_basis: Px(0.0),
            ..default()
        },
        TextEditConfig {
            label,
            variant,
            filter,
            prefix,
            suffix,
            placeholder,
            default_value,
            min,
            max,
            allow_empty,
            drag_bottom,
            initialized: false,
        },
    )
}

fn setup_text_edit_input(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut configs: Query<(Entity, &mut TextEditConfig)>,
) {
    for (entity, mut config) in &mut configs {
        if config.initialized {
            continue;
        }
        config.initialized = true;

        if let Some(ref label) = config.label {
            let label_entity = commands
                .spawn((
                    Text::new(label),
                    TextFont {
                        font: FontSource::Handle(asset_server.load(fonts::REGULAR)),
                        font_size: FontSize::Px(TEXT_SIZE_SM),
                        weight: FontWeight::MEDIUM,
                        ..default()
                    },
                    TextColor(colors::LIGHT_GRAY_1),
                ))
                .id();
            commands.entity(entity).add_child(label_entity);
        }

        let is_numeric = config.variant.is_numeric();

        let wrapper_entity = commands
            .spawn((
                Node {
                    width: Percent(100.0),
                    height: Px(INPUT_HEIGHT),
                    padding: UiRect::all(Px(6.0)),
                    border: UiRect::all(Px(1.0)),
                    border_radius: BorderRadius::all(size::BORDER_RADIUS),
                    align_items: AlignItems::Center,
                    column_gap: Px(6.0),
                    ..default()
                },
                BackgroundColor(colors::TRANSPARENT),
                BorderColor::all(colors::WHITEISH),
                Hovered::default(),
                HoverCursor(SystemCursorIcon::Text),
            ))
            .id();

        commands.entity(entity).add_child(wrapper_entity);

        if is_numeric && !config.drag_bottom {
            const HITBOX_WIDTH: f32 = INPUT_HEIGHT * 0.9;
            let hitbox = commands
                .spawn((
                    DragHitbox::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Px(HITBOX_WIDTH),
                        height: Px(INPUT_HEIGHT),
                        left: Px(0.0),
                        ..default()
                    },
                    ZIndex(10),
                    Hovered::default(),
                    HoverCursor(SystemCursorIcon::EwResize),
                ))
                .id();
            commands.entity(wrapper_entity).add_child(hitbox);
        }

        if is_numeric && config.drag_bottom {
            let hitbox = commands
                .spawn((
                    DragHitbox::default(),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Percent(50.0),
                        height: Px(INPUT_HEIGHT / 2.0),
                        left: Percent(25.0),
                        top: Px(INPUT_HEIGHT + 6.0),
                        border: UiRect::all(Px(1.0)),
                        ..default()
                    },
                    ZIndex(50),
                    Hovered::default(),
                    HoverCursor(SystemCursorIcon::EwResize),
                ))
                .id();
            commands.entity(wrapper_entity).add_child(hitbox);
        }

        if let Some(ref prefix) = config.prefix {
            let prefix_entity = commands
                .spawn((
                    Text::new(prefix.label.clone()),
                    TextFont {
                        font: FontSource::Handle(asset_server.load(fonts::REGULAR)),
                        font_size: FontSize::Px(prefix.size),
                        ..default()
                    },
                    TextColor(colors::WHITE.with_alpha(0.5)),
                    TextLayout::justify(Justify::Center),
                    Node {
                        width: Px(AFFIX_SIZE),
                        ..default()
                    },
                ))
                .id();
            commands.entity(wrapper_entity).add_child(prefix_entity);
        }

        let placeholder_entity = commands
            .spawn((
                Text::new(config.placeholder.clone()),
                TextFont {
                    font: FontSource::Handle(asset_server.load(fonts::REGULAR)),
                    font_size: FontSize::Px(TEXT_SIZE),
                    ..default()
                },
                TextColor(colors::WHITE.with_alpha(0.2)),
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        commands
            .entity(wrapper_entity)
            .add_child(placeholder_entity);

        let initial_text = config.default_value.clone().unwrap_or_default();

        let mut text_input = commands.spawn((
            EditorTextEdit,
            config.variant,
            SelectAllOnFocus,
            EditableText::new(&initial_text),
            // This is a single-line field: word-wrapping (the `TextLayout` default) makes no
            // sense here, and depends on a wrap width derived from the node's computed layout
            // that can transiently be zero while a deeply-nested flex hierarchy resolves,
            // wrapping every character onto its own empty line.
            TextLayout::no_wrap(),
            TextCursorStyle::default(),
            // `bevy_ui_widgets`' own click-to-focus observer queries this non-optionally; without
            // it the query fails and the observer returns before ever calling `InputFocus::set`,
            // so clicking the field never actually focuses it.
            TextScroll::default(),
            TextFont {
                font: FontSource::Handle(asset_server.load(fonts::REGULAR)),
                font_size: FontSize::Px(TEXT_SIZE),
                ..default()
            },
            TextColor(colors::WHITE),
            Node {
                flex_grow: 1.0,
                height: Percent(100.0),
                overflow: Overflow::clip(),
                ..default()
            },
            TextEditPlaceholder(placeholder_entity),
        ));

        if let Some(ref filter) = config.filter {
            let filter = filter.clone();
            text_input.insert(EditableTextFilter::new(move |c| filter.allows(c)));
        }

        if let Some(ref suffix) = config.suffix {
            text_input.insert(TextEditSuffix(suffix.clone()));
        }

        if config.default_value.is_some() && is_numeric {
            // Route the initial value through the same clamp/format path as user edits.
            text_input.insert(TextEditDefaultValue(initial_text.clone()));
        }

        if is_numeric {
            text_input.insert(NumericRange {
                min: config.min,
                max: config.max,
            });
        }

        if config.allow_empty {
            text_input.insert(AllowEmpty);
        }

        let text_input_entity = text_input.id();

        commands.entity(wrapper_entity).add_child(text_input_entity);

        let display_entity = commands
            .spawn((
                TextEditDisplay(text_input_entity),
                Text::new(initial_text.clone()),
                TextFont {
                    font: FontSource::Handle(asset_server.load(fonts::REGULAR)),
                    font_size: FontSize::Px(TEXT_SIZE),
                    ..default()
                },
                TextColor(colors::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Px(0.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(text_input_entity).add_child(display_entity);

        let cursor_entity = commands
            .spawn((
                TextEditCursor(text_input_entity),
                Node {
                    position_type: PositionType::Absolute,
                    left: Px(0.0),
                    top: Percent(15.0),
                    width: Px(2.0),
                    height: Percent(70.0),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(colors::WHITE),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(text_input_entity).add_child(cursor_entity);

        if let Some(ref suffix) = config.suffix {
            let suffix_entity = commands
                .spawn((
                    TextEditSuffixNode(text_input_entity),
                    Text::new(suffix.clone()),
                    TextFont {
                        font: FontSource::Handle(asset_server.load(fonts::REGULAR)),
                        font_size: FontSize::Px(TEXT_SIZE),
                        ..default()
                    },
                    TextColor(colors::LIGHT_GRAY_1),
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id();
            commands.entity(wrapper_entity).add_child(suffix_entity);
        }

        commands
            .entity(wrapper_entity)
            .insert(TextEditWrapper(text_input_entity));
    }
}

/// `EditableText` has no built-in placeholder (it's on bevy's own "not yet implemented" list),
/// so this shows/hides a sibling text node based on whether the buffer is empty.
fn update_placeholder(
    text_edits: Query<(&EditableText, &TextEditPlaceholder), With<EditorTextEdit>>,
    mut placeholders: Query<&mut Node>,
) {
    for (editable, placeholder) in &text_edits {
        if let Ok(mut node) = placeholders.get_mut(placeholder.0) {
            node.display = if editable.editor().raw_text().is_empty() {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

fn apply_default_value(
    mut commands: Commands,
    mut text_edits: Query<(
        Entity,
        &TextEditDefaultValue,
        &TextEditVariant,
        &mut EditableText,
        Option<&NumericRange>,
    )>,
) {
    for (entity, default_value, variant, mut editable, range) in &mut text_edits {
        let value = clamp_value(default_value.0.parse().unwrap_or(0.0), range);
        set_text_value(&mut editable, &format_numeric_value(value, *variant));
        commands.entity(entity).remove::<TextEditDefaultValue>();
    }
}

fn handle_suffix(
    focus: Res<InputFocus>,
    text_edits: Query<(Entity, &EditableText, &ChildOf), With<TextEditSuffix>>,
    mut suffix_nodes: Query<(&TextEditSuffixNode, &mut Node), Without<TextEditWrapper>>,
) {
    for (entity, editable, _child_of) in &text_edits {
        let Some((_, mut node)) = suffix_nodes.iter_mut().find(|(link, _)| link.0 == entity) else {
            continue;
        };

        let show = focus.get() != Some(entity) && !editable.editor().raw_text().is_empty();
        node.display = if show { Display::Flex } else { Display::None };
    }
}

fn handle_focus_style(
    focus: Res<InputFocus>,
    mut wrappers: Query<(&TextEditWrapper, &mut BorderColor, &Hovered)>,
) {
    for (wrapper, mut border_color, hovered) in &mut wrappers {
        let color = match (focus.get() == Some(wrapper.0), hovered.get()) {
            (true, _) => colors::BRIGHT_BLUE,
            (_, true) => colors::WHITEISH.lighter(0.05),
            _ => colors::WHITEISH,
        };
        *border_color = BorderColor::all(color);
    }
}

fn handle_unfocus(
    mut focus: ResMut<InputFocus>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    text_edits: Query<&ChildOf, With<EditorTextEdit>>,
    wrappers: Query<&Hovered, With<TextEditWrapper>>,
) {
    let Some(focused_entity) = focus.get() else {
        return;
    };
    let Ok(child_of) = text_edits.get(focused_entity) else {
        return;
    };
    let Ok(hovered) = wrappers.get(child_of.parent()) else {
        return;
    };

    let clicked_outside = mouse.get_just_pressed().next().is_some() && !hovered.get();
    let key_dismiss = keyboard.just_pressed(KeyCode::Escape)
        || keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::NumpadEnter);

    if clicked_outside || key_dismiss {
        focus.clear();
    }
}

fn handle_clamp_on_unfocus(
    mut commands: Commands,
    focus: Res<InputFocus>,
    mut prev_focus: Local<Option<Entity>>,
    mut text_edits: Query<
        (
            &TextEditVariant,
            &mut EditableText,
            Option<&TextEditSuffix>,
            Option<&NumericRange>,
            Option<&AllowEmpty>,
        ),
        With<EditorTextEdit>,
    >,
) {
    let prev = *prev_focus;
    *prev_focus = focus.get();

    let Some(was_focused) = prev else { return };
    if focus.get() == Some(was_focused) {
        return;
    }

    let Ok((variant, mut editable, suffix, range, allow_empty)) = text_edits.get_mut(was_focused)
    else {
        return;
    };

    let text = strip_suffix(editable.editor().raw_text(), suffix);

    commands.trigger(TextEditCommitEvent {
        entity: was_focused,
        text: text.clone(),
    });

    if !variant.is_numeric() {
        return;
    }

    if text.is_empty() && allow_empty.is_some() {
        return;
    }

    let value = text.parse().unwrap_or(0.0);
    update_input_value(&mut editable, value, *variant, range);
}

fn handle_numeric_increment(
    focus: Res<InputFocus>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut text_edits: Query<
        (
            &TextEditVariant,
            &mut EditableText,
            Option<&TextEditSuffix>,
            Option<&NumericRange>,
        ),
        With<EditorTextEdit>,
    >,
) {
    let Some(focused_entity) = focus.get() else {
        return;
    };
    let Ok((variant, mut editable, suffix, range)) = text_edits.get_mut(focused_entity) else {
        return;
    };
    if !variant.is_numeric() {
        return;
    }

    let direction = match (
        keyboard.just_pressed(KeyCode::ArrowUp),
        keyboard.just_pressed(KeyCode::ArrowDown),
    ) {
        (true, _) => 1.0,
        (_, true) => -1.0,
        _ => return,
    };

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let step = if shift { 10.0 } else { 1.0 };
    let new_value = parse_numeric_value(editable.editor().raw_text(), suffix) + (direction * step);
    let rounded = (new_value * 100.0).round() / 100.0;

    update_input_value(&mut editable, rounded, *variant, range);
}

fn handle_drag_value(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut drag_hitboxes: Query<(Entity, &mut DragHitbox, &Hovered, &ChildOf)>,
    wrappers: Query<&TextEditWrapper>,
    mut text_edits: Query<
        (
            &TextEditVariant,
            &mut EditableText,
            Option<&TextEditSuffix>,
            Option<&NumericRange>,
        ),
        With<EditorTextEdit>,
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let cursor_pos = window.cursor_position();

    for (entity, mut hitbox, hovered, child_of) in &mut drag_hitboxes {
        let Ok(wrapper) = wrappers.get(child_of.parent()) else {
            continue;
        };
        let input_entity = wrapper.0;

        if mouse.just_pressed(MouseButton::Left) && hovered.get() {
            if let Some(pos) = cursor_pos {
                let Ok((_, editable, suffix, _)) = text_edits.get(input_entity) else {
                    continue;
                };
                hitbox.dragging = true;
                hitbox.start_x = pos.x;
                hitbox.start_value = parse_numeric_value(editable.editor().raw_text(), suffix);
                commands
                    .entity(entity)
                    .insert(ActiveCursor(SystemCursorIcon::EwResize));
            }
        }

        if mouse.just_released(MouseButton::Left) {
            if hitbox.dragging {
                if let Ok((_, editable, suffix, _)) = text_edits.get(input_entity) {
                    let text = strip_suffix(editable.editor().raw_text(), suffix);
                    commands.trigger(TextEditCommitEvent {
                        entity: input_entity,
                        text,
                    });
                }
            }
            hitbox.dragging = false;
            commands.entity(entity).remove::<ActiveCursor>();
        }

        if hitbox.dragging
            && let Some(pos) = cursor_pos
        {
            let Ok((variant, mut editable, _, range)) = text_edits.get_mut(input_entity) else {
                continue;
            };

            let alt_mode = keyboard.pressed(KeyCode::SuperLeft)
                || keyboard.pressed(KeyCode::SuperRight)
                || keyboard.pressed(KeyCode::AltLeft)
                || keyboard.pressed(KeyCode::AltRight);

            let (amount, sensitivity) = match (*variant, alt_mode) {
                (TextEditVariant::NumericI32, false) => (1.0, 5.0),
                (TextEditVariant::NumericI32, true) => (10.0, 10.0),
                (_, false) => (0.1, 5.0),
                (_, true) => (1.0, 10.0),
            };

            let steps = ((pos.x - hitbox.start_x) / sensitivity).floor() as f64;
            let new_value = hitbox.start_value + (steps * amount);
            let rounded = (new_value * 100.0).round() / 100.0;

            update_input_value(&mut editable, rounded, *variant, range);
        }
    }
}

/// Applies pressed cursor icons first, then hovered ones, else resets to the platform default.
fn apply_cursor_icon(
    mut commands: Commands,
    window: Query<Entity, With<PrimaryWindow>>,
    active: Query<&ActiveCursor>,
    hovered: Query<(&HoverCursor, &Hovered)>,
) {
    let Ok(window) = window.single() else {
        return;
    };

    let icon = active
        .iter()
        .next()
        .map(|a| a.0)
        .or_else(|| hovered.iter().find(|(_, h)| h.get()).map(|(c, _)| c.0));

    match icon {
        Some(icon) => {
            commands.entity(window).insert(CursorIcon::System(icon));
        }
        None => {
            commands.entity(window).remove::<CursorIcon>();
        }
    }
}

fn strip_suffix(text: &str, suffix: Option<&TextEditSuffix>) -> String {
    suffix
        .and_then(|s| text.strip_suffix(&format!(" {}", s.0)))
        .unwrap_or(text)
        .to_string()
}

fn parse_numeric_value(text: &str, suffix: Option<&TextEditSuffix>) -> f64 {
    strip_suffix(text, suffix).parse().unwrap_or(0.0)
}

fn format_numeric_value(value: f64, variant: TextEditVariant) -> String {
    match variant {
        TextEditVariant::NumericI32 => (value.round() as i32).to_string(),
        TextEditVariant::NumericF32 => {
            let mut text = value.to_string();
            if !text.contains('.') {
                text.push_str(".0");
            }
            text
        }
        TextEditVariant::Default => value.to_string(),
    }
}

fn clamp_value(value: f64, range: Option<&NumericRange>) -> f64 {
    match range {
        Some(r) => value.clamp(r.min, r.max),
        None => value,
    }
}

fn set_text_value(editable: &mut EditableText, text: &str) {
    editable.editor_mut().set_text(text);
    editable.queue_edit(bevy::text::TextEdit::TextEnd(false));
}

fn update_input_value(
    editable: &mut EditableText,
    value: f64,
    variant: TextEditVariant,
    range: Option<&NumericRange>,
) {
    let clamped = clamp_value(value, range);
    set_text_value(editable, &format_numeric_value(clamped, variant));
}

#[derive(EntityEvent)]
pub struct TextEditCommitEvent {
    pub entity: Entity,
    pub text: String,
}
