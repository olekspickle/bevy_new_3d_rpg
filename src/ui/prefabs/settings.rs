use super::*;
#[cfg(feature = "dev_native")]
use bevy::ui::Display as NodeDisplay;
use bevy::window::{PresentMode, PrimaryWindow};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            update_general_volume_label,
            update_music_volume_label,
            update_sfx_volume_label,
            update_fov_label,
            update_tab_content.run_if(resource_changed::<ActiveTab>),
        ),
    );
}

// ============================ CONTROL KNOBS OBSERVERS ============================

pub fn save_settings(
    _: Trigger<OnPress>,
    settings: Res<Settings>,
    root: Query<&Children, With<SaveSettingsLabel>>,
    children_q: Query<&Children>,
    mut text_q: Query<&mut Text>,
) {
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
                                .spawn(keybind_editor(&settings.keybind))
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

fn lower_fov(
    _: Trigger<Pointer<Click>>,
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

fn raise_fov(
    _: Trigger<Pointer<Click>>,
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
fn lower_general(
    _: Trigger<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut general: Single<&mut VolumeNode, With<MainBus>>,
) {
    let new_volume = (settings.sound.general - cfg.settings.step).max(cfg.settings.min_volume);
    settings.sound.general = new_volume;
    general.volume = Volume::Linear(new_volume);
}

fn raise_general(
    _: Trigger<Pointer<Click>>,
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
fn lower_music(
    _: Trigger<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut music: Single<&mut VolumeNode, With<SamplerPool<Music>>>,
) {
    let new_volume = (settings.sound.music - cfg.settings.step).max(cfg.settings.min_volume);
    settings.sound.music = new_volume;
    music.volume = settings.music();
}

fn raise_music(
    _: Trigger<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut music: Single<&mut VolumeNode, With<SamplerPool<Music>>>,
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
fn lower_sfx(
    _: Trigger<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut sfx: Single<&mut VolumeNode, With<SamplerPool<Sfx>>>,
) {
    let new_volume = (settings.sound.sfx - cfg.settings.step).max(cfg.settings.min_volume);
    settings.sound.sfx = new_volume;
    sfx.volume = settings.sfx();
}

fn raise_sfx(
    _: Trigger<Pointer<Click>>,
    cfg: ResMut<Config>,
    mut settings: ResMut<Settings>,
    mut sfx: Single<&mut VolumeNode, With<SamplerPool<Sfx>>>,
) {
    let new_volume = (settings.sound.sfx + cfg.settings.step).min(cfg.settings.max_volume);
    settings.sound.sfx = new_volume;
    sfx.volume = settings.sfx();
}

fn update_sfx_volume_label(
    mut label: Single<&mut Text, With<SfxVolumeLabel>>,
    settings: Res<Settings>,
) {
    let percent = (settings.sound.sfx * 100.0).round();
    let text = format!("{percent: <3}%"); // pad the percent to 3 chars
    label.0 = text;
}

// ============================ OTHER BUTTON HOOKS ============================

fn switch_to_tab(tab: UiTab) -> impl Fn(Trigger<Pointer<Click>>, ResMut<ActiveTab>) + Clone {
    move |_: Trigger<Pointer<Click>>, mut active_tab: ResMut<ActiveTab>| {
        active_tab.0 = tab;
    }
}

fn click_toggle_vsync(
    _: Trigger<Pointer<Click>>,
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

#[cfg(not(target_arch = "wasm32"))]
fn click_toggle_diagnostics(
    _: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut perf_ui: Query<&mut Node, With<PerfUi>>,
    mut label: Query<&mut Text, With<DiagnosticsLabel>>,
) {
    if let Ok(mut perf_ui) = perf_ui.single_mut() {
        if perf_ui.display == NodeDisplay::None {
            perf_ui.display = NodeDisplay::Flex;
            if let Ok(mut label) = label.single_mut() {
                label.0 = "on".to_owned();
            }
        } else {
            perf_ui.display = NodeDisplay::None;
            if let Ok(mut label) = label.single_mut() {
                label.0 = "off".to_owned();
            }
        }

        if let Ok(label) = label.single_mut() {
            info!("new label: {}", label.0);
        }
        state.diagnostics = !state.diagnostics;
        commands.trigger(ToggleDiagnostics);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clock_toggle_debug_ui(
    _: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut label: Query<&mut Text, With<DiagnosticsLabel>>,
) {
    state.debug_ui = !state.debug_ui;

    if let Ok(mut label) = label.single_mut() {
        if state.debug_ui {
            label.0 = "on".to_owned();
        } else {
            label.0 = "off".to_owned();
        }
        info!("new label: {}", label.0);
    }
    commands.trigger(ToggleDebugUi);
}

fn click_toggle_sun_cycle(
    _: Trigger<Pointer<Click>>,
    labels: Query<&Children, With<SunCycleLabel>>,
    mut texts: Query<&mut Text>,
    mut settings: ResMut<Settings>,
) {
    match settings.sun_cycle {
        SunCycle::Nimbus => {
            settings.sun_cycle = SunCycle::DayNight;
        }
        SunCycle::DayNight => {
            settings.sun_cycle = SunCycle::Nimbus;
        }
    }
    for children in labels.iter() {
        for child_entity in children.iter() {
            // Try to get the Text component from the child entity
            if let Ok(mut label) = texts.get_mut(child_entity) {
                label.0 = settings.sun_cycle.as_str().to_owned();
                info!("new sun cycle:{}", label.0);
            }
        }
    }
}

fn click_toggle_settings(
    _: Trigger<OnPress>,
    mut cmds: Commands,
    screen: Res<State<Screen>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if *screen.get() == Screen::Settings {
        next_screen.set(Screen::Title);
    } else {
        cmds.trigger(PopModal);
    }
}

// ============================ UI ============================

pub fn settings_ui() -> impl Bundle {
    (
        ui_root("Settings Screen"),
        BackgroundColor(TRANSLUCENT),
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
                navigation()
            ]
        )],
    )
}

fn tab_bar() -> impl Bundle {
    let opts = Opts::default().border_radius(Px(0.0));
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
            header("Settings"),
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
                        btn(opts.clone().text("Audio"), switch_to_tab(UiTab::Audio)),
                        UiTab::Audio
                    ),
                    (
                        btn(opts.clone().text("Video"), switch_to_tab(UiTab::Video)),
                        UiTab::Video
                    ),
                    (
                        btn(opts.text("Keybindings"), switch_to_tab(UiTab::Keybindings)),
                        UiTab::Keybindings
                    ),
                ],
            ),
        ],
    )
}

