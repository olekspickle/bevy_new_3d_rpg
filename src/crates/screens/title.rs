use super::*;

/// This plugin is responsible for the game menu
/// The menu is only drawn during the State [`Screen::Title`] and is removed when that state is exited
pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), setup_menu);
}

fn setup_menu(mut commands: Commands, mut state: ResMut<GameState>) {
    commands.spawn((
        StateScoped(Screen::Title),
        ui_root("Title UI"),
        BackgroundColor(TRANSLUCENT),
        children![(
            Node {
                width: Vw(40.0),
                height: Vh(40.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Vh(1.0),
                bottom: Vw(1.0),
                left: Vw(1.0),
                ..default()
            },
            // Crutch until we can use #cfg in children![] macro
            // https://github.com/bevyengine/bevy/issues/18953
            #[cfg(target_arch = "wasm32")]
            children![
                btn_big("Play", to::gameplay_or_loading),
                btn_big("Credits", to::credits),
                btn_big("Settings", to::settings),
            ],
            #[cfg(not(target_arch = "wasm32"))]
            children![
                btn_big("Play", to::gameplay_or_loading),
                btn_big("Credits", to::credits),
                btn_big("Settings", to::settings),
                btn_big("Exit", exit_app)
            ],
        )],
    ));

    state.reset();
}

#[cfg(not(target_arch = "wasm32"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
