//! The game's main screen states and transitions between them.

use super::*;
use bevy::{prelude::*, ui::Val::*};
use bevy_seedling::prelude::*;

mod credits;
mod gameplay;
mod loading;
mod settings;
mod splash;
mod title;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.enable_state_scoped_entities::<Screen>();

    app.add_plugins((
        splash::plugin,
        loading::plugin,
        title::plugin,
        settings::plugin,
        credits::plugin,
        gameplay::plugin,
    ))
    .add_systems(Update, track_last_screen.run_if(state_changed::<Screen>))
    .add_observer(on_back)
    .add_observer(on_go_to);
}

// TODO: figure out how to make it a cool observer
// mut transitions: Trigger<StateTransitionEvent<Screen>>,
fn track_last_screen(
    mut transitions: EventReader<StateTransitionEvent<Screen>>,
    mut state: ResMut<GameState>,
) {
    let Some(transition) = transitions.read().last() else {
        return;
    };
    state.last_screen = transition.clone().exited.unwrap_or(Screen::Title);
}

fn on_back(
    trigger: Trigger<Back>,
    mut next_screen: ResMut<NextState<Screen>>,
    screen: Res<State<Screen>>,
) {
    // Do not go to the title on back, we'd rather handle it in gameplay observers
    if *screen.get() == Screen::Gameplay {
        return;
    }

    let back = trigger.event();
    next_screen.set(back.0.clone());
}

pub fn on_go_to(trig: Trigger<GoTo>, mut next_screen: ResMut<NextState<Screen>>) {
    let go_to = trig.event();
    next_screen.set(go_to.0.clone());
}

// TODO: figure out nice click_go_to(Screen::Title) HOF
// fn click_go_to<E, B, M>(s: Screen) -> impl IntoObserverSystem<OnPress, B, M> {
//     |_: Trigger<OnPress>, mut cmds: Commands| cmds.trigger(OnGoTo(s.clone()))
// }
pub mod to {
    use super::*;

    pub fn title(_: Trigger<OnPress>, mut cmds: Commands, mut state: ResMut<GameState>) {
        state.reset();
        cmds.trigger(GoTo(Screen::Title));
    }
    pub fn settings(_: Trigger<OnPress>, mut cmds: Commands) {
        cmds.trigger(GoTo(Screen::Settings));
    }
    pub fn credits(_: Trigger<OnPress>, mut cmds: Commands) {
        cmds.trigger(GoTo(Screen::Credits));
    }
    pub fn gameplay_or_loading(
        _: Trigger<OnPress>,
        resource_handles: Res<ResourceHandles>,
        mut next_screen: ResMut<NextState<Screen>>,
    ) {
        if resource_handles.is_all_done() {
            next_screen.set(Screen::Gameplay);
        } else {
            next_screen.set(Screen::Loading);
        }
    }
}
