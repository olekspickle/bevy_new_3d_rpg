use super::*;
#[cfg(feature = "dev")]
use bevy::ui::Display as NodeDisplay;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy_enhanced_input::prelude::Start;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ActiveTab>();
    app.add_systems(
        Update,
        (
            update_general_volume_label,
            update_music_volume_label,
            update_sfx_volume_label,
            update_fov_label,
            update_tab_content.run_if(resource_changed::<ActiveTab>),
        ),
    )
    .add_observer(to_the_left_tab)
    .add_observer(to_the_right_tab);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect, Component)]
#[reflect(Component)]
pub enum UiTab {
    #[default]
    Audio,
    Video,
    Keybindings,
}

#[derive(Resource, Default)]
pub struct ActiveTab(pub UiTab);

markers!(
    GeneralVolumeLabel,
    MusicVolumeLabel,
    SfxVolumeLabel,
    SunCycleLabel,
    SaveSettingsLabel,
    VsyncLabel,
    FovLabel,
    TabBar,
    TabContent,
    PerfUi
);
#[cfg(feature = "dev")]
markers!(DiagnosticsLabel, DebugUiLabel);

// ============================ CONTROL KNOBS OBSERVERS ============================

pub fn save_settings(
    _: On<Pointer<Click>>,
    settings: Res<Settings>,
    children_q: Query<&Children>,
    root: Query<&Children, With<SaveSettingsLabel>>,
    mut text_q: Query<&mut Text>,
) {
    // TODO: this is an insane nesting, improve it
    match settings.save() {
        Ok(()) => {
            info!("writing settings to '{SETTINGS_PATH}'");
            if let Ok(children) = root.single() {
                for child in children.iter() {
                    if let Ok(grandchildren) = children_q.get(child) {
                        for gc in grandchildren.iter() {
                            if let Ok(mut label) = text_q.get_mut(gc) {
                                label.0 = "Saved!".to_string();
                            }
                        }
                    }
                }
            }
        }
        Err(e) => error!("unable to write settings to '{SETTINGS_PATH}': {e}"),
    }
}

// TAB CHANGING
fn update_tab_content(
    settings: Res<Settings>,
    active_tab: Res<ActiveTab>,
    tab_bar: Query<&Children, With<TabBar>>,
    mut tab_content: Query<(Entity, &Children), With<TabContent>>,
    mut buttons: Query<(&UiTab, &mut Node)>,
    mut commands: Commands,
) -> Result {
    for children in &tab_bar {
        for &child in children {
            if let Ok((tab, mut node)) = buttons.get_mut(child) {
                if *tab == active_tab.0 {
                    node.border.bottom = Px(0.0);

                    let (e, content) = tab_content.single_mut()?;
                    for child in content.iter() {
                        commands.entity(child).despawn();
                    }
                    match tab {
                        UiTab::Audio => {
                            commands.spawn(audio_grid()).insert(ChildOf(e));
                        }
                        UiTab::Video => {
                            commands
                                .spawn(video_grid(&settings.sun_cycle))
                                .insert(ChildOf(e));
                        }
                        UiTab::Keybindings => {
                            commands
                                .spawn(keybind_editor(&settings.input_map))
                                .insert(ChildOf(e));
                        }
                    }
                } else {
                    node.border.bottom = Px(10.0);
                }
            }
        }
    }

    Ok(())
}

// ============================ +/- BUTTON HOOKS ============================

fn fov_lower(
    _: On<Pointer<Click>>,
    cfg: Res<Config>,
    mut settings: ResMut<Settings>,
    mut world_model_projection: Single<&mut Projection>,
) {
    let Projection::Perspective(perspective) = world_model_projection.as_mut() else {
        return;
    };
    let new_fov = (settings.fov - cfg.settings.step.to_degrees()).max(cfg.settings.min_fov);
    perspective.fov = new_fov.to_radians();
    settings.fov = perspective.fov.to_degrees();
}

fn fov_raise(
    _: On<Pointer<Click>>,
    cfg: Res<Config>,
    mut settings: ResMut<Settings>,
    mut world_model_projection: Single<&mut Projection>,
) {
    let Projection::Perspective(perspective) = world_model_projection.as_mut() else {
        return;
    };
    let new_fov = (settings.fov + cfg.settings.step.to_degrees()).min(cfg.settings.max_fov);
    perspective.fov = new_fov.to_radians();
    settings.fov = perspective.fov.to_degrees();
}

fn update_fov_label(settings: Res<Settings>, mut label: Single<&mut Text, With<FovLabel>>) {
    let fov = settings.fov.round();
    let text = format!("{fov: <3}"); // pad to 3 chars
    label.0 = text;
}

// GENERAL
fn general_lower(
    _: On<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut general: Single<&mut VolumeNode, With<MainBus>>,
) {
    let new_volume = (settings.sound.general - cfg.settings.step).max(cfg.settings.min_volume);
    settings.sound.general = new_volume;
    general.volume = Volume::Linear(new_volume);
}