fn navigation() -> impl Bundle {
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
            (btn("Save", save_settings), SaveSettingsLabel),
            btn("Back", click_toggle_settings),
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
        #[cfg(target_arch = "wasm32")]
        children![
            label("Sun cycle"),
            (btn(cycle.as_str(), click_toggle_sun_cycle), SunCycleLabel),
            label("FOV"),
            fov(),
            // TODO: do checkboxes when feathers
            label("VSync"),
            (btn("on", click_toggle_vsync), VsyncLabel),
        ],
        #[cfg(not(target_arch = "wasm32"))]
        children![
            label("Sun cycle"),
            (btn(cycle.as_str(), click_toggle_sun_cycle), SunCycleLabel),
            label("FOV"),
            fov(),
            // TODO: do checkboxes when feathers
            label("VSync"),
            (btn("on", click_toggle_vsync), VsyncLabel),
            label("diagnostics"),
            (btn("on", click_toggle_diagnostics), DiagnosticsLabel),
            label("debug ui"),
            (btn("off", clock_toggle_debug_ui), DebugUiLabel),
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
            label("general"),
            general_volume(),
            label("music"),
            music_volume(),
            label("sfx"),
            sfx_volume(),
        ],
    )
}

fn general_volume() -> impl Bundle {
    (
        Node {
            justify_self: JustifySelf::Center,
            ..Default::default()
        },
        children![
            btn_small("-", lower_general),
            knob_label(GeneralVolumeLabel),
            btn_small("+", raise_general),
        ],
    )
}

// TODO: fov slider when feathers
fn fov() -> impl Bundle {
    (
        knobs_container(),
        children![
            btn_small("-", lower_fov),
            knob_label(FovLabel),
            btn_small("+", raise_fov),
        ],
    )
}

fn music_volume() -> impl Bundle {
    (
        knobs_container(),
        children![
            btn_small("-", lower_music),
            knob_label(MusicVolumeLabel),
            btn_small("+", raise_music),
        ],
    )
}

fn sfx_volume() -> impl Bundle {
    (
        knobs_container(),
        children![
            btn_small("-", lower_sfx),
            knob_label(SfxVolumeLabel),
            btn_small("+", raise_sfx),
        ],
    )
}

fn knob_label(knob: impl Component) -> impl Bundle {
    (
        Node {
            padding: UiRect::horizontal(Px(10.0)),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        children![(label(""), knob)],
    )
}

fn knobs_container() -> impl Bundle {
    Node {
        justify_self: JustifySelf::Center,
        align_content: AlignContent::SpaceEvenly,
        min_width: Px(100.0),
        ..Default::default()
    }
}
