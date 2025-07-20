//! An abstraction for changing mood of the game depending on some triggers
use super::*;
use rand::prelude::*;

const FADE_TIME: f32 = 2.0;

pub fn plugin(app: &mut App) {
    app.add_systems(OnExit(Screen::Gameplay), stop_soundtrack)
        .add_systems(OnEnter(Screen::Gameplay), start_soundtrack)
        // .add_systems(
        //     Update,
        //     (fade_in, fade_out).run_if(in_state(Screen::Gameplay)),
        // )
        .add_observer(change_mood);
}

// TODO: implement different music states
// TODO: basic track/mood change per zone
// good structure in this example: <https://github.com/bevyengine/bevy/blob/main/examples/audio/soundtrack.rs#L29>
fn start_soundtrack(
    mut cmds: Commands,
    settings: Res<Settings>,
    sources: ResMut<AudioSources>,
    // boombox: Query<Entity, With<Boombox>>,
) {
    let mut rng = thread_rng();
    let handle = sources.explore.choose(&mut rng).unwrap();

    // // Play music from boombox entity
    // cmds
    //     .entity(boombox.single()?)
    //     .insert(music(handle.clone(), settings.music());
    // Or just play music
    cmds.spawn((
        Music,
        SamplePlayer::new(handle.clone())
            .with_volume(settings.music())
            .looping(),
    ));
}

fn stop_soundtrack(
    // boombox: Query<Entity, With<Boombox>>,
    mut bg_music: Query<&mut PlaybackSettings, With<Music>>,
) {
    for mut s in bg_music.iter_mut() {
        info!("pause track:{s:?}");
        s.pause();
    }
}

// Every time the GameState resource changes, this system is run to trigger the song change.
fn change_mood(
    on: Trigger<ChangeMood>,
    settings: Res<Settings>,
    sources: Res<AudioSources>,
    music: Query<Entity, (With<SamplerPool<Music>>, Without<SamplerPool<Sfx>>)>,
    mut commands: Commands,
) {
    let mood = &on.0;
    let mut rng = thread_rng();

    // Fade out all currently running tracks
    for track in music.iter() {
        commands.entity(track).insert(FadeOut);
    }

    // Spawn a new music with the appropriate soundtrack based on new mood
    // Volume is set to start at zero and is then increased by the fade_in system.
    match mood {
        MoodType::Exploration => {
            let handle = sources.explore.choose(&mut rng).unwrap();
            commands.spawn((
                Music,
                SamplePlayer::new(handle.clone())
                    .with_volume(settings.music())
                    .looping(),
                FadeIn,
            ));
        }
        MoodType::Combat => {
            let handle = sources.combat.choose(&mut rng).unwrap();
            commands.spawn((
                Music,
                SamplePlayer::new(handle.clone())
                    .with_volume(settings.music())
                    .looping(),
                FadeIn,
            ));
        }
    }
}

// Fades in the audio of entities that has the FadeIn component. Removes the FadeIn component once
// full volume is reached.
// fn fade_in(
//     time: Res<Time>,
//     mut commands: Commands,
//     mut music: Query<(&mut VolumeNode, Entity), With<FadeIn>>,
// ) {
//     for (mut audio, entity) in music.iter_mut() {
//         let current_volume = audio.volume;
//         audio.set_volume(
//             current_volume.fade_towards(Volume::Linear(1.0), time.delta_secs() / FADE_TIME),
//         );
//         if audio.volume().to_linear() >= 1.0 {
//             audio.set_volume(Volume::Linear(1.0));
//             commands.entity(entity).remove::<FadeIn>();
//         }
//     }
// }

// Fades out the audio of entities that has the FadeOut component. Despawns the entities once audio
// volume reaches zero.
// fn fade_out(
//     time: Res<Time>,
//     mut commands: Commands,
//     mut music: Query<(&mut VolumeNode, Entity), With<FadeOut>>,
// ) {
//     for (mut audio, entity) in music.iter_mut() {
//         let current_volume = audio.volume;
//         audio.set_volume(
//             current_volume.fade_towards(Volume::Linear(0.0), time.delta_secs() / FADE_TIME),
//         );
//         if audio.volume().to_linear() <= 0.0 {
//             commands.entity(entity).despawn();
//         }
//     }
// }
