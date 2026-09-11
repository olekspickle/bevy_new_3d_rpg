//! A gameplay-only modal listing one instance of every UI primitive this project has, as a
//! living design-system reference. Toggled by [`toggle_ui_showcase`] (Tab, or the gamepad
//! Mode/PS button — see its doc comment for why that's the closest available binding).

use super::*;
use crate::asset_loading::Textures;
use crate::player::Player;

markers!(ShowcasePanel);

pub fn showcase_panel(textures: &Textures) -> impl Bundle + use<> {
    let textures = textures.clone();
    let row = || Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Px(20.0),
        ..default()
    };
    let caption = |text: &'static str| widget::label(Props::new(text).width(Vw(10.0)));

    (
        ShowcasePanel,
        widget::ui_root("UI Showcase"),
        children![(
            BorderColor::all(colors::WHITEISH),
            BackgroundColor(colors::TRANSLUCENT),
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Px(2.0)),
                padding: UiRect::all(Vw(2.0)),
                row_gap: Px(14.0),
                overflow: Overflow::scroll_y(),
                max_height: Percent(90.0),
                ..default()
            },
            children![
                widget::header("UI Showcase (Tab to close)"),
                (
                    row(),
                    children![caption("label"), widget::label("A label.")]
                ),
                (
                    row(),
                    children![caption("header"), widget::header("A header.")]
                ),
                (
                    row(),
                    children![
                        caption("icon"),
                        widget::icon(
                            Props::default()
                                .width(Vw(2.0))
                                .height(Vw(2.0))
                                .image(textures.github.clone())
                        )
                    ]
                ),
                (
                    row(),
                    children![
                        caption("btn"),
                        widget::btn("btn", noop_click),
                        caption("btn_small"),
                        widget::btn_small("small", noop_click),
                        caption("btn_big"),
                        widget::btn_big("big", noop_click),
                    ]
                ),
                (
                    row(),
                    children![caption("checkbox"), widget::checkbox(true, noop_checkbox)]
                ),
                (
                    row(),
                    children![
                        caption("slider"),
                        widget::slider(ShowcaseSliderLabel, 0.0..=1.0, 0.05, 0.5, noop_slider)
                    ]
                ),
                (
                    row(),
                    children![
                        caption("text_edit"),
                        text_edit::text_edit(
                            text_edit::TextEditProps::default().with_placeholder("type here")
                        )
                    ]
                ),
            ]
        )],
    )
}

fn noop_click(_: On<Pointer<Click>>) {}
fn noop_checkbox(_: On<bevy::ui_widgets::ValueChange<bool>>) {}
fn noop_slider(_: On<bevy::ui_widgets::ValueChange<f32>>) {}

markers!(ShowcaseSliderLabel);

/// Toggles the [`showcase_panel`] modal on `Tab`, or on gamepad `Mode` (the PS/Home button) for
/// controller parity. Bevy's abstracted `GamepadButton` enum has no distinct touchpad-click
/// variant at all — `Select` is already bound to Back/Escape elsewhere in this project's
/// `player/input.rs`, so `Mode` is the closest available button; a true touchpad-click isn't
/// portably exposed through bevy's gamepad API.
pub fn toggle_ui_showcase(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    screen: Res<State<Screen>>,
    textures: Res<Textures>,
    showcase_q: Query<Entity, With<ShowcasePanel>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
) {
    if !screen.get().is_gameplay() {
        return;
    }

    let pressed = keyboard.just_pressed(KeyCode::Tab)
        || gamepads
            .iter()
            .any(|gamepad| gamepad.just_pressed(GamepadButton::Mode));
    if !pressed {
        return;
    }

    if showcase_q.single().is_ok() {
        commands.entity(*player).trigger(PopModal);
    } else {
        push_modal(&mut commands, *player, showcase_panel(&textures));
    }
}