fn general_raise(
    _: On<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut general: Single<&mut VolumeNode, With<MainBus>>,
) {
    let new_volume = (settings.sound.general + cfg.settings.step).min(cfg.settings.max_volume);
    settings.sound.general = new_volume;
    general.volume = Volume::Linear(new_volume);
}

fn update_general_volume_label(
    settings: Res<Settings>,
    mut label: Single<&mut Text, With<GeneralVolumeLabel>>,
) {
    let percent = (settings.sound.general * 100.0).round();
    let text = format!("{percent: <3}%"); // pad the percent to 3 chars
    label.0 = text;
}

// MUSIC
fn music_lower(
    _: On<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut music: Single<&mut VolumeNode, With<SamplerPool<MusicPool>>>,
) {
    let new_volume = (settings.sound.music - cfg.settings.step).max(cfg.settings.min_volume);
    settings.sound.music = new_volume;
    music.volume = settings.music();
}

fn music_raise(
    _: On<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut music: Single<&mut VolumeNode, With<SamplerPool<MusicPool>>>,
) {
    let new_volume = (settings.sound.music + cfg.settings.step).min(cfg.settings.max_volume);
    settings.sound.music = new_volume;
    music.volume = settings.music();
}

fn update_music_volume_label(
    settings: Res<Settings>,
    mut label: Single<&mut Text, With<MusicVolumeLabel>>,
) {
    let percent = (settings.sound.music * 100.0).round();
    let text = format!("{percent: <3}%"); // pad the percent to 3 chars
    label.0 = text;
}

// SFX
fn sfx_lower(
    _: On<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut sfx: Single<&mut VolumeNode, With<SoundEffectsBus>>,
) {
    let new_volume = (settings.sound.sfx - cfg.settings.step).max(cfg.settings.min_volume);
    settings.sound.sfx = new_volume;
    sfx.volume = settings.sfx();
}

fn sfx_raise(
    _: On<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut sfx: Single<&mut VolumeNode, With<SoundEffectsBus>>,
) {
    let new_volume = (settings.sound.sfx + cfg.settings.step).min(cfg.settings.max_volume);
    settings.sound.sfx = new_volume;
    sfx.volume = settings.sfx();
}

fn update_sfx_volume_label(
    settings: Res<Settings>,
    mut label: Single<&mut Text, With<SfxVolumeLabel>>,
) {
    let percent = (settings.sound.sfx * 100.0).round();
    let text = format!("{percent: <3}%"); // pad the percent to 3 chars
    label.0 = text;
}

// ============================ OTHER BUTTON HOOKS ============================

fn switch_to_tab(tab: UiTab) -> impl Fn(On<Pointer<Click>>, ResMut<ActiveTab>) + Clone {
    move |_: On<Pointer<Click>>, mut active_tab: ResMut<ActiveTab>| {
        active_tab.0 = tab;
    }
}

fn to_the_left_tab(_: On<Start<CycleTabBack>>, mut active_tab: ResMut<ActiveTab>) {
    match active_tab.0 {
        UiTab::Audio => active_tab.0 = UiTab::Keybindings,
        UiTab::Video => active_tab.0 = UiTab::Audio,
        UiTab::Keybindings => active_tab.0 = UiTab::Video,
    }
}

fn to_the_right_tab(_: On<Start<CycleTab>>, mut active_tab: ResMut<ActiveTab>) {
    match active_tab.0 {
        UiTab::Audio => active_tab.0 = UiTab::Video,
        UiTab::Video => active_tab.0 = UiTab::Keybindings,
        UiTab::Keybindings => active_tab.0 = UiTab::Audio,
    }
}

fn click_toggle_vsync(
    _: On<Pointer<Click>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) -> Result {
    for mut window in windows.iter_mut() {
        if matches!(window.present_mode, PresentMode::AutoVsync) {
            window.present_mode = PresentMode::AutoNoVsync;
        } else {
            window.present_mode = PresentMode::AutoVsync;
        }
        info!(" window present_mode changed to: {:?}", window.present_mode);
    }

    Ok(())
}

#[cfg(feature = "dev")]
fn click_toggle_diagnostics(
    _: On<Pointer<Click>>,
    mut state: ResMut<GameState>,
    mut perf_ui: Query<&mut Node, With<PerfUi>>,
    mut label: Query<&mut Text, With<DiagnosticsLabel>>,
) {
    if let Ok(mut perf_ui) = perf_ui.single_mut() {
        state.diagnostics = !state.diagnostics;
        let s = if perf_ui.display == NodeDisplay::None {
            perf_ui.display = NodeDisplay::Flex;
            "on"
        } else {
            perf_ui.display = NodeDisplay::None;
            "off"
        };

        if let Ok(mut label) = label.single_mut() {
            label.0 = s.to_owned();
        }
    }
}

