//! Main animation system
//!
//! My idea is based on tnua approach - detect if we are to alter the animation state
//! and then change the animation state accordingly
//!
//! Create a new animation graph for each player, start all animations with 0 weight and add weight
//! exponentially between frames based on input
use super::*;
use bevy::world_serialization::WorldInstanceReady;
use std::time::Duration;

/// Animation control knobs
mod knobs {
    /// General damping factor to slow down animations using speed
    pub const DAMPING: f32 = 0.05;
    pub const TRANSITION: f32 = 0.2;
    /// Use a slightly faster transition for jumps to make them feel "snappy"
    pub const JUMP_TRANSITION: f32 = 0.1;
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (calcucate_animations, animate)
            .chain()
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Build animation graph when scene loads
pub fn prepare_animations(
    on: On<WorldInstanceReady>,
    models: Res<Models>,
    gltfs: Res<Assets<Gltf>>,
    children_q: Query<&Children>,
    animation_player_q: Query<Entity, With<AnimationPlayer>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut players: Query<&mut Player>,
    mut commands: Commands,
) {
    let Some(animation_player_e) = on.entity.get_recursive(children_q, animation_player_q) else {
        return;
    };
    let Ok(mut animation_player) = animation_players.get_mut(animation_player_e) else {
        return;
    };
    let Some(gltf) = gltfs.get(&models.player) else {
        return;
    };

    // we list acnimations here in the same order they are listed in AnimationState enum
    let clips = vec![
        gltf.named_animations["Idle_Loop"].clone(),
        gltf.named_animations["Jog_Fwd_Loop"].clone(),
        gltf.named_animations["Sprint_Loop"].clone(),
        gltf.named_animations["Jump_Start"].clone(),
        gltf.named_animations["Jump_Loop"].clone(),
        gltf.named_animations["Jump_Land"].clone(),
        gltf.named_animations["Crouch_Fwd_Loop"].clone(),
        gltf.named_animations["Crouch_Idle_Loop"].clone(),
        gltf.named_animations["Roll"].clone(),
    ];

    let (graph, nodes) = AnimationGraph::from_clips(clips);
    let graph_handle = graphs.add(graph);

    commands
        .entity(animation_player_e)
        .insert(AnimationGraphHandle(graph_handle))
        .insert(AnimationTransitions::default());

    let idle_node = nodes[0];
    animation_player.play(idle_node).repeat();

    if let Ok(mut player) = players.single_mut() {
        debug!("adding animations to player: {}", player.id);
        player.animation = Animations {
            // state: (AnimationState::StandIdle, AnimationState::StandIdle),
            current: AnimationState::StandIdle,
            requested: None,
            nodes,
            animation_player_e,
        };
    }
}

pub fn calcucate_animations(
    time: Res<Time>,
    cfg: Res<Config>,
    mut players: Query<(
        Entity,
        &CharacterController,
        &CharacterControllerState,
        &Transform,
        &mut PreviousPosition,
        &mut Player,
    )>,
    mut commands: Commands,
) {
    let idle_to_run_ani = cfg.player.movement.idle_to_run_threshold * 1000.0;

    for (entity, ahoy, ahoy_state, pos, mut prev_pos, mut player) in players.iter_mut() {
        let animation = &mut player.animation;

        let displacement = pos.translation - prev_pos.0;
        let velocity = displacement / time.delta_secs();
        let h_speed = Vec3::new(velocity.x, 0.0, velocity.z).length().abs();
        let v_speed = velocity.y;
        prev_pos.0 = pos.translation;

        let moving = h_speed > idle_to_run_ani;
        let grounded = ahoy_state.grounded.is_some();
        // tracing::debug!(
        //     "player speed: {h_speed}, idle to run threshold: {idle_to_run_ani}, grounded: {grounded}"
        // );

        // MANTLE
        if ahoy_state.mantle.is_some() {
            animation.request(AnimationState::Dash); // or Mantle
            continue;
        }

        // Handle stationary players: if speed is near zero and we're not moving up,
        // treat as grounded to avoid false jump animations when physics incorrectly reports ungrounded
        let treat_as_grounded =
            !grounded && h_speed < 0.1 && v_speed <= 0.1 && !animation.current.is_jumping();
        let effectively_grounded = grounded || treat_as_grounded;

        // in the air animation
        if !effectively_grounded {
            if !animation.current.is_jumping() {
                if v_speed > 0.1 {
                    animation.request(AnimationState::Jump);
                } else {
                    animation.request(AnimationState::JumpLoop);
                }
            }
            continue;
        }

        // at this point we are GROUNDED
        // Only transition from jump to land/run - ignore if crouching
        if effectively_grounded && animation.current.is_jumping() && !ahoy_state.crouching {
            commands.entity(entity).trigger(PlayerLanded);
            if moving {
                animation.request(AnimationState::Run(ahoy.speed));
            } else {
                animation.request(AnimationState::Land);
            }
            continue;
        }

        // Also handle releasing crouch: if in crouch animation but not holding button, go to StandIdle
        if animation.current.is_crouching() && !ahoy_state.crouching {
            animation.request(AnimationState::StandIdle);
            continue;
        }

        // CROUCH - only request crouch if holding crouch button
        if ahoy_state.crouching && animation.current.can_crouch() {
            if moving {
                animation.request(AnimationState::Crouch(ahoy.speed));
            } else {
                animation.request(AnimationState::CrouchIdle);
            }
            continue;
        }

        // SPRINT
        let is_sprinting = ahoy.speed > cfg.player.movement.speed();
        if is_sprinting {
            animation.request(AnimationState::Sprint(ahoy.speed));
            continue;
        }

        // and finally RUN\IDLE
        if moving {
            animation.request(AnimationState::Run(ahoy.speed));
        } else {
            animation.request(AnimationState::StandIdle);
        }
    }
}

pub fn animate(
    mut players: Query<&mut Player>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut transitions_query: Query<&mut AnimationTransitions>,
) {
    for mut player in players.iter_mut() {
        let ani = &mut player.animation;
        let Ok(mut animation_player) = animation_players.get_mut(ani.animation_player_e) else {
            continue;
        };

        let Ok(mut transitions) = transitions_query.get_mut(ani.animation_player_e) else {
            continue;
        };

        if let Some(next) = ani.requested.take() {
            let node = ani.nodes[next.idx()];

            let duration = if next.is_jumping() {
                knobs::JUMP_TRANSITION
            } else {
                knobs::TRANSITION
            };

            // debug!("animate: playing {:?} duration:{}", next, duration);

            transitions.play(
                &mut animation_player,
                node,
                Duration::from_secs_f32(duration),
            );

            if !next.is_locked() {
                animation_player.animation_mut(node).map(|a| a.repeat());
            }

            // Set speed on the NEW animation, not the old one
            let next_node = ani.nodes[next.idx()];
            if let Some(active) = animation_player.animation_mut(next_node) {
                match next {
                    AnimationState::Run(s)
                    | AnimationState::Sprint(s)
                    | AnimationState::Crouch(s) => active.set_speed(s * knobs::DAMPING),
                    AnimationState::Land => active.set_speed(2.5),
                    _ => active.set_speed(1.0),
                };
            }

            ani.current = next;
        }

        if ani.current.is_locked() {
            let node = ani.nodes[ani.current.idx()];

            if let Some(active) = animation_player.animation(node)
                && active.is_finished()
            {
                let chained = match ani.current {
                    AnimationState::Jump => Some(AnimationState::JumpLoop),
                    AnimationState::Land => Some(AnimationState::StandIdle),
                    _ => None,
                };

                if let Some(next) = chained {
                    // debug!("chained animation: {next:?}");
                    ani.requested = Some(next);
                }
            }
        }
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct Animations {
    /// (current, next)
    // pub state: (AnimationState, AnimationState),
    pub current: AnimationState,
    pub requested: Option<AnimationState>,
    /// AnimationState → Graph node
    pub nodes: Vec<AnimationNodeIndex>,
    /// Entity that owns AnimationPlayer
    pub animation_player_e: Entity,
}

impl Default for Animations {
    fn default() -> Self {
        Self {
            // state: (AnimationState::StandIdle, AnimationState::StandIdle),
            current: AnimationState::StandIdle,
            requested: None,
            nodes: Vec::new(),
            animation_player_e: Entity::PLACEHOLDER,
        }
    }
}

/// State change helpers
impl Animations {
    fn request(&mut self, state: AnimationState) {
        let is_moving = self.current.is_moving();
        let is_jumping_to_land = self.current.is_jumping() && matches!(state, AnimationState::Land);
        let requested_is_movement = matches!(
            state,
            AnimationState::Run(_) | AnimationState::Sprint(_) | AnimationState::Crouch(_)
        );
        let same_clip = self.current.idx() == state.idx();
        let is_locked = self.current.is_locked();

        // debug!(
        //     "request: curr={:?} next={:?} same_clip={}",
        //     self.current, state, same_clip
        // );

        if is_locked && !is_jumping_to_land && !is_moving && !requested_is_movement {
            return;
        }

        if same_clip {
            self.current = state;
        } else {
            self.requested = Some(state);
        }
    }

    pub fn idle(&mut self) {
        self.request(AnimationState::StandIdle);
    }

    pub fn run(&mut self, speed: f32) {
        self.request(AnimationState::Run(speed));
    }

    pub fn sprint(&mut self, speed: f32) {
        self.request(AnimationState::Sprint(speed));
    }

    pub fn crouch(&mut self, speed: f32) {
        self.request(AnimationState::Crouch(speed));
    }

    pub fn crouch_idle(&mut self) {
        self.request(AnimationState::CrouchIdle);
    }

    pub fn dash(&mut self) {
        self.request(AnimationState::Dash);
    }

    pub fn fall(&mut self) {
        self.request(AnimationState::JumpLoop);
    }

    pub fn start_jump(&mut self) {
        self.request(AnimationState::Jump);
    }
    pub fn end_jump(&mut self) {
        self.request(AnimationState::Land);
    }
}

/// The order is important here because we use it as indexes for animation node vec
#[derive(Component, Default, Reflect, Clone, Copy, PartialEq, Debug)]
#[reflect(Component)]
pub enum AnimationState {
    #[default]
    StandIdle,
    Run(f32),
    Sprint(f32),
    Jump,
    JumpLoop,
    Land,
    Crouch(f32),
    CrouchIdle,
    Dash,
}
impl AnimationState {
    pub fn idx(&self) -> usize {
        match self {
            AnimationState::StandIdle => 0,
            AnimationState::Run(_) => 1,
            AnimationState::Sprint(_) => 2,
            AnimationState::Jump => 3,
            AnimationState::JumpLoop => 4,
            AnimationState::Land => 5,
            AnimationState::Crouch(_) => 6,
            AnimationState::CrouchIdle => 7,
            AnimationState::Dash => 8,
        }
    }

    /// Animations that should not be interrupted by other animations
    pub fn is_locked(&self) -> bool {
        matches!(
            self,
            AnimationState::Jump | AnimationState::Land | AnimationState::Dash
        )
    }
    pub fn can_crouch(&self) -> bool {
        !matches!(
            self,
            AnimationState::Jump
                | AnimationState::JumpLoop
                | AnimationState::Land
                | AnimationState::Dash
        )
    }
    pub fn is_running(&self) -> bool {
        matches!(self, AnimationState::Run(_))
    }
    pub fn is_falling(&self) -> bool {
        matches!(self, AnimationState::JumpLoop)
    }
    pub fn is_jumping(&self) -> bool {
        matches!(self, AnimationState::Jump | AnimationState::JumpLoop)
    }
    pub fn is_crouching(&self) -> bool {
        matches!(self, AnimationState::Crouch(_) | AnimationState::CrouchIdle)
    }
    pub fn is_landing(&self) -> bool {
        matches!(self, AnimationState::Land)
    }
    pub fn is_moving(&self) -> bool {
        matches!(
            self,
            AnimationState::Run(_) | AnimationState::Sprint(_) | AnimationState::Crouch(_)
        )
    }
}
