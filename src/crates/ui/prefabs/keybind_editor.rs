use super::*;
use bevy::{
    ecs::{
        relationship::RelatedSpawner,
        spawn::{SpawnWith, SpawnableList},
    },
    input::{ButtonState, common_conditions::*, keyboard::KeyboardInput, mouse::MouseButtonInput},
    ui::FocusPolicy,
};
use std::fmt::Write;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_button_text,
            (
                cancel_binding.run_if(input_just_pressed(KeyCode::Escape)),
                bind,
            )
                .chain(),
        ),
    );
}

pub fn keybind_editor(keybind: &Keybind) -> impl Bundle {
    // We use separate root node to let dialogs cover the whole UI.
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Vw(1.0)),
                row_gap: Vw(1.0),
                ..Default::default()
            },
            children![
                actions_grid(keybind.clone()),
                (
                    Node {
                        align_items: AlignItems::End,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::End,
                        ..Default::default()
                    },
                    children![(
                        SettingsButton,
                        Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<_>| {
                            parent.spawn(Text::new("Apply")).observe(apply);
                        }))
                    )],
                )
            ]
        )],
    )
}

/// Creates [`SettingsField`] from passed field.
///
/// Strips everything before first `.` in order to turn "settings.field_name" into just "field_name".
macro_rules! settings_field {
    ($path:expr) => {{
        let _validate_field = &$path;
        let full_path = stringify!($path);
        let field_name = full_path
            .split_once('.')
            .map(|(_, s)| s)
            .unwrap_or(full_path);
        SettingsField(field_name)
    }};
}

/// Stores name of the [`Settings`] field.
///
/// Used to utilize reflection when applying settings.
#[derive(Component, Clone, Copy)]
struct SettingsField(&'static str);

/// Number of input columns.
const INPUTS_PER_ACTION: usize = 3;

fn actions_grid(keybind: Keybind) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            column_gap: Vw(1.0),
            row_gap: Vw(1.0),
            grid_template_columns: vec![GridTrack::auto(); INPUTS_PER_ACTION + 1],
            ..Default::default()
        },
        // We could utilzie reflection to iterate over fields,
        // but in real application you most likely want to have a nice and translatable text on buttons.
        Children::spawn((
            action_row("Forward", settings_field!(keybind.forward), keybind.forward),
            action_row("Left", settings_field!(keybind.left), keybind.left),
            action_row(
                "Backward",
                settings_field!(keybind.backward),
                keybind.backward,
            ),
            action_row("Right", settings_field!(keybind.right), keybind.right),
            action_row("Jump", settings_field!(keybind.jump), keybind.jump),
            action_row("Crouch", settings_field!(keybind.crouch), keybind.crouch),
            action_row("Dash", settings_field!(keybind.dash), keybind.dash),
            action_row("Sprint", settings_field!(keybind.sprint), keybind.sprint),
        )),
    )
}

fn action_row(
    name: &'static str,
    field: SettingsField,
    inputs: Vec<Input>,
) -> impl SpawnableList<ChildOf> {
    (
        Spawn(Text::new(name)),
        SpawnWith(move |parent: &mut RelatedSpawner<_>| {
            for index in 0..INPUTS_PER_ACTION {
                let input = inputs.get(index).copied();
                parent.spawn((
                    Node {
                        column_gap: Vw(1.0),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<_>| {
                        let button_entity = parent
                            .spawn((
                                field,
                                Name::new(name),
                                InputButton { input },
                                children![Text::default()], // Will be updated automatically on `InputButton` insertion
                            ))
                            .observe(show_binding_dialog)
                            .id();
                        parent
                            .spawn((DeleteButton { button_entity }, children![Text::new("X")]))
                            .observe(delete_binding);
                    })),
                ));
            }
        }),
    )
}

fn delete_binding(
    trigger: Trigger<Pointer<Click>>,
    mut input_buttons: Query<(&Name, &mut InputButton)>,
    delete_buttons: Query<&DeleteButton>,
) {
    let delete_button = delete_buttons.get(trigger.target()).unwrap();
    let (name, mut input_button) = input_buttons
        .get_mut(delete_button.button_entity)
        .expect("delete button should point to an input button");
    info!("deleting binding for '{name}'");
    input_button.input = None;
}

fn show_binding_dialog(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    root_entity: Single<Entity, (With<Node>, Without<ChildOf>)>,
    names: Query<&Name>,
) {
    let name = names.get(trigger.target()).unwrap();
    info!("starting binding for '{name}'");

    commands.entity(*root_entity).with_child((
        BindingDialog {
            button_entity: trigger.target(),
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Vw(1.0)),
                row_gap: Vw(1.0),
                ..Default::default()
            },
            BackgroundColor(WHITEISH),
            children![label(
                "Binding \"{name}\", \npress any key or Esc to cancel"
            )]
        )],
    ));
}

