use super::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<PlayerCtx>()
        .add_input_context::<ModalCtx>()
        .add_systems(Startup, spawn_ctx)
        .add_observer(rm_modal_ctx)
        .add_observer(rm_player_ctx)
        .add_observer(add_modal_ctx)
        .add_observer(add_player_ctx);
}

fn spawn_ctx(mut cmds: Commands) {
    cmds.spawn(ModalCtx);
}

#[derive(InputAction, Component)]
#[action_output(Vec2)]
pub struct Navigate;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Pan;

// #[cfg(feature = "top_down")]
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct ScrollZoom;

// #[cfg(feature = "top_down")]
#[derive(InputAction)]
#[action_output(bool)]
pub struct RotateToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Attack;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Jump;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Sprint;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Dash;

#[derive(InputAction, Component)]
#[action_output(bool)]
pub struct Crouch;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Pause;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Mute;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Escape;

#[derive(InputAction)]
#[action_output(Vec2)]
struct NavigateModal;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Select;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct RightTab;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct LeftTab;

pub fn add_player_ctx(
    on: Trigger<OnAdd, PlayerCtx>,
    mut commands: Commands,
    // mut settings: ResMut<Settings>,
) {
    let mut e = commands.entity(on.target());

    e.insert(actions!(PlayerCtx[
        (
            Action::<Pan>::new(),
            ActionSettings {
                require_reset: true,
                ..Default::default()
            },
            Bindings::spawn((
                Spawn((Binding::mouse_motion(),Scale::splat(0.1), Negate::all())),
                Axial::right_stick().with((Scale::splat(2.0), Negate::x())) ,
            )),
        ),

        (
            Action::<Navigate>::new(),
            DeadZone::default(),
            Scale::splat(0.3),
            Bindings::spawn(( Cardinal::wasd_keys(), Cardinal::arrow_keys(), Axial::left_stick() )),
        ),
        (
            Action::<Crouch>::new(),
            bindings![KeyCode::ControlLeft, GamepadButton::East],
        ),
        (
            Action::<Jump>::new(),
            bindings![KeyCode::Space, GamepadButton::South],
        ),
        (
            Action::<Dash>::new(),
            bindings![KeyCode::AltLeft, GamepadButton::LeftTrigger],
        ),
        (
            Action::<Sprint>::new(),
            bindings![KeyCode::ShiftLeft, GamepadButton::LeftThumb],
        ),
        (
            Action::<Attack>::new(),
            bindings![MouseButton::Left, GamepadButton::RightTrigger2],
        ),

        (
            Action::<Pause>::new(),
            bindings![KeyCode::KeyP],
        ),
        (
            Action::<Mute>::new(),
            bindings![KeyCode::KeyM],
        ),
        (
            Action::<Escape>::new(),
            ActionSettings {
                require_reset: true,
                ..Default::default()
            },
            bindings![KeyCode::Escape, GamepadButton::Select],
        ),
    ]));

    // #[cfg(feature = "top_down")]
    e.insert(actions!(ModalCtx[
        (
            Action::<ScrollZoom>::new(),
            ActionSettings {
                require_reset: true,
                ..Default::default()
            },
                Bindings::spawn( Spawn(Binding::mouse_motion())),
        ),
        (
            Action::<RotateToggle>::new(),
            bindings![MouseButton::Right],
        ),
    ]));
}

fn rm_player_ctx(on: Trigger<OnRemove, PlayerCtx>, mut commands: Commands) {
    commands
        .entity(on.target())
        .remove_with_requires::<PlayerCtx>()
        .despawn_related::<Actions<PlayerCtx>>();
}

fn add_modal_ctx(on: Trigger<OnAdd, ModalCtx>, mut commands: Commands) {
    commands.entity(on.target()).insert((
        ContextPriority::<ModalCtx>::new(1),
        actions!(ModalCtx[
            (
                Action::<NavigateModal>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
                Bindings::spawn((
                    Cardinal::wasd_keys(),
                    Spawn((Binding::mouse_motion(),Scale::splat(0.1), Negate::all())),
                    Axial::right_stick().with((Scale::splat(2.0), Negate::x())) ,
                )),
            ),
        (
            Action::<Select>::new(),
            bindings![KeyCode::Enter, GamepadButton::South, MouseButton::Left],
        ),
        (
            Action::<RightTab>::new(),
            bindings![KeyCode::Tab, GamepadButton::RightTrigger],
        ),
        (
            Action::<LeftTab>::new(),
            bindings![GamepadButton::LeftTrigger],
        ),
        (
            Action::<Escape>::new(),
                ActionSettings {
                    require_reset: true,
                    ..Default::default()
                },
            bindings![KeyCode::Escape, GamepadButton::Select],
        ),
        ]),
    ));
}

fn rm_modal_ctx(on: Trigger<OnRemove, ModalCtx>, mut commands: Commands) {
    commands
        .entity(on.target())
        .remove_with_requires::<ModalCtx>()
        .despawn_related::<Actions<ModalCtx>>();
}
