//! A credits screen that can be accessed from the main menu
use super::*;
use crate::asset_loading::{LoadResource, ron::RonLoadPlugin};
use bevy::ecs::{lifecycle::HookContext, spawn::SpawnIter, world::DeferredWorld};
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

/// Percent of screen height per second
const SCROLL_SPEED: f32 = 5.0;
/// Roll speed multiplier while holding "speed up"
const SPEED_UP_FACTOR: f32 = 4.0;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(RonLoadPlugin::<CreditsPreset>::default())
        .load_resource_from_path::<CreditsPreset>("credits.ron")
        .add_input_context::<CreditsInput>()
        .add_systems(
            OnEnter(Screen::Credits),
            (start_credits_music, spawn_credits_screen),
        )
        .add_systems(Update, roll_the_credits.run_if(in_state(Screen::Credits)));
}

markers!(CreditsRoot, CreditsBackBtn);

/// Up/Down (or dpad) axis: hold up to speed the roll up, hold down to stop it.
#[derive(InputAction)]
#[action_output(f32)]
pub struct AdjustRoll;

#[derive(Component, Default)]
#[component(on_add = CreditsInput::on_add)]
pub(crate) struct CreditsInput;

impl CreditsInput {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .insert(actions!(CreditsInput[(
                Action::<AdjustRoll>::new(),
                Bindings::spawn((
                    Bidirectional::new(KeyCode::ArrowUp, KeyCode::ArrowDown),
                    Bidirectional::new(GamepadButton::DPadUp, GamepadButton::DPadDown),
                )),
            )]));
    }
}

#[derive(Asset, Clone, Debug, Default, Serialize, Deserialize, Reflect, Resource)]
#[reflect(Resource)]
pub struct CreditsPreset {
    pub assets: Vec<(String, String)>,
    pub devs: Vec<(String, String)>,
}

fn spawn_credits_screen(mut commands: Commands, credits: Res<CreditsPreset>) {
    commands.spawn((
        DespawnOnExit(Screen::Credits),
        CreditsInput,
        widget::ui_root("credits screen"),
        BackgroundColor(colors::TRANSLUCENT),
        children![(
            CreditsRoot,
            Node {
                width: Percent(100.0),
                position_type: PositionType::Absolute,
                bottom: Percent(-100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::scroll_y(),
                row_gap: Vh(5.0),
                ..default()
            },
            children![
                widget::header("Created by"),
                flatten(&credits.devs),
                widget::header("Assets"),
                flatten(&credits.assets),
                (
                    widget::btn_big("Back", click_go_to),
                    Screen::Title,
                    CreditsBackBtn
                ),
            ]
        )],
    ));
}

fn flatten(devs: &[(String, String)]) -> impl Bundle {
    let devs: Vec<[String; 2]> = devs.iter().map(|(n, k)| [n.clone(), k.clone()]).collect();
    grid(devs)
}

fn grid(content: Vec<[String; 2]>) -> impl Bundle {
    let content = content.into_iter().flatten().enumerate().map(|(i, text)| {
        (
            Text(text),
            Node {
                justify_self: if i.is_multiple_of(2) {
                    JustifySelf::End
                } else {
                    JustifySelf::Start
                },
                ..default()
            },
        )
    });

    (
        Name::new("Credits Grid"),
        Node {
            display: Display::Grid,
            row_gap: Vh(1.0),
            column_gap: Vw(5.0),
            grid_template_columns: RepeatedGridTrack::vw(2, 35.0),
            ..default()
        },
        Children::spawn(SpawnIter(content)),
    )
}

fn start_credits_music(
    settings: Res<Settings>,
    mut commands: Commands,
    mut sources: ResMut<AudioSources>,
    mut music: Query<&mut PlaybackSettings, With<MusicPool>>,
) {
    for mut s in music.iter_mut() {
        s.pause();
    }

    let handle = sources.explore.pick(&mut rand::rng());
    commands.spawn((
        DespawnOnExit(Screen::Credits),
        Name::new("Credits Music"),
        MusicPool,
        SamplePlayer::new(handle.clone())
            .with_volume(settings.music())
            .looping(),
    ));
}

/// Roll the credits up from below the screen until the back button
/// reaches the center of the screen.
/// Both [`UiGlobalTransform`] (node center) and [`ComputedNode`] are in
/// physical pixels with y going down from the top of the viewport.
fn roll_the_credits(
    time: Res<Time>,
    adjust: Single<&Action<AdjustRoll>>,
    root: Single<&ComputedNode, With<CreditsInput>>,
    btn: Single<&UiGlobalTransform, With<CreditsBackBtn>>,
    mut node: Single<&mut Node, With<CreditsRoot>>,
) {
    // skip until the first layout pass produces real geometry
    let screen_center = root.size().y * 0.5;
    if screen_center <= 0.0 || btn.translation.y <= screen_center {
        return;
    }

    let adjust = ***adjust;
    let speed = if adjust < 0.0 {
        0.0 // hold down to stop
    } else {
        SCROLL_SPEED * (1.0 + adjust * (SPEED_UP_FACTOR - 1.0))
    };

    if let Percent(bottom) = node.bottom {
        node.bottom = Percent(bottom + speed * time.delta_secs());
    }
}
