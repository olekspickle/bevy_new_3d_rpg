use super::*;

pub fn plugin(app: &mut App) {
    app.add_event::<Back>()
        .add_event::<GoTo>()
        .add_event::<OnPress>()
        .add_event::<ChangeMood>()
        .add_event::<SettingsChanged>()
        .add_event::<SwitchTab>()
        .add_event::<NewModal>()
        .add_event::<PopModal>()
        .add_event::<ClearModals>()
        .add_event::<FovIncrement>()
        .add_event::<CamCursorToggle>()
        .add_event::<ToggleVsync>()
        .add_event::<ToggleMute>()
        .add_event::<TogglePause>()
        .add_event::<ToggleDebugUi>()
        .add_event::<ToggleDiagnostics>()
        .add_observer(pause)
        .add_observer(mute)
        .add_observer(back);
}

#[derive(Event)]
pub struct GoTo(pub Screen);
#[derive(Event)]
pub struct Back(pub Screen);
#[derive(Event, Deref)]
pub struct SwitchTab(pub UiTab);
#[derive(Event, Deref)]
pub struct NewModal(pub Modal);
#[derive(Event)]
pub struct PopModal;
#[derive(Event)]
pub struct ClearModals;
#[derive(Event)]
pub struct CamCursorToggle;
#[derive(Event)]
pub struct FovIncrement;
#[derive(Event)]
pub struct ToggleVsync;
#[derive(Event)]
pub struct TogglePause;
#[derive(Event)]
pub struct ToggleMute;
#[derive(Event)]
pub struct ToggleDiagnostics;
#[derive(Event)]
pub struct ToggleDebugUi;
#[derive(Event)]
pub struct ChangeMood(pub MoodType);
/// Event triggered on a UI entity when the [`Interaction`] component on the same entity changes to
/// [`Interaction::Pressed`]. Observe this event to detect e.g. button presses.
#[derive(Event)]
pub struct OnPress;
#[derive(Event)]
pub struct SettingsChanged;

// ================== trigger events on input ========================
fn back(
    _: Trigger<Started<Escape>>,
    screen: Res<State<Screen>>,
    states: Res<GameState>,
    mut commands: Commands,
) {
    match screen.get() {
        Screen::Splash | Screen::Title | Screen::Loading => {}
        _ => {
            let last = states.last_screen.clone();
            commands.trigger(Back(last));
        }
    }
}
fn pause(_: Trigger<Started<Pause>>, mut commands: Commands) {
    commands.trigger(TogglePause);
}
fn mute(_: Trigger<Started<Mute>>, mut commands: Commands) {
    commands.trigger(ToggleMute);
}
