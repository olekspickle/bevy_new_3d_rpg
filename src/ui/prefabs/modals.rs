use super::*;

pub fn click_to_menu(_: On<Pointer<Click>>, mut commands: Commands) {
    commands.trigger(GoTo(Screen::Title));
}
pub fn click_spawn_settings(
    on: On<Pointer<Click>>,
    settings: Res<Settings>,
    cfg: Res<Config>,
    mut commands: Commands,
) {
    push_modal(&mut commands, on.entity, settings_modal(&settings, &cfg));
}

pub fn settings_modal(settings: &Settings, cfg: &Config) -> impl Bundle + use<> {
    settings_ui(settings, cfg)
}

pub fn menu_modal() -> impl Bundle {
    let opts = Props::new("Settings")
        .width(Vw(15.0))
        .padding(UiRect::axes(Vw(2.0), Vw(0.5)));
    (
        widget::ui_root("In game menu"),
        children![(
            BorderColor::all(colors::WHITEISH),
            BackgroundColor(colors::TRANSLUCENT),
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
                    children![widget::btn_small(
                        Props::new("back").width(Vw(5.0)).border(UiRect::DEFAULT),
                        crate::ui::click_pop_modal
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
                        widget::btn(opts.clone(), click_spawn_settings),
                        widget::btn(opts.text("Main Menu"), click_to_menu)
                    ]
                )
            ]
        )],
    )
}
