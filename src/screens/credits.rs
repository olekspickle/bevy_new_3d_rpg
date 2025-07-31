//! A credits screen that can be accessed from the main menu

use super::*;
use bevy::ecs::spawn::SpawnIter;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Credits),
        (start_credits_music, spawn_credits_screen),
    );
}

fn spawn_credits_screen(mut commands: Commands, credits: Res<Credits>) {
    commands.spawn((
        StateScoped(Screen::Credits),
        Name::new("Credits Screen"),
        Node {
            width: Percent(100.0),
            height: Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::scroll_y(),
            row_gap: Vh(5.0),
            ..default()
        },
        BackgroundColor(TRANSLUCENT),
        children![
            header("Breated by"),
            flatten(&credits.devs),
            header("Assets"),
            flatten(&credits.assets),
            btn_big("Back", to::title),
        ],
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
    mut commands: Commands,
    settings: Res<Settings>,
    sources: ResMut<AudioSources>,
    mut music: Query<&mut PlaybackSettings, With<Music>>,
) {
    for mut s in music.iter_mut() {
        s.pause();
    }

    let handle = sources.explore[0].clone();
    commands.spawn((
        StateScoped(Screen::Credits),
        Name::new("Credits Music"),
        Music,
        SamplePlayer::new(handle)
            .with_volume(settings.music())
            .looping(),
    ));
}
