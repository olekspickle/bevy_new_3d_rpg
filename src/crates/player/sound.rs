use super::*;
use audio::*;
use bevy_seedling::prelude::*;
use rand::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(movement_sound)
        .add_observer(dash_sound)
        .add_observer(jump_sound);
}

fn movement_sound(
    on: Trigger<Fired<Navigate>>,
    time: Res<Time>,
    state: Res<GameState>,
    settings: Res<Settings>,
    sources: ResMut<AudioSources>,
    tnua: Query<&TnuaController, With<Player>>,
    actions: Single<&Actions<GameplayCtx>>,
    mut cmds: Commands,
    mut step_timer: Query<&mut StepTimer, With<Player>>,
) -> Result {
    if state.muted || state.paused {
        return Ok(());
    }

    let controller = tnua.get(on.target())?;
    let mut step_timer = step_timer.get_mut(on.target())?;

    let Some((_, basis)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
        return Ok(());
    };

    // WALK SOUND
    if step_timer.tick(time.delta()).just_finished() && basis.standing_on_entity().is_some() {
        let mut rng = thread_rng();
        let i = rng.gen_range(0..sources.steps.len());
        let handle = if actions.value::<Crouch>()?.as_bool() {
            // TODO: select crouch steps
            sources.steps[i].clone()
        } else {
            sources.steps[i].clone()
        };
        cmds.spawn((Sfx, SamplePlayer::new(handle).with_volume(settings.sfx())));
    }

    Ok(())
}

fn jump_sound(
    _: Trigger<Started<Jump>>,
    state: Res<GameState>,
    settings: Res<Settings>,
    sources: ResMut<AudioSources>,
    // jump_timer: Query<&JumpTimer, With<Player>>,
    mut cmds: Commands,
) -> Result {
    if state.muted || state.paused {
        return Ok(());
    }

    // let jump_timer = jump_timer.get(on.target())?;
    // if jump_timer.just_finished() {
    let mut rng = thread_rng();
    let i = rng.gen_range(0..sources.steps.len());
    let handle = sources.steps[i].clone();
    cmds.spawn((Sfx, SamplePlayer::new(handle).with_volume(settings.sfx())));
    // }

    Ok(())
}

fn dash_sound(
    _: Trigger<Started<Dash>>,
    state: Res<GameState>,
    settings: Res<Settings>,
    sources: ResMut<AudioSources>,
    // jump_timer: Query<&JumpTimer, With<Player>>,
    mut cmds: Commands,
) -> Result {
    if state.muted || state.paused {
        return Ok(());
    }

    // let jump_timer = jump_timer.get(on.target())?;
    // if jump_timer.just_finished() {
    let mut rng = thread_rng();
    let i = rng.gen_range(0..sources.steps.len());
    let handle = sources.steps[i].clone();
    cmds.spawn((Sfx, SamplePlayer::new(handle).with_volume(settings.sfx())));
    // }

    Ok(())
}