#[cfg(feature = "dev")]
fn click_toggle_debug_ui(
    _: On<Pointer<Click>>,
    children: Query<&Children>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut label: Query<Entity, With<DebugUiLabel>>,
) {
    state.debug_ui = !state.debug_ui;

    if let Ok(mut label) = label.single_mut() {
        commands.trigger(ToggleDebugUi);
        let s = if state.debug_ui { "on" } else { "off" };
        label.replace_recursive(
            children,
            commands,
            (widget::btn(s, click_toggle_debug_ui), DebugUiLabel),
        );
    }
}

fn click_toggle_sun_cycle(
    _: On<Pointer<Click>>,
    children: Query<&Children>,
    commands: Commands,
    mut settings: ResMut<Settings>,
    mut labels: Query<Entity, With<SunCycleLabel>>,
) {
    match settings.sun_cycle {
        SunCycle::Nimbus => settings.sun_cycle = SunCycle::DayNight,
        SunCycle::DayNight => settings.sun_cycle = SunCycle::Nimbus,
    }

    if let Ok(mut label) = labels.single_mut() {
        label.replace_recursive(
            children,
            commands,
            (
                widget::btn(settings.sun_cycle.as_str(), click_toggle_sun_cycle),
                SunCycleLabel,
            ),
        );
    }
}

fn click_toggle_settings(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    screen: Res<State<Screen>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if *screen.get() == Screen::Settings {
        next_screen.set(Screen::Title);
    } else {
        commands.entity(click.event_target()).trigger(PopModal);
    }
}

// ============================ UI ============================

pub fn settings_ui() -> impl Bundle {
    (
        widget::ui_root("Settings Screen"),
        BackgroundColor(colors::TRANSLUCENT),
        children![(
            Node {
                width: Percent(80.0),
                height: Percent(80.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                tab_bar(),
                (TabContent, Node::default(), children![audio_grid()]),
                bottom_row()
            ]
        )],
    )
}

fn tab_bar() -> impl Bundle {
    let opts = Props::default().border_radius(Px(0.0));
    (
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            top: Vh(2.0),
            ..default()
        },
        children![
            widget::header("Settings"),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    width: Percent(100.0),
                    ..default()
                },
                TabBar,
                children![
                    (
                        widget::btn(opts.clone().text("Audio"), switch_to_tab(UiTab::Audio)),
                        UiTab::Audio
                    ),
                    (
                        widget::btn(opts.clone().text("Video"), switch_to_tab(UiTab::Video)),
                        UiTab::Video
                    ),
                    (
                        widget::btn(opts.text("Keybindings"), switch_to_tab(UiTab::Keybindings)),
                        UiTab::Keybindings
                    ),
                ],
            ),
        ],
    )
}

fn bottom_row() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            width: Percent(50.0),
            bottom: Vh(1.0),
            ..default()
        },
        children![
            (widget::btn("Save", save_settings), SaveSettingsLabel),
            widget::btn("Back", click_toggle_settings),
        ],
    )
}

fn video_grid(cycle: &SunCycle) -> impl Bundle {
    (
        Name::new("Settings Video Grid"),
        Node {
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::vw(4, 20.0),
            justify_items: JustifyItems::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        #[cfg(not(feature = "dev"))]
        children![
            widget::label("Sun cycle"),
            (
                widget::btn(cycle.as_str(), click_toggle_sun_cycle),
                SunCycleLabel
            ),
            widget::label("FOV"),
            widget::plus_minus_bar(FovLabel, fov_lower, fov_raise),
            // TODO: do checkboxes when feathers
            widget::label("VSync"),
            (widget::btn("on", click_toggle_vsync), VsyncLabel),
        ],
        #[cfg(feature = "dev")]
        children![
            widget::label("Sun cycle"),
            (
                widget::btn(cycle.as_str(), click_toggle_sun_cycle),
                SunCycleLabel
            ),
            widget::label("FOV"),
            widget::plus_minus_bar(FovLabel, fov_lower, fov_raise),
            // TODO: do checkboxes when feathers
            widget::label("VSync"),
            (widget::btn("on", click_toggle_vsync), VsyncLabel),
            widget::label("diagnostics"),
            (
                widget::btn("on", click_toggle_diagnostics),
                DiagnosticsLabel
            ),
            widget::label("debug ui"),
            (widget::btn("off", click_toggle_debug_ui), DebugUiLabel),
        ],
    )
}

fn audio_grid() -> impl Bundle {
    (
        Name::new("Settings Grid"),
        Node {
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        children![
            widget::label("general"),
            widget::plus_minus_bar(GeneralVolumeLabel, general_lower, general_raise),
            widget::label("music"),
            widget::plus_minus_bar(MusicVolumeLabel, music_lower, music_raise),
            widget::label("sfx"),
            widget::plus_minus_bar(SfxVolumeLabel, sfx_lower, sfx_raise),
        ],
    )
}
