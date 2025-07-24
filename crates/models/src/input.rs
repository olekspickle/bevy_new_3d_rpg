use super::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<GameplayCtx>()
        .add_input_context::<ModalCtx>()
        .add_systems(Startup, spawn_ctx)
        .add_observer(on_ctx_switch)
        .add_observer(bind_modal)
        .add_observer(bind_gameplay);
}

fn spawn_ctx(mut cmds: Commands) {
    cmds.spawn((GlobalInputCtx, Actions::<ModalCtx>::default(), ModalCtx));
}

/// Context switch observer
/// If event entity is the [`Entity`] placeholder it is meant for global context entity
/// [`InputContext`] which is mostly needed for game UI when no player spawned yet.
///
/// We could just have one context for everything, but I plan to support split screen
/// and changing context on player is useful for different cases like driving,
/// swimming, so I find this setup the most reasonable.
///
/// TODO: check with split screen
/// TODO: I'm pretty sure there is a better way to organize this, but I'm not familiar enough with
/// [`bevy_enhanced_input`] yet to figure it out, so for now it works
fn on_ctx_switch(
    on: Trigger<SwitchInputCtx>,
    mut commands: Commands,
    mut global_ctx: Query<Entity, With<GlobalInputCtx>>,
    mut players: Query<(Entity, &mut CurrentCtx), With<Player>>,
) {
    let entity = on.event().entity;
    let new_ctx = &on.event().ctx;

    if entity == Entity::PLACEHOLDER {
        // global context reset
        if let Ok(global_ctx) = global_ctx.single_mut() {
            match new_ctx {
                Context::Modal => commands
                    .entity(global_ctx)
                    .insert(Actions::<ModalCtx>::default()),
                Context::Gameplay => commands.entity(global_ctx).remove::<Actions<ModalCtx>>(),
            };

            info!("Switched global context to {:?}", new_ctx);
        }

        return;
    }

    if let Ok((entity, mut current_ctx)) = players.get_mut(entity) {
        match (current_ctx.0.clone(), new_ctx.clone()) {
            (Context::Modal, Context::Gameplay) => {
                commands
                    .entity(entity)
                    .remove::<Actions<ModalCtx>>()
                    .insert(Actions::<GameplayCtx>::default());
            }
            (Context::Gameplay, Context::Modal) => {
                commands
                    .entity(entity)
                    .remove::<Actions<GameplayCtx>>()
                    .insert(Actions::<ModalCtx>::default());
            }
            _ => {}
        }

        current_ctx.0 = new_ctx.clone();
        info!("Switched player {entity:?} context to {:?}", new_ctx);
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct CurrentCtx(pub Context);

#[derive(Debug, Clone, Default)]
pub enum Context {
    #[default]
    Modal,
    Gameplay,
}

/// TODO: figure out split screen
/// Used as both input context and component.
#[derive(InputContext, Component, Clone, Copy)]
pub struct GameplayCtx;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub struct Navigate;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, require_reset = true)]
pub struct Pan;

// #[cfg(feature = "top_down")]
#[derive(Debug, InputAction)]
#[input_action(output = Vec2, require_reset = true)]
pub struct ScrollZoom;

// #[cfg(feature = "top_down")]
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct RotateToggle;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Attack;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Jump;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Sprint;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Dash;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Crouch;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Pause;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Mute;

#[derive(InputContext, Component, Clone, Copy)]
#[input_context(priority = 1)]
pub struct ModalCtx;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2, require_reset = true)]
struct NavigateModal;

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
pub struct Escape;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Select;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct RightTab;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct LeftTab;

fn bind_gameplay(
    trigger: Trigger<Binding<GameplayCtx>>,
    mut context: Query<(&GameplayCtx, &mut Actions<GameplayCtx>)>,
) {
    let (&_id, mut actions) = context
        .get_mut(trigger.target())
        .expect("Failed to query gameplay context actions");

    // let gamepad_entity = gamepads.iter().nth(id as usize);
    // actions.set_gamepad(gamepad_entity.unwrap_or(Entity::PLACEHOLDER));

    actions
        .bind::<Navigate>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers((
            DeadZone::default(), // Apply non-uniform normalization to ensure consistent speed, otherwise diagonal movement will be faster.
            Scale::splat(0.3), // Additionally multiply by a constant to achieve the desired speed.
        ));

    actions.bind::<Pan>().to((
        Input::mouse_motion().with_modifiers((Scale::splat(0.1), Negate::all())),
        Axial::right_stick().with_modifiers_each((Scale::splat(2.0), Negate::x())),
    ));

    // #[cfg(feature = "top_down")]
    {
        actions.bind::<ScrollZoom>().to(Input::mouse_wheel());
        actions.bind::<RotateToggle>().to(MouseButton::Right);
    }

    actions.bind::<Pause>().to(KeyCode::KeyP);
    actions.bind::<Mute>().to(KeyCode::KeyM);
    actions
        .bind::<Escape>()
        .to((KeyCode::Escape, GamepadButton::Select));
    actions
        .bind::<Crouch>()
        .to((KeyCode::ControlLeft, GamepadButton::East));
    actions
        .bind::<Jump>()
        .to((KeyCode::Space, GamepadButton::South));
    actions
        .bind::<Dash>()
        .to((KeyCode::AltLeft, GamepadButton::LeftTrigger));
    actions
        .bind::<Sprint>()
        .to((KeyCode::ShiftLeft, GamepadButton::LeftThumb));
    actions
        .bind::<Attack>()
        .to((MouseButton::Left, GamepadButton::RightTrigger2));
}

fn bind_modal(
    trigger: Trigger<Binding<ModalCtx>>,
    mut menus: Query<(&ModalCtx, &mut Actions<ModalCtx>)>,
) {
    let (&_id, mut actions) = menus
        .get_mut(trigger.target())
        .expect("Failed to get modal context id");

    // let gamepad_entity = gamepads.iter().nth(id as usize);
    // actions.set_gamepad(gamepad_entity.unwrap_or(Entity::PLACEHOLDER));

    actions.bind::<NavigateModal>().to((
        Cardinal::wasd_keys(),
        Input::mouse_motion().with_modifiers((Scale::splat(0.1), Negate::all())),
        Axial::right_stick().with_modifiers_each((Scale::splat(2.0), Negate::x())),
    ));

    actions
        .bind::<Escape>()
        .to((KeyCode::Escape, GamepadButton::East));
    actions
        .bind::<Select>()
        .to((KeyCode::Enter, GamepadButton::South, MouseButton::Left));
    actions.bind::<RightTab>().to(GamepadButton::RightTrigger);
    actions.bind::<LeftTab>().to(GamepadButton::LeftTrigger);
}
