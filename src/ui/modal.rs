//! This plugin handles any kind of modal that would be present during gameplay

use super::*;
use crate::player::{Player, modal_ctx_active, player_ctx_active};
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy_enhanced_input::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<Modal>()
        .init_resource::<Modals>()
        .add_input_context::<ModalInput>()
        .add_systems(Startup, spawn_ctx)
        .add_observer(on_clear_modals)
        .add_observer(on_pop_modal)
        .add_observer(on_new_modal);
}

markers!(MainMenuCtx);

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

#[derive(Resource, Default, Deref, DerefMut, Debug, Clone)]
pub struct Modals(pub Vec<Modal>);

/// Modal stack. kudo for the idea to @skyemakesgames
/// Only relevant in [`Screen::Gameplay`]
#[derive(States, Component, Reflect, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Modal {
    #[default]
    Main,
    Settings,
}
impl Modal {
    pub fn is_main(&self) -> bool {
        matches!(self, Self::Main)
    }
}

#[derive(EntityEvent)]
pub struct NewModal {
    pub entity: Entity,
    pub modal: Modal,
}
#[derive(EntityEvent)]
pub struct PopModal(pub Entity);

#[derive(EntityEvent)]
pub struct ClearModals(pub Entity);

// TODO: the event entity will be th ebutton
pub fn click_pop_modal(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    players_q: Query<Entity, With<Player>>,
) {
    for e in players_q.iter() {
        commands.entity(e).trigger(PopModal);
    }
}

pub fn on_new_modal(
    on: On<NewModal>,
    screen: Res<State<Screen>>,
    state: Res<GameState>,
    mut commands: Commands,
    mut modals: ResMut<Modals>,
    mut window_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !screen.get().is_gameplay() {
        return;
    }

    let mut target = commands.entity(on.entity);
    if modals.is_empty() {
        // only pause the first time we spawn a main modal
        if on.modal.is_main() && !state.paused {
            target.trigger(TogglePause);
        }

        // something else already showed the cursor (e.g. cursor)
        if let Ok(cursor_options) = window_q.single_mut()
            && !cursor_options.visible
        {
            target.trigger(ToggleCamCursor);
        }

        target.insert(modal_ctx_active());
    }

    // despawn all previous modal entities to avoid clattering
    target.trigger(ClearModals);
    match on.event().modal {
        Modal::Main => commands.spawn(menu_modal()),
        Modal::Settings => commands.spawn(settings_modal()),
    };

    modals.push(on.event().modal.clone());
}

pub fn on_pop_modal(
    pop: On<PopModal>,
    screen: Res<State<Screen>>,
    modals_q: Query<(Entity, &Modal)>,
    mut commands: Commands,
    mut modals: ResMut<Modals>,
) {
    if !screen.get().is_gameplay() {
        return;
    }

    debug!("Chat, are we popping? {:?}", modals);
    assert!(!modals.is_empty());

    let Some(popped) = modals.pop() else {
        error!("popped none modal after assert");
        return;
    };

    for (e, modal) in modals_q.iter() {
        if *modal == popped {
            commands.entity(e).despawn();
        }
    }

    // respawn next in the modal stack
    if let Some(modal) = modals.last() {
        match modal {
            Modal::Main => commands.spawn(menu_modal()),
            Modal::Settings => commands.spawn(settings_modal()),
        };
    }

    if modals.is_empty() {
        info!("PopModal target entity: {}", pop.event_target());
        commands
            .entity(pop.event_target())
            .insert(player_ctx_active())
            .trigger(TogglePause)
            .trigger(ToggleCamCursor);
    }
}

pub fn on_clear_modals(
    _: On<ClearModals>,
    modals_q: Query<Entity, With<Modal>>,
    mut commands: Commands,
) {
    for m in modals_q.iter() {
        commands.entity(m).despawn();
    }
}
