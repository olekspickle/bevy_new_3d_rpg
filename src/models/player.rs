use super::*;
use std::collections::HashMap;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct Player {
    pub id: Entity,
    pub speed: f32,
    pub animation_state: AnimationState,
    pub animations: HashMap<String, AnimationNodeIndex>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: Entity::PLACEHOLDER,
            speed: 1.0,
            animation_state: AnimationState::StandIdle,
            animations: HashMap::new(),
        }
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub enum AnimationState {
    #[default]
    StandIdle,
    Run(f32),
    Sprint(f32),
    Climb(f32),
    JumpStart,
    JumpLoop,
    JumpLand,
    Fall,
    Crawl(f32),
    Crouch,
    Dash,
    WallSlide,
    WallJump,
    KnockBack,
}
