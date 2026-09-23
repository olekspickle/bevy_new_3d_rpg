use super::*;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

const FADE_TIME: f64 = 0.5; // seconds

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        crossfade_music.run_if(on_timer(Duration::from_secs_f64(FADE_TIME))),
    );
}

markers!(FadeIn, FadeOut);

fn crossfade_music(
    settings: Res<Settings>,
    mut commands: Commands,
    mut pb_settings: Query<&mut PlaybackSettings>,
    mut volume_nodes: Query<(&VolumeNode, &mut AudioEvents)>,
    mut fade_out: Query<(Entity, &SampleEffects), (With<FadeOut>, Without<FadeIn>)>,
    mut fade_in: Query<(Entity, &SampleEffects), (With<FadeIn>, Without<FadeOut>)>,
) {
    let fade_duration = DurationSeconds(FADE_TIME);

    for (e, effects) in fade_out.iter_mut() {
        let Ok((node, mut events)) = volume_nodes.get_effect_mut(effects) else {
            continue;
        };
        let Ok(mut pb) = pb_settings.get_mut(e) else {
            continue;
        };

        let mut audio = commands.entity(e);
        // to prevent doing both fades in case both components somehow made it to an entity
        // I prefer FadeOut to not cause a cacophony of sounds
        // TODO: figure out cleaner way to do this
        audio.remove::<FadeIn>();

        if node.volume.linear() <= 0.01 {
            audio.remove::<FadeOut>();
            pb.pause();
            // debug!("Paused music, rm FadeOut: {e}");
            continue;
        }

        // debug!("fading out: {e}");
        node.fade_to(Volume::SILENT, fade_duration, &mut events);
    }

    for (e, effects) in fade_in.iter_mut() {
        let Ok((node, mut events)) = volume_nodes.get_effect_mut(effects) else {
            continue;
        };
        if node.volume.linear() >= settings.music().linear() {
            commands.entity(e).remove::<FadeIn>();
            // debug!("rm FadeIn: {e}");
            continue;
        }

        let Ok(mut pb) = pb_settings.get_mut(e) else {
            continue;
        };
        // debug!("fading in: {e}");
        node.fade_to(settings.music(), fade_duration, &mut events);
        pb.play();
    }
}
