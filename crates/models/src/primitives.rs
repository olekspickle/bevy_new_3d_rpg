use super::*;

/// Macro to hide the derive trait boilerplate
macro_rules! declare_markers {
  ( $( $name:ident ),* ) => {
        $(
            #[derive(Component, Reflect, Clone)]
            #[reflect(Component)]
            pub struct $name;
        )*
    };
}

declare_markers!(
    SceneCamera,
    // scene
    Sun,
    Moon,
    // TODO: The idea is to create a boombox with spatial audio
    // <https://github.com/bevyengine/bevy/blob/main/examples/audio/spatial_audio_3d.rs>
    Boombox,
    SunCycleLabel,
    // user input context
    GlobalInputCtx,
    // UI: mostly for nodes or labels that have to change visibility or content at some point
    PerfUi,
    GameplayUi,
    PauseIcon,
    MuteIcon,
    MenuModal,
    // settings
    SettingsModal,
    TabBar,
    TabContent,
    DisabledButton,
    Slider,
    SliderThumb,
    Checkbox,
    GeneralVolumeLabel,
    MusicVolumeLabel,
    SfxVolumeLabel,
    DiagnosticsLabel,
    DebugUiLabel,
    SaveSettingsLabel,
    FovLabel,
    // animations
    FadeIn,
    FadeOut
);

macro_rules! timers {
  ( $( $name:ident ),* ) => {
        $(
            #[derive(Component, Reflect, Deref, DerefMut, Debug)]
            #[reflect(Component)]
            pub struct $name(pub Timer);
        )*
    };
}
timers!(JumpTimer, StepTimer);

// macro_rules! sliders {
//   ( $( $name:ident ),* ) => {
//         $(
//             #[derive(Component, Reflect, Debug)]
//             #[reflect(Component)]
//             pub struct $name{
//                 pub current: f32
//             }
//         )*
//     };
// }
//
// sliders!(SliderGeneral, SliderMusic, SliderSfx);
