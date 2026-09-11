//! The screen state for the main gameplay.
//!
//! Place to all UI HUD, modal logic and other gameplay effects

use super::*;
use bevy_enhanced_input::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ui::modal::plugin)
        .add_systems(
            OnEnter(Screen::Gameplay),
            (spawn_gameplay_ui, deactivate_global_modal_ctx),
        )
        .add_observer(toggle_mute)
        .add_observer(toggle_pause)
        .add_observer(trigger_menu_toggle_on_esc);
}

markers!(PauseIcon, MuteIcon, GameplayUi);

fn spawn_gameplay_ui(mut commands: Commands, textures: Res<Textures>, _settings: Res<Settings>) {
    // info!("settings on gameplay enter:{settings:?}");
    let ico = Props::default().hidden().width(Vw(1.0)).height(Vw(5.0));
    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        GameplayUi,
        widget::ui_root("Gameplay Ui"),
        children![
            // mute/pause icons
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    position_type: PositionType::Absolute,
                    top: Vh(5.0),
                    left: Vw(47.5),
                    ..Default::default()
                },
                children![
                    (
                        widget::icon(ico.clone().image(textures.pause.clone())),
                        PauseIcon
                    ),
                    (widget::icon(ico.image(textures.mute.clone())), MuteIcon),
                ]
            ),
        ],
    ));
}

fn toggle_pause(
    _: On<TogglePause>,
    mut state: ResMut<GameState>,
    mut time: ResMut<Time<Virtual>>,
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
    trace!("paused: {}", state.paused);
}

fn toggle_mute(
    _: On<ToggleMute>,
    settings: ResMut<Settings>,
    mut state: ResMut<GameState>,
    mut label: Query<&mut Node, With<MuteIcon>>,
    mut music: Single<&mut VolumeNode, (With<MusicPool>, Without<SoundEffectsBus>)>,
    mut sfx: Single<&mut VolumeNode, (With<SoundEffectsBus>, Without<MusicPool>)>,
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
    debug!("muted: {}", state.muted);
}

// ============================ UI ============================

fn trigger_menu_toggle_on_esc(
    on: On<Back>,
    mut commands: Commands,
    screen: Res<State<Screen>>,
    modals: If<ResMut<Modals>>,
) {
    if !screen.get().is_gameplay() {
        return;
    }

    debug!(
        "on back and in gameplay: e-{}, modals:{modals:?}",
        on.entity
    );

    if modals.is_empty() {
        info!("triggering main menu");
        push_modal(&mut commands, on.entity, menu_modal());
    } else {
        info!("popping modal");
        commands.entity(on.entity).trigger(PopModal);
    }
}

fn deactivate_global_modal_ctx(
    mut commands: Commands,
    main_menu_ctx: Single<Entity, With<MainMenuCtx>>,
) {
    commands
        .entity(*main_menu_ctx)
        .insert(ContextActivity::<ModalInput>::INACTIVE);
    // commands.entity(*main_menu_ctx).trigger(ToggleCamCursor);
}
