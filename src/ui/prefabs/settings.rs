use super::*;
#[cfg(feature = "dev")]
use bevy::ui::Display as NodeDisplay;
use bevy::ui_widgets::ValueChange;
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
    FovLabel,
    TabBar,
    TabContent,
    PerfUi
);

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
    settings: If<Res<Settings>>,
    cfg: If<Res<Config>>,
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
                            commands
                                .spawn(audio_grid(&settings, &cfg))
                                .insert(ChildOf(e));
                        }
                        UiTab::Video => {
                            commands
                                .spawn(video_grid(&settings, &cfg))
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

// ============================ SLIDER HOOKS ============================

fn fov_changed(
    value_change: On<ValueChange<f32>>,
    mut settings: ResMut<Settings>,
    mut world_model_projection: Single<&mut Projection>,
) {
    let Projection::Perspective(perspective) = world_model_projection.as_mut() else {
        return;
    };
    perspective.fov = value_change.value.to_radians();
    settings.fov = value_change.value;
}

fn update_fov_label(settings: Res<Settings>, mut label: Single<&mut Text, With<FovLabel>>) {
    let fov = settings.fov.round();
    let text = format!("{fov: <3}"); // pad to 3 chars
    label.0 = text;
}

// GENERAL
fn general_changed(
    value_change: On<ValueChange<f32>>,
    mut settings: ResMut<Settings>,
    mut general: Single<&mut VolumeNode, With<MainBus>>,
) {
    settings.sound.general = value_change.value;
    general.volume = Volume::Linear(value_change.value);
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
fn music_changed(
    value_change: On<ValueChange<f32>>,
    mut settings: ResMut<Settings>,
    mut music: Single<&mut VolumeNode, With<SamplerPool<MusicPool>>>,
) {
    settings.sound.music = value_change.value;
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
fn sfx_changed(
    value_change: On<ValueChange<f32>>,
    mut settings: ResMut<Settings>,
    mut sfx: Single<&mut VolumeNode, With<SoundEffectsBus>>,
) {
    settings.sound.sfx = value_change.value;
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

fn vsync_changed(
    value_change: On<ValueChange<bool>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for mut window in windows.iter_mut() {
        window.present_mode = if value_change.value {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        info!(" window present_mode changed to: {:?}", window.present_mode);
    }
}

#[cfg(feature = "dev")]
fn diagnostics_changed(
    value_change: On<ValueChange<bool>>,
    mut state: ResMut<GameState>,
    mut perf_ui: Query<&mut Node, With<PerfUi>>,
) {
    state.diagnostics = value_change.value;
    if let Ok(mut perf_ui) = perf_ui.single_mut() {
        perf_ui.display = if value_change.value {
            NodeDisplay::Flex
        } else {
            NodeDisplay::None
        };
    }
}

#[cfg(feature = "dev")]
fn debug_ui_changed(
    value_change: On<ValueChange<bool>>,
    mut state: ResMut<GameState>,
    mut commands: Commands,
) {
    state.debug_ui = value_change.value;
    commands.trigger(ToggleDebugUi);
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
            widget::btn(settings.sun_cycle.as_str(), click_toggle_sun_cycle),
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

pub fn settings_ui(settings: &Settings, cfg: &Config) -> impl Bundle + use<> {
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
                (
                    TabContent,
                    Node::default(),
                    children![audio_grid(settings, cfg)]
                ),
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

fn video_grid(settings: &Settings, cfg: &Config) -> impl Bundle + use<> {
    let fov_range = cfg.settings.min_fov..=cfg.settings.max_fov;
    let fov_step = cfg.settings.step.to_degrees();
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
                widget::btn(settings.sun_cycle.as_str(), click_toggle_sun_cycle),
                SunCycleLabel
            ),
            widget::label("FOV"),
            widget::slider(FovLabel, fov_range, fov_step, settings.fov, fov_changed),
            widget::label("VSync"),
            widget::checkbox(true, vsync_changed),
        ],
        #[cfg(feature = "dev")]
        children![
            widget::label("Sun cycle"),
            (
                widget::btn(settings.sun_cycle.as_str(), click_toggle_sun_cycle),
                SunCycleLabel
            ),
            widget::label("FOV"),
            widget::slider(FovLabel, fov_range, fov_step, settings.fov, fov_changed),
            widget::label("VSync"),
            widget::checkbox(true, vsync_changed),
            widget::label("diagnostics"),
            widget::checkbox(true, diagnostics_changed),
            widget::label("debug ui"),
            widget::checkbox(false, debug_ui_changed),
        ],
    )
}

fn audio_grid(settings: &Settings, cfg: &Config) -> impl Bundle + use<> {
    let volume_range = cfg.settings.min_volume..=cfg.settings.max_volume;
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
            widget::slider(
                GeneralVolumeLabel,
                volume_range.clone(),
                cfg.settings.step,
                settings.sound.general,
                general_changed,
            ),
            widget::label("music"),
            widget::slider(
                MusicVolumeLabel,
                volume_range.clone(),
                cfg.settings.step,
                settings.sound.music,
                music_changed,
            ),
            widget::label("sfx"),
            widget::slider(
                SfxVolumeLabel,
                volume_range,
                cfg.settings.step,
                settings.sound.sfx,
                sfx_changed,
            ),
        ],
    )
}
