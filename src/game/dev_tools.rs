//! Development tools for the game. This plugin is only enabled in dev builds.
use super::*;
use crate::camera::SceneCamera;
use crate::shared::GameState;
use bevy::camera::RenderTarget;
use bevy::dev_tools::states::log_transitions;
use bevy::input::common_conditions::input_just_pressed;
use bevy::window::{WindowClosed, WindowRef};
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin, bevy_egui::EguiContext, bevy_egui::EguiGlobalSettings,
    bevy_egui::EguiPlugin, bevy_egui::EguiPrimaryContextPass, bevy_egui::PrimaryEguiContext,
    bevy_egui::egui, bevy_inspector,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default())
        .insert_resource(EguiGlobalSettings {
            auto_create_primary_context: false,
            ..default()
        })
        .init_resource::<InspectorWindow>()
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(
            EguiPrimaryContextPass,
            world_inspector.run_if(|state: Res<GameState>| state.inspector_enabled),
        )
        .add_systems(
            Update,
            (
                log_transitions::<Screen>,
                toggle_inspector.run_if(input_just_pressed(KeyCode::Backquote)),
                close_inspector_window,
            ),
        )
        .add_observer(toggle_debug_ui)
        .add_observer(on_set_inspector_enabled)
        .add_observer(on_set_inspector_own_window);
}

/// Inspector's own OS window and its camera, present only while shown.
#[derive(Resource, Default)]
struct InspectorWindow(Option<(Entity, Entity)>);

fn toggle_inspector(state: Res<GameState>, mut commands: Commands) {
    commands.trigger(SetInspectorEnabled(!state.inspector_enabled));
}

fn on_set_inspector_enabled(
    on: On<SetInspectorEnabled>,
    mut state: ResMut<GameState>,
    inspector: ResMut<InspectorWindow>,
    main_camera: Single<Entity, With<SceneCamera>>,
    commands: Commands,
) {
    state.inspector_enabled = on.0;
    sync_inspector(&state, inspector, *main_camera, commands);
}

fn on_set_inspector_own_window(
    on: On<SetInspectorOwnWindow>,
    mut state: ResMut<GameState>,
    inspector: ResMut<InspectorWindow>,
    main_camera: Single<Entity, With<SceneCamera>>,
    commands: Commands,
) {
    state.inspector_own_window = on.0;
    sync_inspector(&state, inspector, *main_camera, commands);
}

/// Own window by default: egui on the main camera may draw under `LensDistortion`'s pass.
/// Spawned/despawned rather than hidden since winit can't toggle visibility on Wayland.
fn sync_inspector(
    state: &GameState,
    mut inspector: ResMut<InspectorWindow>,
    main_camera: Entity,
    mut commands: Commands,
) {
    let own_window = state.inspector_enabled && state.inspector_own_window;
    match (own_window, inspector.0) {
        (true, None) => {
            let window = commands
                .spawn(Window {
                    title: "Inspector".to_string(),
                    resolution: (400, 800).into(),
                    ..default()
                })
                .id();
            let camera = commands
                .spawn((
                    Camera2d,
                    RenderTarget::Window(WindowRef::Entity(window)),
                    PrimaryEguiContext,
                ))
                .id();
            inspector.0 = Some((window, camera));
        }
        (false, Some((window, camera))) => {
            commands.entity(camera).despawn();
            commands.entity(window).try_despawn();
            inspector.0 = None;
        }
        _ => {}
    }

    if state.inspector_enabled && !state.inspector_own_window {
        commands.entity(main_camera).insert(PrimaryEguiContext);
    } else {
        commands.entity(main_camera).remove::<PrimaryEguiContext>();
    }
}

/// Fills its own window; floats when sharing the main camera so the game stays visible.
fn world_inspector(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let own_window = world.resource::<GameState>().inspector_own_window;

    let contents = |ui: &mut egui::Ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            bevy_inspector::ui_for_world(world, ui);
            ui.allocate_space(ui.available_size());
        });
    };
    let ctx = egui_context.get_mut();
    if own_window {
        let mut viewport_ui = egui::Ui::new(
            ctx.clone(),
            "inspector viewport".into(),
            egui::UiBuilder::new()
                .layer_id(egui::LayerId::background())
                .max_rect(ctx.viewport_rect()),
        );
        egui::CentralPanel::default().show_inside(&mut viewport_ui, contents);
    } else {
        egui::Window::new("World Inspector").show(ctx, contents);
    }
}

/// Closing the inspector window via the OS hides the inspector.
fn close_inspector_window(
    mut closed: MessageReader<WindowClosed>,
    inspector: Res<InspectorWindow>,
    mut commands: Commands,
) {
    let Some((window, _)) = inspector.0 else {
        return;
    };
    if closed.read().any(|c| c.window == window) {
        commands.trigger(SetInspectorEnabled(false));
    }
}

fn toggle_debug_ui(_: On<ToggleDebugUi>, mut options: ResMut<GlobalUiDebugOptions>) {
    options.toggle();
}
