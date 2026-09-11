use super::*;
use crate::shared::*;
use bevy::ecs::spawn::SpawnIter;
use bevy_enhanced_input::prelude::*;
use strum::IntoEnumIterator;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RebindCapture>()
        .add_systems(Update, (capture_binding, update_capture_hint).chain());
}

markers!(KeybindHint);

/// Which binding slot (of up to [`BINDINGS_COUNT`]) of which [`KeybindAction`] is currently
/// waiting for the next key/mouse press, if any.
#[derive(Resource, Default)]
struct RebindCapture {
    slot: Option<(KeybindAction, usize)>,
    /// Set one frame after `slot` starts capturing, so the same click that opened capture
    /// mode isn't immediately read back as the new binding.
    armed: bool,
}

/// Marks a rebind button with the action/slot it edits.
#[derive(Component, Clone, Copy)]
struct BindingSlot {
    action: KeybindAction,
    index: usize,
}

pub fn keybind_editor(input_map: &InputSettings) -> impl Bundle {
    // Resolved eagerly (into owned bundles, no borrow of `input_map` retained) so the
    // `SpawnIter` below doesn't need to capture a non-'static reference.
    let rows: Vec<_> = KeybindAction::iter()
        .map(|action| action_row(action, input_map))
        .collect();
    (
        Name::new("Keybind Editor"),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Px(10.0),
            ..default()
        },
        Children::spawn((
            Spawn((widget::label(""), KeybindHint)),
            SpawnIter(rows.into_iter()),
        )),
    )
}

fn action_row(action: KeybindAction, input_map: &InputSettings) -> impl Bundle {
    let bindings = action.bindings(input_map);
    (
        Name::new("Keybind Row"),
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(4, 150.0),
            column_gap: Px(10.0),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            widget::label(<&str>::from(action)),
            binding_slot_btn(action, 0, bindings[0]),
            binding_slot_btn(action, 1, bindings[1]),
            binding_slot_btn(action, 2, bindings[2]),
        ],
    )
}

fn binding_slot_btn(action: KeybindAction, index: usize, binding: Binding) -> impl Bundle {
    (
        widget::btn_small(binding.to_string(), begin_rebind(action, index)),
        BindingSlot { action, index },
    )
}

fn begin_rebind(
    action: KeybindAction,
    index: usize,
) -> impl Fn(On<Pointer<Click>>, ResMut<RebindCapture>) + Clone {
    move |_: On<Pointer<Click>>, mut capture: ResMut<RebindCapture>| {
        capture.slot = Some((action, index));
        capture.armed = false;
    }
}

fn update_capture_hint(capture: Res<RebindCapture>, mut hint: Query<&mut Text, With<KeybindHint>>) {
    let Ok(mut text) = hint.single_mut() else {
        return;
    };
    text.0 = match capture.slot {
        Some((action, index)) => format!(
            "Press any key to bind \"{}\" (slot {}) \u{2014} Esc to cancel",
            <&str>::from(action),
            index + 1
        ),
        None => String::new(),
    };
}

fn capture_binding(
    mut capture: ResMut<RebindCapture>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut settings: ResMut<Settings>,
    children_q: Query<&Children>,
    mut commands: Commands,
    slots: Query<(Entity, &BindingSlot)>,
) {
    let Some((action, index)) = capture.slot else {
        return;
    };

    // Skip the frame the slot was clicked on, so that click isn't read back as the binding.
    if !capture.armed {
        capture.armed = true;
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        capture.slot = None;
        return;
    }

    let Some(binding) = keyboard
        .get_just_pressed()
        .next()
        .map(|&key| Binding::from(key))
        .or_else(|| {
            mouse
                .get_just_pressed()
                .next()
                .map(|&button| Binding::from(button))
        })
    else {
        return;
    };

    action.bindings_mut(&mut settings.input_map)[index] = binding;
    capture.slot = None;

    let Some((mut entity, _)) = slots
        .iter()
        .find(|(_, slot)| slot.action == action && slot.index == index)
    else {
        return;
    };

    commands.trigger(SettingsChanged);
    entity.replace_recursive(
        children_q,
        commands,
        widget::btn_small(binding.to_string(), begin_rebind(action, index)),
    );
}
