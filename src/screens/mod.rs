//! The game's main screen states and transitions between them.
use crate::asset_loading::{AudioSources, ResourceHandles, Textures};
use crate::game::{ToggleMute, TogglePause};
use crate::markers;
use crate::shared::{AppSystems, GameState, Settings};
use crate::ui::{
    self, MainMenuCtx, Modal, ModalInput, Modals, NewModal, PopModal, Props, colors, settings_ui,
    widget,
};
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_enhanced_input::prelude::{InputAction, Start};
use bevy_seedling::prelude::*;

mod credits;
mod gameplay;
mod loading;
mod settings;
mod splash;
mod title;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        splash::plugin,
        loading::plugin,
        title::plugin,
        settings::plugin,
        credits::plugin,
        gameplay::plugin,
    ))
    .add_systems(OnEnter(Screen::Gameplay), unpause_on_enter)
    .add_systems(Update, track_last_screen.run_if(state_changed::<Screen>))
    .add_observer(on_esc)
    .add_observer(on_back)
    .add_observer(on_go_to);
}

/// The game's main screen states.
/// See <https://bevy-cheatbook.github.io/programming/states.html>
/// Or <https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs>
#[derive(Component, States, Default, Clone, Eq, PartialEq, Debug, Hash, Reflect)]
pub enum Screen {
    // TODO: splash should be first
    // Bevy tribute <3
    Splash,
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    Tutorial,
    Credits,
    Settings,
    // Here the menu is drawn and waiting for player interaction
    Title,
    // During this State the actual game logic is executed
    Gameplay,
}

impl Screen {
    pub fn is_gameplay(&self) -> bool {
        matches!(self, Self::Gameplay)
    }
}

/// Back/escape navigation, bound in both player and modal input contexts.
#[derive(InputAction)]
#[action_output(bool)]
pub struct Escape;

fn unpause_on_enter(mut state: ResMut<GameState>, mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() || state.paused {
        time.unpause();
        state.reset();
    }
}

// TODO: figure out how to make it a cool observer
// mut transitions: On<StateTransitionEvent<Screen>>,
fn track_last_screen(
    mut transitions: MessageReader<StateTransitionEvent<Screen>>,
    mut state: ResMut<GameState>,
) {
    let Some(transition) = transitions.read().last() else {
        return;
    };
    state.last_screen = transition.clone().exited.unwrap_or(Screen::Title);
}

#[derive(Event)]
pub struct GoTo(pub Screen);
#[derive(EntityEvent)]
pub struct Back {
    pub entity: Entity,
    pub screen: Screen,
}

pub fn click_go_to(
    on: On<Pointer<Click>>,
    screens: Query<&Screen>,
    parent_q: Query<&ChildOf>,
    mut commands: Commands,
) {
    if let Ok(screen) = parent_q.get(on.entity)
        && let Ok(screen) = screens.get(screen.parent())
    {
        commands.trigger(GoTo(screen.clone()));
    }
}

fn on_esc(
    on: On<Start<Escape>>,
    state: Res<GameState>,
    screen: Res<State<Screen>>,
    mut commands: Commands,
) {
    match screen.get() {
        Screen::Splash | Screen::Title | Screen::Loading => {}
        _ => {
            let last = state.last_screen.clone();
            commands.trigger(Back {
                entity: on.event_target(),
                screen: last,
            });
        }
    }
}

fn on_back(
    trigger: On<Back>,
    screen: Res<State<Screen>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    // Do not go to the title on back, we'd rather handle it in gameplay observers
    if screen.get().is_gameplay() {
        return;
    }

    let back = trigger.event();
    next_screen.set(back.screen.clone());
}

pub fn on_go_to(goto: On<GoTo>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(goto.event().0.clone());
}
