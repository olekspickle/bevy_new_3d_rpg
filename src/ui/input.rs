//! [`ModalInput`]: the input context shared by menus, modals and other UI screens, plus a
//! gamepad-driven virtual pointer, since Wayland can't warp the OS cursor.
use super::*;
use bevy::asset::uuid::Uuid;
use bevy::camera::RenderTarget;
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::input::mouse::MouseMotion;
use bevy::picking::pointer::{
    Location, PointerAction, PointerButton, PointerId, PointerInput, PointerLocation,
};
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_enhanced_input::prelude::*;

mod knobs {
    /// Logical pixels per second at full stick tilt
    pub const CURSOR_SPEED: f32 = 900.0;
    pub const CURSOR_SIZE: f32 = 16.0;
}

const GAMEPAD_POINTER: PointerId =
    PointerId::Custom(Uuid::from_u128(0x6d0d_a1c3_9e4b_4f7a_b1f2_c0de_cafe_0001));

pub fn plugin(app: &mut App) {
    app.add_input_context::<ModalInput>()
        .add_systems(Startup, (spawn_ctx, spawn_gamepad_cursor))
        .add_systems(Update, (hide_gamepad_cursor, move_gamepad_cursor).chain())
        .add_observer(press_gamepad_cursor)
        .add_observer(release_gamepad_cursor);
}

markers!(MainMenuCtx, GamepadCursor);

fn spawn_ctx(mut commands: Commands) {
    commands.spawn((MainMenuCtx, ModalInput));
}

/// Moves the gamepad cursor
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct NavigateModal;

/// Clicks with the gamepad cursor
#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Select;

/// Controller tab switch right
#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct CycleTab;

/// Controller tab switch left
#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct CycleTabBack;

#[derive(Component, Default)]
#[component(on_add = ModalInput::on_add)]
pub(crate) struct ModalInput;

impl ModalInput {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .insert(actions!(ModalInput[
                (
                    Action::<CycleTab>::new(),
                    bindings![KeyCode::Tab, GamepadButton::RightTrigger],
                ),
                (
                    Action::<CycleTabBack>::new(),
                    ActionSettings {
                        consume_input: true,
                        ..Default::default()
                    },
                    bindings![
                        (KeyCode::Tab).with_mod_keys(ModKeys::SHIFT),
                        GamepadButton::LeftTrigger
                    ],
                ),
                (
                    Action::<NavigateModal>::new(),
                    // screen y grows downward
                    Bindings::spawn(Axial::right_stick().with((DeadZone::default(), Negate::y()))),
                ),
                (
                    Action::<Select>::new(),
                    bindings![GamepadButton::South],
                ),
                (
                    Action::<Escape>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    bindings![KeyCode::Escape, GamepadButton::East],
                )
            ]));
    }
}

/// The cursor node is also the picking pointer, so UI reacts to it like to the mouse.
fn spawn_gamepad_cursor(mut commands: Commands) {
    commands.spawn((
        Name::new("Gamepad Cursor"),
        GamepadCursor,
        GAMEPAD_POINTER,
        Node {
            position_type: PositionType::Absolute,
            width: Px(knobs::CURSOR_SIZE),
            height: Px(knobs::CURSOR_SIZE),
            border: UiRect::all(Px(2.0)),
            border_radius: BorderRadius::MAX,
            ..Default::default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(Color::BLACK.with_alpha(0.4)),
        GlobalZIndex(i32::MAX),
        Pickable::IGNORE,
        Visibility::Hidden,
    ));
}

/// Hides on mouse movement or once no UI context is active.
fn hide_gamepad_cursor(
    mut motion: MessageReader<MouseMotion>,
    contexts: Query<Option<&ContextActivity<ModalInput>>, With<ModalInput>>,
    cursor: Single<(&mut Visibility, &mut PointerLocation), With<GamepadCursor>>,
) {
    let mouse_moved = motion.read().count() > 0;
    let in_ui = contexts.iter().any(|activity| activity.is_none_or(|a| **a));
    let (mut visibility, mut pointer) = cursor.into_inner();
    if *visibility == Visibility::Hidden || in_ui && !mouse_moved {
        return;
    }
    *visibility = Visibility::Hidden;
    // no location keeps the hidden pointer from hovering anything
    pointer.location = None;
}

fn move_gamepad_cursor(
    // menus pause virtual time
    time: Res<Time<Real>>,
    navigate: Query<&Action<NavigateModal>>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    cursor: Single<(&mut Node, &mut Visibility, &PointerLocation), With<GamepadCursor>>,
    mut inputs: MessageWriter<PointerInput>,
) {
    let stick = navigate.iter().map(|a| **a).sum::<Vec2>();
    if stick == Vec2::ZERO {
        return;
    }
    let (window_e, window) = *window;
    let (mut node, mut visibility, pointer) = cursor.into_inner();
    // a freshly shown cursor starts where the mouse is
    let from = match (*visibility, pointer.location()) {
        (Visibility::Visible, Some(location)) => location.position,
        _ => window.cursor_position().unwrap_or(window.size() / 2.0),
    };
    // squared response keeps small tilts precise
    let step = stick * stick.length() * knobs::CURSOR_SPEED * time.delta_secs();
    let to = (from + step).clamp(Vec2::ZERO, window.size());
    let Some(target) = RenderTarget::Window(WindowRef::Entity(window_e)).normalize(Some(window_e))
    else {
        return;
    };

    *visibility = Visibility::Visible;
    node.left = Px(to.x - knobs::CURSOR_SIZE / 2.0);
    node.top = Px(to.y - knobs::CURSOR_SIZE / 2.0);
    let location = Location {
        target,
        position: to,
    };
    let action = PointerAction::Move { delta: to - from };
    inputs.write(PointerInput::new(GAMEPAD_POINTER, location, action));
}

fn press_gamepad_cursor(
    _: On<Start<Select>>,
    cursor: Single<&PointerLocation, With<GamepadCursor>>,
    inputs: MessageWriter<PointerInput>,
) {
    send_button(
        &cursor,
        inputs,
        PointerAction::Press(PointerButton::Primary),
    );
}

fn release_gamepad_cursor(
    _: On<Complete<Select>>,
    cursor: Single<&PointerLocation, With<GamepadCursor>>,
    inputs: MessageWriter<PointerInput>,
) {
    send_button(
        &cursor,
        inputs,
        PointerAction::Release(PointerButton::Primary),
    );
}

fn send_button(
    pointer: &PointerLocation,
    mut inputs: MessageWriter<PointerInput>,
    action: PointerAction,
) {
    // a hidden cursor has no location
    if let Some(location) = pointer.location() {
        inputs.write(PointerInput::new(GAMEPAD_POINTER, location.clone(), action));
    }
}
