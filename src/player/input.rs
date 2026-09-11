use super::*;
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};

pub fn plugin(app: &mut App) {
    app.add_input_context::<PlayerInput>()
        .add_observer(on_modal_add)
        .add_observer(reload_player_bindings);
}

#[derive(InputAction)]
#[action_output(bool)]
pub struct ZoomView;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Sprint;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Dash;

#[derive(Component, Default)]
#[component(on_add = PlayerInput::on_add)]
pub(crate) struct PlayerInput;

impl PlayerInput {
    /// Reads the current keyboard/mouse bindings from [`Settings::input_map`] so rebinding in the
    /// keybind editor takes effect.
    ///
    /// Note: Gamepad isn't rebindable (see [`InputSettings`]), so those
    /// stick around as fixed fallback bindings alongside whatever the player
    /// configured for keyboard/mouse.
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let keys = world.resource::<Settings>().input_map.clone();

        world.commands().entity(ctx.entity).insert((
            actions!(PlayerInput[
                (
                    Action::<Movement>::new(),
                    DeadZone::default(),
                    Scale::splat(0.3),
                    Bindings::spawn((
                        Cardinal::new(keys.forward[0], keys.left[0], keys.backward[0], keys.right[0]),
                        Cardinal::new(keys.forward[1], keys.left[1], keys.backward[1], keys.right[1]),
                        Cardinal::new(keys.forward[2], keys.left[2], keys.backward[2], keys.right[2]),
                        Axial::left_stick(),
                    )),
                ),
                (
                    Action::<Crouch>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    bindings![keys.crouch[0], keys.crouch[1], keys.crouch[2], GamepadButton::East],
                ),
                (
                    Action::<Jump>::new(),
                    bindings![keys.jump[0], keys.jump[1], keys.jump[2], GamepadButton::South],
                ),
                (
                    Action::<Sprint>::new(),
                    bindings![keys.sprint[0], keys.sprint[1], keys.sprint[2], GamepadButton::LeftThumb],
                ),
                (
                    Action::<Dash>::new(),
                    bindings![keys.dash[0], keys.dash[1], keys.dash[2], GamepadButton::LeftTrigger],
                ),
                // (
                //     Action::<Attack>::new(),
                //     bindings![MouseButton::Left, GamepadButton::RightTrigger2],
                // ),
                // (
                //     Action::<ZoomView>::new(),
                //     bindings![MouseButton::Right, GamepadButton::RightTrigger2],
                // ),
                (
                    Action::<RotateCamera>::new(),
                    Bindings::spawn((
                        // tweak mouse and right stick sensitivity in Scale::splat values
                        Spawn((Binding::mouse_motion(), Scale::splat(0.07))),
                        Axial::right_stick().with((Scale::splat(4.0), DeadZone::default())),
                    )),
                ),
                (
                    Action::<Escape>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    bindings![KeyCode::Escape, GamepadButton::Select],
                )
            ]),
            ModalInput,
            ContextActivity::<ModalInput>::INACTIVE,
            ContextActivity::<PlayerInput>::ACTIVE,
        ));
    }
}

fn on_modal_add(
    _: On<Add, ModalRoot>,
    mut commands: Commands,
    players_q: Query<Entity, With<Player>>,
) {
    // TODO: only do that for the player that called the modal
    for e in players_q.iter() {
        commands.entity(e).insert_if_new(modal_ctx_active());
    }
}

/// Rebuild the live player's bindings from [`Settings::input_map`] whenever the keybind editor
/// commits a change, so a rebind takes effect immediately instead of only on the next respawn.
fn reload_player_bindings(
    _: On<SettingsChanged>,
    mut commands: Commands,
    players_q: Query<Entity, With<PlayerInput>>,
) {
    for player in &players_q {
        // `on_add` only fires on a fresh insertion, so the actions have to actually be removed
        // first - re-inserting an already-present `PlayerInput` marker would be a no-op.
        commands
            .entity(player)
            .despawn_related::<Actions<PlayerInput>>()
            .remove::<PlayerInput>()
            .insert(PlayerInput);
    }
}

pub fn modal_ctx_active() -> impl Bundle {
    (
        ContextActivity::<ModalInput>::ACTIVE,
        ContextActivity::<PlayerInput>::INACTIVE,
    )
}
pub fn player_ctx_active() -> impl Bundle {
    (
        ContextActivity::<ModalInput>::INACTIVE,
        ContextActivity::<PlayerInput>::ACTIVE,
    )
}
