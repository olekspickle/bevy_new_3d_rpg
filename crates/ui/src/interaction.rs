use super::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<UiInteraction>().add_systems(
        Update,
        (
            apply_interaction_palette,
            (trigger_on_press, btn_sounds).run_if(resource_exists::<AudioSources>),
        ),
    );
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`]
/// and [`BorderColor`] based on the current interaction state.
///
/// Struct of pairs (bg_color, border_color)
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct UiInteraction {
    pub none: (Color, Color),
    pub hovered: (Color, Color),
    pub pressed: (Color, Color),
}
impl UiInteraction {
    pub const DEFAULT: Self = Self {
        none: (TRANSPARENT, WHITEISH),
        hovered: (LIGHT_BLUE, WHITEISH),
        pressed: (DIM_BLUE, WHITEISH),
    };
    pub fn all(c: Color) -> Self {
        Self {
            none: (c, c),
            hovered: (c, c),
            pressed: (c, c),
        }
    }
    pub fn none(mut self, c: (Color, Color)) -> Self {
        self.none = c;
        self
    }
    pub fn pressed(mut self, c: (Color, Color)) -> Self {
        self.pressed = c;
        self
    }
    pub fn hovered(mut self, c: (Color, Color)) -> Self {
        self.hovered = c;
        self
    }
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (
            &Interaction,
            &UiInteraction,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, Without<DisabledButton>),
    >,
) {
    for (interaction, palette, mut border_color, mut background) in &mut palette_query {
        let (bg, border) = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        };
        *background = bg.into();
        *border_color = border.into();
    }
}

fn trigger_on_press(
    interaction_query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut commands: Commands,
) {
    for (entity, interaction) in &interaction_query {
        if matches!(interaction, Interaction::Pressed) {
            commands.trigger_targets(OnPress, entity);
        }
    }
}

// TODO: not sure it's possible to do efficiently with observers in 3d, like in BevyFlock,
// it's dropping FPS like crazy
fn btn_sounds(
    mut commands: Commands,
    settings: Res<Settings>,
    audio_sources: Res<AudioSources>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, Without<DisabledButton>)>,
) {
    for interaction in &interaction_query {
        let source = match interaction {
            Interaction::Hovered => audio_sources.btn_hover.clone(),
            Interaction::Pressed => audio_sources.btn_press.clone(),
            _ => continue,
        };
        commands.spawn((
            Sfx,
            SamplePlayer::new(source.clone()).with_volume(settings.sfx()),
        ));
    }
}
