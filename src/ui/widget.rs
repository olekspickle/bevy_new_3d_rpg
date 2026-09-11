use super::*;
use bevy::ecs::system::IntoObserverSystem;
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::ui::Checked;
use bevy::ui_widgets::{
    Checkbox, Slider, SliderRange, SliderStep, SliderValue, ValueChange, slider_self_update,
};
use std::borrow::Cow;
use std::ops::RangeInclusive;

markers!(CheckboxMark, SliderFill);

/// Registers the internal wiring observers for [`checkbox`]/[`slider`] globally, once.
///
/// These must NOT be per-entity `Observer` components: a bundle can only hold one
/// component of a given type, so a per-entity `Observer::new(checkbox_visual_update)`
/// alongside the caller's own `Observer::new(action)` panics at spawn time with
/// "duplicate components: Observer". Both `checkbox_visual_update` and
/// `slider_self_update` key off the event's `source`/`entity` field already, so a
/// global observer works identically to a local one here (this is also how
/// `bevy_ui_widgets`' own `SliderPlugin`/`CheckboxPlugin` register their internal
/// observers).
pub fn plugin(app: &mut App) {
    app.add_observer(checkbox_visual_update)
        .add_observer(slider_self_update);
}

/// Carries the initial `checked` state for a [`checkbox`] into an `on_add` hook, since
/// `Option<Checked>` can't be spliced into a fixed-shape bundle tuple directly.
#[derive(Component)]
#[component(on_add = InitialChecked::on_add)]
struct InitialChecked(bool);

impl InitialChecked {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        if world.get::<InitialChecked>(ctx.entity).is_some_and(|c| c.0) {
            world.commands().entity(ctx.entity).insert(Checked);
        }
    }
}

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        GlobalZIndex(-1), // to see bevy-egui
        Name::new(name),
        Node {
            width: Percent(100.0),
            height: Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Vh(5.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

pub fn icon(opts: impl Into<Props>) -> impl Bundle {
    let opts = opts.into();
    (
        Label,
        Name::new("Icon"),
        opts.node.clone(),
        opts.into_image_bundle(),
    )
}
pub fn label(opts: impl Into<Props>) -> impl Bundle {
    let opts = opts.into();
    (
        Label,
        Name::new("Label"),
        opts.node.clone(),
        opts.into_text_bundle(),
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub fn header(opts: impl Into<Props>) -> impl Bundle {
    let opts = opts.into();
    (Label, Name::new("Header"), opts.into_text_bundle())
}

// A regular wide button with text and an action defined as an [`Observer`].
pub fn btn_big<E, B, M, I>(opts: impl Into<Props>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let opts: Props = opts.into();
    let new_node = Node {
        min_width: Vw(30.0),
        padding: UiRect::axes(Vw(8.0), Vh(2.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..opts.node.clone()
    };
    let opts = opts.node(new_node);

    btn(opts, action)
}

// A small square button with text and an action defined as an [`Observer`].
pub fn btn_small<E, B, M, I>(opts: impl Into<Props>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let opts: Props = opts.into();
    let new_node = Node {
        margin: UiRect::ZERO,
        padding: UiRect::horizontal(Vw(1.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(Px(7.0)),
        ..opts.node.clone()
    };

    btn(opts.node(new_node), action)
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
/// Background color is set by [`PaletteSet`]
pub fn btn<E, B, M, I>(opts: impl Into<Props>, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let mut opts: Props = opts.into();
    let action = IntoObserverSystem::into_system(action);

    (
        Button,
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            let content = match &opts.content {
                WidgetContent::Image(_) => parent
                    .spawn((opts.clone().into_image_bundle(), Pickable::IGNORE))
                    .id(),
                WidgetContent::Text(_) => parent
                    .spawn((opts.clone().into_text_bundle(), Pickable::IGNORE))
                    .id(),
            };
            opts.node.width = Percent(100.0);
            opts.node.height = Percent(100.0);

            parent
                .spawn((
                    opts.bg_color,
                    opts.border_color,
                    opts.palette_set,
                    Name::new("Button Content"),
                ))
                .insert(opts.node)
                .add_children(&[content])
                .observe(action);
        })),
    )
}

/// A headless `bevy_ui_widgets::Checkbox` styled with this project's own [`PaletteSet`]
/// hover/press feedback (not `bevy_feathers`). `action` is called with the new boolean value
/// whenever the checkbox is toggled (click or keyboard); the checkmark visual and the
/// `Checked` component are kept in sync internally.
pub fn checkbox<E, B, M, I>(checked: bool, action: I) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    // Spawning the interactive entity as a child and attaching `action` via `.observe(...)`
    // scopes the observer to that one entity, same as `btn()` below.
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Checkbox"),
        Node {
            width: size::CHECKBOX_SIZE,
            height: size::CHECKBOX_SIZE,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Checkbox,
                    InitialChecked(checked),
                    Name::new("Checkbox Interactive"),
                    Node {
                        width: Percent(100.0),
                        height: Percent(100.0),
                        border: UiRect::all(Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(colors::TRANSPARENT),
                    BorderColor::all(colors::WHITEISH),
                    PaletteSet::default(),
                    Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                        parent.spawn((
                            CheckboxMark,
                            Node {
                                width: Percent(60.0),
                                height: Percent(60.0),
                                display: if checked {
                                    Display::Flex
                                } else {
                                    Display::None
                                },
                                ..default()
                            },
                            BackgroundColor(colors::WHITE),
                            Pickable::IGNORE,
                        ));
                    })),
                ))
                .observe(action);
        })),
    )
}

