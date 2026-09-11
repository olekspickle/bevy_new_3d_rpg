//! A universal, stylable modal stack plugin.
//!
//! Any screen can push arbitrary content ([`push_modal`] takes any `impl Bundle`) onto the
//! stack and expect consistent behavior: pausing gameplay and showing the cursor the first
//! time a modal opens, keyboard/gamepad tab-cycling and select navigation ([`ModalInput`]),
//! and restoring gameplay once the stack empties again. Only the top of the stack is ever
//! visible — pushing a new modal hides the one it covers, so popping back to it later shows
//! the same entity again. Closing a modal (popping it) despawns it for good.

use super::*;
use crate::player::{Player, modal_ctx_active, player_ctx_active};
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy_enhanced_input::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<Modals>()
        .add_input_context::<ModalInput>()
        .add_systems(Startup, spawn_ctx)
        .add_observer(on_push_modal)
        .add_observer(on_pop_modal);
}

markers!(MainMenuCtx, ModalRoot);

fn spawn_ctx(mut commands: Commands) {
    commands.spawn((MainMenuCtx, ModalInput));
}

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct NavigateModal;

/// Controller element select. F.e. for inventory cell
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
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    Bindings::spawn((
                        Spawn((Binding::mouse_motion(), Scale::splat(0.1), Negate::all())),
                        Axial::right_stick().with((Scale::splat(2.0), Negate::x())),
                    )),
                ),
                (
                    Action::<Select>::new(),
                    bindings![KeyCode::Enter, GamepadButton::South, MouseButton::Left],
                ),
                (
                    Action::<Escape>::new(),
                    ActionSettings {
                        require_reset: true,
                        ..Default::default()
                    },
                    bindings![KeyCode::Escape, GamepadButton::West],
                )
            ]));
    }
}

/// Spawns `content` as a modal root and pushes it onto the [`Modals`] stack, hiding whatever
/// it covers rather than despawning it. `entity` is the entity whose input context toggles to
/// [`ModalInput`] and which receives the pause/cursor side effects if this is the first modal
/// opened — almost always the player entity.
///
/// ```ignore
/// push_modal(&mut commands, on.entity, menu_modal());
/// ```
pub fn push_modal(commands: &mut Commands, entity: Entity, content: impl Bundle) {
    let modal = commands
        .spawn((ModalRoot, DespawnOnExit(Screen::Gameplay), content))
        .id();
    commands.trigger(PushModal { entity, modal });
}

/// Stack of currently-spawned modal root entities, topmost last. Only the last one is ever
/// visible; the rest are alive but hidden underneath it.
#[derive(Resource, Default, Debug)]
pub struct Modals {
    stack: Vec<Entity>,
}

impl Modals {
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Resets the stack's bookkeeping without despawning anything — entities are expected to
    /// already be gone (e.g. via `DespawnOnExit(Screen::Gameplay)`) by the time this is called.
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

/// Triggered by [`push_modal`] once `modal` has been spawned; do not construct this directly.
#[derive(EntityEvent)]
pub struct PushModal {
    pub entity: Entity,
    pub modal: Entity,
}

#[derive(EntityEvent)]
pub struct PopModal(pub Entity);

pub fn click_pop_modal(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    players_q: Query<Entity, With<Player>>,
) {
    for e in players_q.iter() {
        commands.entity(e).trigger(PopModal);
    }
}

pub fn on_push_modal(
    on: On<PushModal>,
    screen: Res<State<Screen>>,
    state: Res<GameState>,
    mut commands: Commands,
    mut modals: ResMut<Modals>,
    mut window_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !screen.get().is_gameplay() {
        commands.entity(on.modal).despawn();
        return;
    }

    let mut target = commands.entity(on.entity);
    if modals.is_empty() {
        // only pause/show-cursor the first time a modal opens over an otherwise-unpaused game
        if !state.paused {
            target.trigger(TogglePause);
        }

        // something else already showed the cursor (e.g. cursor)
        if let Ok(cursor_options) = window_q.single_mut()
            && !cursor_options.visible
        {
            target.trigger(ToggleCamCursor);
        }

        target.insert(modal_ctx_active());
    } else if let Some(&covered) = modals.stack.last() {
        commands.entity(covered).insert(Visibility::Hidden);
    }

    modals.stack.push(on.modal);
}

pub fn on_pop_modal(
    pop: On<PopModal>,
    screen: Res<State<Screen>>,
    mut commands: Commands,
    mut modals: ResMut<Modals>,
) {
    if !screen.get().is_gameplay() {
        return;
    }

    debug!("popping modal, stack depth before pop: {}", modals.len());
    assert!(!modals.is_empty(), "popped modal with an empty stack");

    if let Some(closed) = modals.stack.pop() {
        commands.entity(closed).despawn();
    }

    if let Some(&revealed) = modals.stack.last() {
        commands.entity(revealed).insert(Visibility::Inherited);
        return;
    }

    info!("PopModal target entity: {}", pop.event_target());
    commands
        .entity(pop.event_target())
        .insert(player_ctx_active())
        .trigger(TogglePause)
        .trigger(ToggleCamCursor);
}