fn bind(
    mut commands: Commands,
    mut key_events: EventReader<KeyboardInput>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
    dialog: Single<(Entity, &BindingDialog)>,
    root_entity: Single<Entity, (With<Node>, Without<ChildOf>)>,
    mut buttons: Query<(Entity, &Name, &mut InputButton)>,
) {
    let keys = key_events
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .map(|event| event.key_code.into());
    let mouse_buttons = mouse_button_events
        .read()
        .filter(|event| event.state == ButtonState::Pressed)
        .map(|event| event.button.into());

    let Some(input) = keys.chain(mouse_buttons).next() else {
        return;
    };

    let (dialog_entity, dialog) = *dialog;

    if let Some((conflict_entity, name, _)) = buttons
        .iter()
        .find(|(.., button)| button.input == Some(input))
    {
        info!("found conflict with '{name}' for '{input}'");

        commands.entity(*root_entity).with_child((
            ConflictDialog {
                button_entity: dialog.button_entity,
                conflict_entity,
            },
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Vw(1.0)),
                    row_gap: Vw(1.0),
                    ..Default::default()
                },
                BackgroundColor(WHITEISH),
                children![
                    (
                        TextColor(super::GRAY),
                        Text::new(format!("\"{input}\" is already used by \"{name}\"",)),
                    ),
                    (
                        Node {
                            column_gap: Vw(1.0),
                            ..Default::default()
                        },
                        Children::spawn(SpawnWith(|parent: &mut RelatedSpawner<_>| {
                            parent
                                .spawn((SettingsButton, children![Text::new("Replace")]))
                                .observe(replace_binding);
                            parent
                                .spawn((SettingsButton, children![Text::new("Cancel")]))
                                .observe(cancel_replace_binding);
                        }))
                    )
                ]
            )],
        ));
    } else {
        let (_, name, mut button) = buttons
            .get_mut(dialog.button_entity)
            .expect("binding dialog should point to a button with input");
        info!("assigning '{input}' to '{name}'");
        button.input = Some(input);
    }

    commands.entity(dialog_entity).despawn();
}

fn cancel_binding(mut commands: Commands, dialog_entity: Single<Entity, With<BindingDialog>>) {
    info!("cancelling binding");
    commands.entity(*dialog_entity).despawn();
}

fn replace_binding(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    dialog: Single<(Entity, &ConflictDialog)>,
    mut buttons: Query<(&Name, &mut InputButton)>,
) {
    let (dialog_entity, dialog) = *dialog;
    let (_, mut conflict_button) = buttons
        .get_mut(dialog.conflict_entity)
        .expect("binding conflict should point to a button");
    let input = conflict_button.input;
    conflict_button.input = None;

    let (name, mut button) = buttons
        .get_mut(dialog.button_entity)
        .expect("binding should point to a button");
    button.input = input;

    info!("reassigning binding to '{name}'");
    commands.entity(dialog_entity).despawn();
}

fn cancel_replace_binding(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    dialog_entity: Single<Entity, With<ConflictDialog>>,
) {
    info!("cancelling replace binding");
    commands.entity(*dialog_entity).despawn();
}

fn apply(
    _trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut settings: ResMut<Settings>,
    buttons: Query<(&InputButton, &SettingsField)>,
) {
    settings.keybind.clear();
    for (button, field) in &buttons {
        if let Some(input) = button.input {
            // Utilize reflection to write by field name.
            let field_value = settings
                .path_mut::<Vec<Input>>(field.0)
                .expect("fields with bindings should be stored as Vec");
            field_value.push(input);
        }
    }

    commands.trigger(RebuildBindings);
}

fn update_button_text(
    buttons: Query<(&InputButton, &Children), Changed<InputButton>>,
    mut text: Query<&mut Text>,
) {
    for (button, children) in &buttons {
        let mut iter = text.iter_many_mut(children);
        let mut text = iter.fetch_next().unwrap();
        text.clear();
        if let Some(input) = button.input {
            write!(text, "{input}").unwrap();
        } else {
            write!(text, "Empty").unwrap();
        };
    }
}

#[derive(Component, Default)]
#[require(
    Button,
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(160.0),
        height: Val::Px(35.0),
        ..Default::default()
    },
)]
struct SettingsButton;

/// Stores information about button binding.
#[derive(Component)]
#[require(SettingsButton)]
struct InputButton {
    /// Assigned input.
    input: Option<Input>,
}

/// Stores assigned button with input.
#[derive(Component)]
#[require(
    Button,
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Px(35.0),
        height: Val::Px(35.0),
        ..Default::default()
    },
)]
struct DeleteButton {
    /// Entity with [`InputButton`].
    button_entity: Entity,
}

#[derive(Component, Default)]
#[require(
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..Default::default()
    },
    FocusPolicy::Block,
    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
)]
struct Dialog;

#[derive(Component)]
#[require(Dialog)]
struct BindingDialog {
    /// Entity with [`InputButton`].
    button_entity: Entity,
}

#[derive(Component)]
#[require(Dialog)]
struct ConflictDialog {
    /// Entity with [`InputButton`].
    button_entity: Entity,
    /// Entity with [`InputButton`] that conflicts with [`Self::button_entity`].
    conflict_entity: Entity,
}