/// Keeps a [`checkbox`]'s [`Checked`] component and checkmark visual in sync with the
/// widget's own `ValueChange<bool>` (fired by `bevy_ui_widgets` on click/keyboard toggle).
fn checkbox_visual_update(
    on: On<ValueChange<bool>>,
    children_q: Query<&Children>,
    mut mark_q: Query<&mut Node, With<CheckboxMark>>,
    mut commands: Commands,
) {
    if on.value {
        commands.entity(on.source).insert(Checked);
    } else {
        commands.entity(on.source).remove::<Checked>();
    }
    if let Ok(children) = children_q.get(on.source) {
        for child in children.iter() {
            if let Ok(mut node) = mark_q.get_mut(child) {
                node.display = if on.value {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

/// A headless `bevy_ui_widgets::Slider` (drag/click/keyboard-driven) paired with a label
/// showing the current value, styled with this project's own [`PaletteSet`] (not
/// `bevy_feathers`). `action` is called with the new value on every change, including live
/// updates while dragging (`ValueChange::is_final == false`).
pub fn slider<E, B, M, I>(
    label_marker: impl Component,
    range: RangeInclusive<f32>,
    step: f32,
    value: f32,
    action: I,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let slider_range = SliderRange::from_range(range);
    let fill_percent = (slider_range.thumb_position(value) * 100.0).clamp(0.0, 100.0);
    // Spawning the interactive entity as a child and attaching `action` via `.observe(...)`
    // scopes the observer to that one entity, same as `btn()` below.
    let action = IntoObserverSystem::into_system(action);
    (
        Node {
            justify_self: JustifySelf::Start,
            align_items: AlignItems::Center,
            column_gap: Px(10.0),
            ..default()
        },
        children![
            (
                Name::new("Slider"),
                Node {
                    width: Vw(10.0),
                    height: size::ROW_HEIGHT,
                    ..default()
                },
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    parent
                        .spawn((
                            Slider::default(),
                            slider_range,
                            SliderStep(step),
                            SliderValue(value),
                            Name::new("Slider Interactive"),
                            Node {
                                width: Percent(100.0),
                                height: Percent(100.0),
                                border: UiRect::all(Px(2.0)),
                                padding: UiRect::all(Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(colors::TRANSPARENT),
                            BorderColor::all(colors::WHITEISH),
                            // Background stays transparent in every state; only the border
                            // reacts to hover/press, so the fill bar remains the sole
                            // indicator of the slider's value.
                            PaletteSet {
                                none: Palette::new(
                                    colors::WHITE,
                                    colors::TRANSPARENT,
                                    BorderColor::all(colors::WHITEISH),
                                ),
                                hovered: Palette::new(
                                    colors::WHITE,
                                    colors::TRANSPARENT,
                                    BorderColor::all(colors::BRIGHT_BLUE),
                                ),
                                pressed: Palette::new(
                                    colors::WHITE,
                                    colors::TRANSPARENT,
                                    BorderColor::all(colors::BRIGHT_BLUE),
                                ),
                                disabled: Palette::new(
                                    colors::TRANSPARENT,
                                    colors::TRANSPARENT,
                                    BorderColor::all(colors::WARM_GRAY_1),
                                ),
                            },
                            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                                parent.spawn((
                                    SliderFill,
                                    Node {
                                        height: Percent(100.0),
                                        width: Percent(fill_percent),
                                        ..default()
                                    },
                                    BackgroundColor(colors::BRIGHT_BLUE),
                                    Pickable::IGNORE,
                                ));
                            })),
                        ))
                        .observe(action);
                })),
            ),
            (label(""), label_marker),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::World;

    markers!(TestSliderLabel);

    fn noop_bool_action(_: On<ValueChange<bool>>) {}
    fn noop_f32_action(_: On<ValueChange<f32>>) {}

    /// A bundle can only hold one component of a given type, so `checkbox`/`slider`
    /// must never put more than one `Observer` in their returned bundle. Actually spawn
    /// both into a real `World` rather than just type-checking, since a duplicate
    /// component panics only at runtime bundle-validation, not at compile time.
    #[test]
    fn checkbox_spawns_without_duplicate_observer_panic() {
        let mut world = World::new();
        world.spawn(checkbox(false, noop_bool_action));
    }

    #[test]
    fn slider_spawns_without_duplicate_observer_panic() {
        let mut world = World::new();
        world.spawn(slider(
            TestSliderLabel,
            0.0..=1.0,
            0.1,
            0.5,
            noop_f32_action,
        ));
    }

    /// Each checkbox's `action` observer is scoped to that checkbox's own entity, so
    /// triggering `ValueChange` on one checkbox only invokes that checkbox's action.
    #[test]
    fn checkbox_action_only_fires_for_its_own_entity() {
        use bevy::ecs::world::CommandQueue;

        #[derive(Resource, Default)]
        struct Fired(Vec<&'static str>);

        fn action_a(_: On<ValueChange<bool>>, mut fired: ResMut<Fired>) {
            fired.0.push("a");
        }
        fn action_b(_: On<ValueChange<bool>>, mut fired: ResMut<Fired>) {
            fired.0.push("b");
        }

        fn find_checkbox_child(world: &World, root: Entity) -> Entity {
            world
                .get::<Children>(root)
                .expect("checkbox root should have spawned a child")
                .iter()
                .find(|&child| world.get::<Checkbox>(child).is_some())
                .expect("checkbox child not found")
        }

        let mut world = World::new();
        world.insert_resource(Fired::default());

        let root_a = world.spawn(checkbox(false, action_a)).id();
        let root_b = world.spawn(checkbox(false, action_b)).id();
        world.flush();

        let checkbox_a = find_checkbox_child(&world, root_a);
        let _checkbox_b = find_checkbox_child(&world, root_b);

        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        commands.trigger(ValueChange {
            source: checkbox_a,
            value: true,
            is_final: true,
        });
        queue.apply(&mut world);

        assert_eq!(
            world.resource::<Fired>().0,
            vec!["a"],
            "only the clicked checkbox's own action should fire"
        );
    }
}
