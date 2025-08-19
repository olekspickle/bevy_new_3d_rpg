//! The screen state for the main gameplay.

use super::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_gameplay_ui)
        .add_observer(toggle_mute)
        .add_observer(toggle_pause)
        .add_observer(trigger_menu_toggle_on_esc)
        .add_observer(add_new_modal)
        .add_observer(pop_modal)
        .add_observer(clear_modals);
}

fn spawn_gameplay_ui(mut cmds: Commands, textures: Res<Textures>, _settings: Res<Settings>) {
    // info!("settings on gameplay enter:{settings:?}");
    let opts = Opts::default().hidden().width(Vw(5.0)).height(Vw(5.0));
    cmds.spawn((
        StateScoped(Screen::Gameplay),
        GameplayUi,
        ui_root("Gameplay Ui"),
        children![
            // mute/pause icons
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    position_type: PositionType::Absolute,
                    top: Px(0.0),
                    left: Vw(47.5),
                    ..Default::default()
                },
                children![
                    (icon(opts.clone().image(textures.pause.clone())), PauseIcon),
                    (icon(opts.clone().image(textures.mute.clone())), MuteIcon),
                ]
            ),
        ],
    ));
}

fn toggle_pause(
    _: Trigger<TogglePause>,
    mut time: ResMut<Time<Virtual>>,
    mut state: ResMut<GameState>,
    mut pause_label: Query<&mut Node, With<PauseIcon>>,
) {
    if let Ok(mut label) = pause_label.single_mut() {
        if time.is_paused() || state.paused {
            time.unpause();
            label.display = Display::None;
        } else {
            time.pause();
            label.display = Display::Flex;
        }
    }

    state.paused = !state.paused;
    info!("paused: {}", state.paused);
}

fn toggle_mute(
    _: Trigger<ToggleMute>,
    settings: ResMut<Settings>,
    mut state: ResMut<GameState>,
    mut label: Query<&mut Node, With<MuteIcon>>,
    mut music: Single<&mut VolumeNode, (With<SamplerPool<Music>>, Without<SamplerPool<Sfx>>)>,
    mut sfx: Single<&mut VolumeNode, (With<SamplerPool<Sfx>>, Without<SamplerPool<Music>>)>,
) {
    if let Ok(mut node) = label.single_mut() {
        if state.muted {
            music.volume = settings.music();
            sfx.volume = settings.sfx();
            node.display = Display::None;
        } else {
            music.volume = Volume::SILENT;
            sfx.volume = Volume::SILENT;
            node.display = Display::Flex;
        }
    }
    state.muted = !state.muted;
    info!("muted: {}", state.muted);
}

// ============================ UI ============================

fn click_to_menu(
    on: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
) {
    info!("click NewModal target entity: {}", on.target());
    commands.entity(on.target()).insert(ModalCtx);
    commands.trigger(GoTo(Screen::Title));
    state.reset();
}
fn click_pop_modal(_: Trigger<Pointer<Click>>, mut cmds: Commands) {
    cmds.trigger(PopModal);
}
fn click_spawn_settings(_: Trigger<Pointer<Click>>, mut cmds: Commands) {
    cmds.trigger(NewModal(Modal::Settings));
}

fn trigger_menu_toggle_on_esc(
    _: Trigger<Back>,
    mut cmds: Commands,
    screen: Res<State<Screen>>,
    state: ResMut<GameState>,
) {
    if *screen.get() != Screen::Gameplay {
        return;
    }

    if state.modals.is_empty() {
        cmds.trigger(NewModal(Modal::Main));
    } else {
        cmds.trigger(PopModal);
    }
}

fn add_new_modal(
    on: Trigger<NewModal>,
    screen: Res<State<Screen>>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
) {
    if *screen.get() != Screen::Gameplay {
        return;
    }

    info!("new modal:{:?}, settings.paused:{}", on.0, state.paused);
    if state.modals.is_empty() {
        info!("NewModal target entity: {}", on.target());
        commands.entity(on.target()).insert(ModalCtx);
        if Modal::Main == on.0 {
            if !state.paused {
                commands.trigger(TogglePause);
            }
            commands.trigger(CamCursorToggle);
        }
    }

    // despawn all previous modal entities to avoid clattering
    commands.trigger(ClearModals);
    let NewModal(modal) = on.event();
    match modal {
        Modal::Main => commands.spawn(menu_modal()),
        Modal::Settings => commands.spawn(settings_modal()),
    };

    state.modals.push(modal.clone());
}

fn pop_modal(
    on: Trigger<PopModal>,
    screen: Res<State<Screen>>,
    menu_marker: Query<Entity, With<MenuModal>>,
    settings_marker: Query<Entity, With<SettingsModal>>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
) {
    if Screen::Gameplay != *screen.get() {
        return;
    }

    info!("Chat are we popping? {:?}", state.modals);
    // just a precaution
    assert!(!state.modals.is_empty());

    let popped = state.modals.pop().expect("failed to pop modal");
    match popped {
        Modal::Main => {
            if let Ok(menu) = menu_marker.single() {
                commands.entity(menu).despawn();
            }
        }
        Modal::Settings => {
            if let Ok(menu) = settings_marker.single() {
                commands.entity(menu).despawn();
            }
        }
    }

    // respawn next in the modal stack
    if let Some(modal) = state.modals.last() {
        match modal {
            Modal::Main => commands.spawn(menu_modal()),
            Modal::Settings => commands.spawn(settings_modal()),
        };
    }

    if state.modals.is_empty() {
        info!("PopModal target entity: {}", on.target());
        commands.entity(on.target()).insert(ModalCtx);
        commands.trigger(TogglePause);
        commands.trigger(CamCursorToggle);
    }
}

fn clear_modals(
    _: Trigger<ClearModals>,
    state: ResMut<GameState>,
    menu_marker: Query<Entity, With<MenuModal>>,
    settings_marker: Query<Entity, With<SettingsModal>>,
    mut cmds: Commands,
) {
    for m in &state.modals {
        match m {
            Modal::Main => {
                if let Ok(modal) = menu_marker.single() {
                    cmds.entity(modal).despawn();
                }
            }
            Modal::Settings => {
                if let Ok(modal) = settings_marker.single() {
                    cmds.entity(modal).despawn();
                }
            }
        }
    }
}

// MODALS

fn settings_modal() -> impl Bundle {
    (StateScoped(Screen::Gameplay), SettingsModal, settings_ui())
}

fn menu_modal() -> impl Bundle {
    let opts = Opts::new("Settings")
        .width(Vw(15.0))
        .padding(UiRect::axes(Vw(2.0), Vw(0.5)));
    (
        StateScoped(Screen::Gameplay),
        MenuModal,
        ui_root("In game menu"),
        children![(
            BorderColor(WHITEISH),
            BackgroundColor(TRANSLUCENT),
            Node {
                border: UiRect::all(Px(2.0)),
                padding: UiRect::all(Vw(10.0)),
                left: Px(0.0),
                bottom: Px(0.0),
                ..default()
            },
            children![
                (
                    Node {
                        position_type: PositionType::Absolute,
                        right: Px(0.0),
                        bottom: Px(0.0),
                        ..Default::default()
                    },
                    children![btn_small(
                        Opts::new("back").width(Vw(5.0)).border(UiRect::DEFAULT),
                        click_pop_modal
                    )]
                ),
                (
                    Node {
                        row_gap: Percent(20.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        ..default()
                    },
                    children![
                        btn(opts.clone(), click_spawn_settings),
                        btn(opts.text("Main Menu"), click_to_menu)
                    ]
                )
            ]
        )],
    )
}
